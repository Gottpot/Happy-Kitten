//! Touch gestures: swiping scrolls, a long press lets you work on what is under the finger.
//!
//! On a phone a finger that starts on a clip or a file card should scroll the list, not drag the
//! thing. So inside the registered *zones* (the timeline lanes and the asset cards) the touch is
//! held back until it is clear what the user wants:
//!
//! * lifted quickly without moving → a tap, delivered as an ordinary click;
//! * moved before [`LONG_PRESS`] → a swipe: the app never sees a press, egui scrolls the area
//!   under the finger through mouse-wheel events (with a little fling after letting go);
//! * held still for [`LONG_PRESS`] → the press is delivered now (with a buzz) and from then on the
//!   finger works like a mouse: move and trim clips, drag cards onto the timeline, rubber-band.
//!
//! Everything outside the zones (buttons, sliders, the ruler, the playhead) behaves as before.
//! The filter works on egui's raw input, so none of the drawing code needs to know about it.

use egui::{Context, Event, Id, Modifiers, Order, PointerButton, Pos2, Rect, Vec2, epaint::Color32};
use std::collections::VecDeque;

/// How long a finger must rest before it grabs the clip or card under it.
pub const LONG_PRESS: f64 = 0.38;
/// Movement (in points) that still counts as "resting".
const SLOP: f32 = 10.0;
/// A fling slower than this (points per second) is not started.
const FLING_MIN: f32 = 260.0;
/// No fling starts faster than this (points per second).
const FLING_MAX: f32 = 3200.0;
/// Fling slows down by this factor per second (exponent).
const FLING_DECAY: f32 = 3.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axes {
    Both,
    Vertical,
}

#[derive(Clone, Copy)]
struct Zone {
    axes: Axes,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lock {
    X,
    Y,
}

enum State {
    Idle,
    /// Finger down inside a zone, nothing sent to the app yet.
    Pending {
        start: Pos2,
        latest: Pos2,
        since: f64,
        axes: Axes,
        modifiers: Modifiers,
    },
    Scroll {
        last: Pos2,
        lock: Lock,
        vel: Vec2,
        last_move: f64,
        last_frame: f64,
        frame_delta: Vec2,
    },
    Fling {
        vel: Vec2,
        last_frame: f64,
    },
    /// The press was delivered; events pass until the finger lifts.
    Armed,
    /// Two fingers (pinch) or something we do not understand: events pass untouched.
    Ignore,
}

pub struct Touch {
    state: State,
    /// Fingers currently on the glass, from egui's `Touch` events.
    fingers: u32,
    /// Events to deliver at the start of the next frame (the release of a tap, ...).
    queue: VecDeque<Event>,
    /// The `PointerGone` that follows a lift is held until our own events have been seen.
    hold_gone: bool,
    /// Where and when the last long press took hold, for the little ring.
    pulse: Option<(Pos2, f64)>,
    vibrate: bool,
}

impl Default for Touch {
    fn default() -> Self {
        Touch {
            state: State::Idle,
            fingers: 0,
            queue: VecDeque::new(),
            hold_gone: false,
            pulse: None,
            vibrate: false,
        }
    }
}

fn zones_id(next: bool) -> Id {
    Id::new(if next { "hk-touch-zones-next" } else { "hk-touch-zones" })
}

/// Registers a rectangle (clipped to what is visible) where swiping should scroll and a long
/// press should be needed to work on the content. Call it while drawing, every frame.
pub fn zone(ui: &egui::Ui, rect: Rect, axes: Axes) {
    let rect = rect.intersect(ui.clip_rect());
    if rect.width() < 1.0 || rect.height() < 1.0 {
        return;
    }
    ui.ctx().data_mut(|d| {
        d.get_temp_mut_or_default::<Vec<(Rect, bool)>>(zones_id(true))
            .push((rect, axes == Axes::Vertical));
    });
}

/// Call once at the end of every frame: this frame's zones become the ones the next frame's
/// input is checked against.
pub fn end_frame(ctx: &Context) {
    ctx.data_mut(|d| {
        let next: Vec<(Rect, bool)> = d
            .get_temp_mut_or_default::<Vec<(Rect, bool)>>(zones_id(true))
            .drain(..)
            .collect();
        *d.get_temp_mut_or_default::<Vec<(Rect, bool)>>(zones_id(false)) = next;
    });
}

fn zone_at(ctx: &Context, pos: Pos2) -> Option<Zone> {
    // Windows, menus and dialogs lie above the zones: a press on them is not ours.
    if ctx
        .layer_id_at(pos)
        .is_some_and(|l| l.order != Order::Background)
    {
        return None;
    }
    ctx.data(|d| {
        d.get_temp::<Vec<(Rect, bool)>>(zones_id(false))
            .unwrap_or_default()
            .into_iter()
            .rev()
            .find(|(r, _)| r.contains(pos))
            .map(|(_, vertical)| Zone {
                axes: if vertical { Axes::Vertical } else { Axes::Both },
            })
    })
}

fn wheel(delta: Vec2) -> Event {
    Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta,
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    }
}

impl Touch {
    /// True once per long press; the caller buzzes the phone.
    pub fn take_vibrate(&mut self) -> bool {
        std::mem::take(&mut self.vibrate)
    }

    /// Runs on the raw input of every frame, before egui sees it.
    pub fn filter(&mut self, ctx: &Context, raw: &mut egui::RawInput) {
        let now = raw.time.unwrap_or(0.0);
        let mut out: Vec<Event> = self.queue.drain(..).collect();

        for ev in std::mem::take(&mut raw.events) {
            match ev {
                Event::Touch { phase, .. } => {
                    match phase {
                        egui::TouchPhase::Start => self.fingers += 1,
                        egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                            self.fingers = self.fingers.saturating_sub(1);
                        }
                        egui::TouchPhase::Move => {}
                    }
                    if self.fingers >= 2 && !matches!(self.state, State::Armed) {
                        // A pinch: hand everything to egui as it is. A press we were still
                        // holding back is dropped, the pinch only needs the touch events.
                        self.state = State::Ignore;
                    }
                    out.push(ev);
                }
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers,
                } => {
                    if matches!(self.state, State::Fling { .. }) {
                        // a finger stops the fling; the hidden pointer is gone for good
                        self.state = State::Idle;
                        self.hold_gone = false;
                    }
                    let zone = if self.fingers == 1 && matches!(self.state, State::Idle) {
                        zone_at(ctx, pos)
                    } else {
                        None
                    };
                    match zone {
                        Some(z) => {
                            // Hover only, so the area under the finger is the one that scrolls.
                            out.push(Event::PointerMoved(pos));
                            log::debug!("Geste: Berührung in Zone");
                            self.state = State::Pending {
                                start: pos,
                                latest: pos,
                                since: now,
                                axes: z.axes,
                                modifiers,
                            };
                        }
                        None => {
                            if self.fingers <= 1 {
                                self.state = State::Idle;
                            }
                            out.push(ev);
                        }
                    }
                }
                Event::PointerMoved(p) => match &mut self.state {
                    State::Pending {
                        start,
                        latest,
                        axes,
                        ..
                    } => {
                        *latest = p;
                        let d = p - *start;
                        if d.length() > SLOP {
                            let lock = if *axes == Axes::Vertical || d.y.abs() >= d.x.abs() {
                                Lock::Y
                            } else {
                                Lock::X
                            };
                            let first = match lock {
                                Lock::X => Vec2::new(d.x, 0.0),
                                Lock::Y => Vec2::new(0.0, d.y),
                            };
                            log::debug!("Geste: Wischen {lock:?}");
                            out.push(wheel(first));
                            self.state = State::Scroll {
                                last: p,
                                lock,
                                vel: Vec2::ZERO,
                                last_move: now,
                                last_frame: now,
                                frame_delta: first,
                            };
                        }
                    }
                    State::Scroll {
                        last,
                        lock,
                        last_move,
                        frame_delta,
                        ..
                    } => {
                        let d = p - *last;
                        *last = p;
                        let d = match lock {
                            Lock::X => Vec2::new(d.x, 0.0),
                            Lock::Y => Vec2::new(0.0, d.y),
                        };
                        *frame_delta += d;
                        *last_move = now;
                        out.push(wheel(d));
                    }
                    _ => out.push(ev),
                },
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers,
                } => match std::mem::replace(&mut self.state, State::Idle) {
                    State::Pending { start, .. } => {
                        log::debug!("Geste: Tippen");
                        // A tap: press now, release in the next frame (egui wants to see the
                        // press at least one frame before the release).
                        out.push(Event::PointerButton {
                            pos: start,
                            button: PointerButton::Primary,
                            pressed: true,
                            modifiers,
                        });
                        self.queue.push_back(Event::PointerButton {
                            pos,
                            button: PointerButton::Primary,
                            pressed: false,
                            modifiers,
                        });
                        self.hold_gone = true;
                        ctx.request_repaint();
                    }
                    State::Scroll {
                        vel,
                        last_move,
                        ..
                    } => {
                        let moving = now - last_move < 0.09 && vel.length() > FLING_MIN;
                        if moving {
                            let vel = if vel.length() > FLING_MAX {
                                vel * (FLING_MAX / vel.length())
                            } else {
                                vel
                            };
                            self.state = State::Fling {
                                vel,
                                last_frame: now,
                            };
                            self.hold_gone = true;
                            ctx.request_repaint();
                        }
                    }
                    State::Ignore if self.fingers > 0 => {
                        self.state = State::Ignore;
                        out.push(ev);
                    }
                    _ => out.push(ev),
                },
                Event::PointerGone => {
                    if self.hold_gone {
                        // delivered when the tap's release or the fling is done
                        if !matches!(self.state, State::Fling { .. }) {
                            self.queue.push_back(Event::PointerGone);
                            self.hold_gone = false;
                        }
                    } else if matches!(self.state, State::Pending { .. } | State::Scroll { .. }) {
                        self.state = State::Idle;
                        out.push(ev);
                    } else {
                        out.push(ev);
                    }
                }
                other => out.push(other),
            }
        }

        // frame-wise work: long press, velocity, fling
        match &mut self.state {
            State::Pending {
                start,
                latest,
                since,
                modifiers,
                ..
            } => {
                if now - *since >= LONG_PRESS {
                    out.push(Event::PointerButton {
                        pos: *start,
                        button: PointerButton::Primary,
                        pressed: true,
                        modifiers: *modifiers,
                    });
                    if *latest != *start {
                        out.push(Event::PointerMoved(*latest));
                    }
                    log::debug!("Geste: Griff nach {:.2} s", now - *since);
                    self.pulse = Some((*start, now));
                    self.vibrate = true;
                    self.state = State::Armed;
                    ctx.request_repaint();
                } else {
                    ctx.request_repaint_after(std::time::Duration::from_millis(40));
                }
            }
            State::Scroll {
                vel,
                last_frame,
                frame_delta,
                ..
            } => {
                let dt = (now - *last_frame) as f32;
                if dt > 0.001 {
                    let v = *frame_delta / dt;
                    *vel = *vel * 0.5 + v * 0.5;
                    *last_frame = now;
                    *frame_delta = Vec2::ZERO;
                }
            }
            State::Fling { vel, last_frame } => {
                let dt = ((now - *last_frame) as f32).clamp(0.0, 0.05);
                *last_frame = now;
                out.push(wheel(*vel * dt));
                *vel *= (-FLING_DECAY * dt).exp();
                if vel.length() < 30.0 {
                    self.state = State::Idle;
                    if self.hold_gone {
                        self.hold_gone = false;
                        out.push(Event::PointerGone);
                    }
                } else {
                    ctx.request_repaint();
                }
            }
            State::Idle | State::Armed | State::Ignore => {}
        }
        if !self.queue.is_empty() {
            ctx.request_repaint();
        }
        if self.fingers == 0 && matches!(self.state, State::Ignore) {
            self.state = State::Idle;
        }
        raw.events = out;
    }

    /// A ring that spreads from the finger when a long press takes hold.
    pub fn draw_pulse(&mut self, ctx: &Context) {
        let Some((pos, since)) = self.pulse else {
            return;
        };
        let t = (ctx.input(|i| i.time) - since) as f32;
        if !(0.0..0.4).contains(&t) {
            self.pulse = None;
            return;
        }
        let k = t / 0.4;
        let alpha = ((1.0 - k) * 200.0) as u8;
        let painter = ctx.layer_painter(egui::LayerId::new(Order::Tooltip, Id::new("hk-pulse")));
        painter.circle_stroke(
            pos,
            14.0 + 34.0 * k,
            egui::Stroke::new(3.0, Color32::from_rgba_unmultiplied(233, 90, 130, alpha)),
        );
        ctx.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{TouchDeviceId, TouchId, TouchPhase, pos2, vec2};

    const ZONE_TOP: f32 = 100.0;

    /// Runs one egui pass and throws the paint output away.
    fn run(ctx: &Context, raw: egui::RawInput, f: impl FnMut(&mut egui::Ui)) {
        let mut out = ctx.run_ui(raw, f);
        out.textures_delta.clear();
    }

    struct Rig {
        ctx: Context,
        touch: Touch,
        time: f64,
    }

    impl Rig {
        fn new() -> Rig {
            let mut rig = Rig {
                ctx: Context::default(),
                touch: Touch::default(),
                time: 0.0,
            };
            rig.frame(vec![]);
            rig
        }

        /// One frame at `dt` seconds after the last; returns what egui was given.
        fn frame_after(&mut self, dt: f64, events: Vec<Event>) -> Vec<Event> {
            self.time += dt;
            let mut raw = egui::RawInput {
                time: Some(self.time),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 800.0))),
                events,
                ..Default::default()
            };
            self.touch.filter(&self.ctx, &mut raw);
            let delivered = raw.events.clone();
            run(&self.ctx, raw, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    zone(
                        ui,
                        Rect::from_min_max(pos2(0.0, ZONE_TOP), pos2(400.0, 500.0)),
                        Axes::Both,
                    );
                });
                end_frame(ui.ctx());
            });
            delivered
        }

        fn frame(&mut self, events: Vec<Event>) -> Vec<Event> {
            self.frame_after(0.016, events)
        }
    }

    fn touch(phase: TouchPhase, pos: Pos2) -> Event {
        Event::Touch {
            device_id: TouchDeviceId(0),
            id: TouchId(1),
            phase,
            pos,
            force: None,
        }
    }

    fn down(pos: Pos2) -> Vec<Event> {
        vec![
            touch(TouchPhase::Start, pos),
            Event::PointerMoved(pos),
            button(pos, true),
        ]
    }

    fn up(pos: Pos2) -> Vec<Event> {
        vec![
            touch(TouchPhase::End, pos),
            button(pos, false),
            Event::PointerGone,
        ]
    }

    fn button(pos: Pos2, pressed: bool) -> Event {
        Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }
    }

    fn presses(events: &[Event]) -> usize {
        events
            .iter()
            .filter(|e| matches!(e, Event::PointerButton { pressed: true, .. }))
            .count()
    }

    fn wheel_y(events: &[Event]) -> f32 {
        events
            .iter()
            .filter_map(|e| match e {
                Event::MouseWheel { delta, .. } => Some(delta.y),
                _ => None,
            })
            .sum()
    }

    #[test]
    fn a_tap_inside_a_zone_becomes_a_click_one_frame_later() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        assert_eq!(presses(&rig.frame(down(p))), 0, "press is held back");
        let released = rig.frame(up(p));
        assert_eq!(presses(&released), 1, "tap delivers the press");
        assert!(!released.iter().any(|e| matches!(e, Event::PointerGone)));
        let next = rig.frame(vec![]);
        assert!(matches!(
            next.first(),
            Some(Event::PointerButton { pressed: false, .. })
        ));
        assert!(matches!(next.last(), Some(Event::PointerGone)));
    }

    #[test]
    fn a_swipe_scrolls_and_never_presses() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        rig.frame(down(p));
        let moved = rig.frame(vec![Event::PointerMoved(p + vec2(0.0, 40.0))]);
        assert_eq!(presses(&moved), 0);
        assert!(wheel_y(&moved) > 39.0, "the content follows the finger");
        let more = rig.frame(vec![Event::PointerMoved(p + vec2(0.0, 70.0))]);
        assert!((wheel_y(&more) - 30.0).abs() < 0.01);
        let end = rig.frame(up(p + vec2(0.0, 70.0)));
        assert_eq!(presses(&end), 0);
        assert!(!end.iter().any(|e| matches!(e, Event::PointerButton { .. })));
    }

    #[test]
    fn a_swipe_locks_to_its_main_direction() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        rig.frame(down(p));
        let moved = rig.frame(vec![Event::PointerMoved(p + vec2(40.0, 6.0))]);
        let wheel: Vec<Vec2> = moved
            .iter()
            .filter_map(|e| match e {
                Event::MouseWheel { delta, .. } => Some(*delta),
                _ => None,
            })
            .collect();
        assert_eq!(wheel, vec![vec2(40.0, 0.0)]);
    }

    #[test]
    fn a_long_press_delivers_the_press_and_then_acts_like_a_mouse() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        rig.frame(down(p));
        assert_eq!(presses(&rig.frame_after(0.2, vec![])), 0, "too early");
        let armed = rig.frame_after(LONG_PRESS, vec![]);
        assert_eq!(presses(&armed), 1, "the press arrives once the hold is long enough");
        assert!(rig.touch.take_vibrate());
        let drag = rig.frame(vec![Event::PointerMoved(p + vec2(60.0, 0.0))]);
        assert!(drag.iter().any(|e| matches!(e, Event::PointerMoved(_))));
        assert_eq!(wheel_y(&drag), 0.0, "no scrolling while a clip is held");
        let end = rig.frame(up(p + vec2(60.0, 0.0)));
        assert!(end.iter().any(|e| matches!(e, Event::PointerButton { pressed: false, .. })));
    }

    #[test]
    fn small_wobble_while_holding_still_counts_as_holding() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        rig.frame(down(p));
        rig.frame(vec![Event::PointerMoved(p + vec2(4.0, 3.0))]);
        let armed = rig.frame_after(LONG_PRESS, vec![]);
        assert_eq!(presses(&armed), 1);
    }

    #[test]
    fn outside_the_zones_nothing_changes() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 50.0); // above the zone: the ruler, a button, ...
        let delivered = rig.frame(down(p));
        assert_eq!(presses(&delivered), 1);
        let moved = rig.frame(vec![Event::PointerMoved(p + vec2(30.0, 0.0))]);
        assert!(moved.iter().any(|e| matches!(e, Event::PointerMoved(_))));
        assert_eq!(wheel_y(&moved), 0.0);
    }

    #[test]
    fn a_second_finger_leaves_the_pinch_to_egui() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        rig.frame(down(p));
        let second = Event::Touch {
            device_id: TouchDeviceId(0),
            id: TouchId(2),
            phase: TouchPhase::Start,
            pos: pos2(260.0, 300.0),
            force: None,
        };
        rig.frame(vec![second]);
        let moved = rig.frame(vec![Event::PointerMoved(p + vec2(40.0, 40.0))]);
        assert_eq!(wheel_y(&moved), 0.0, "a pinch does not scroll");
    }

    #[test]
    fn a_fast_swipe_keeps_scrolling_after_the_finger_left() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 400.0);
        rig.frame(down(p));
        rig.frame_after(0.02, vec![Event::PointerMoved(p + vec2(0.0, -30.0))]);
        rig.frame_after(0.02, vec![Event::PointerMoved(p + vec2(0.0, -70.0))]);
        rig.frame_after(0.02, vec![Event::PointerMoved(p + vec2(0.0, -120.0))]);
        let lift = rig.frame_after(0.01, up(p + vec2(0.0, -120.0)));
        assert!(!lift.iter().any(|e| matches!(e, Event::PointerGone)), "pointer stays for the fling");
        let fling = rig.frame_after(0.016, vec![]);
        assert!(wheel_y(&fling) < -1.0, "still moving up");
        let mut gone = false;
        for _ in 0..200 {
            gone |= rig
                .frame_after(0.016, vec![])
                .iter()
                .any(|e| matches!(e, Event::PointerGone));
        }
        assert!(gone, "the fling ends and lets the pointer go");
    }

    #[test]
    fn a_dialog_above_the_zone_keeps_its_touches() {
        let mut rig = Rig::new();
        let p = pos2(200.0, 300.0);
        // a window lying over the zone
        rig.time += 0.016;
        let raw = egui::RawInput {
            time: Some(rig.time),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 800.0))),
            ..Default::default()
        };
        run(&rig.ctx, raw, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                zone(
                    ui,
                    Rect::from_min_max(pos2(0.0, ZONE_TOP), pos2(400.0, 500.0)),
                    Axes::Both,
                );
            });
            egui::Window::new("dialog")
                .fixed_pos(pos2(100.0, 250.0))
                .fixed_size(vec2(200.0, 100.0))
                .show(&ui.ctx().clone(), |ui| {
                    ui.label("hello");
                });
            end_frame(ui.ctx());
        });
        // second frame so the window's layer is known to the hit test
        run(
            &rig.ctx,
            egui::RawInput {
                time: Some(rig.time + 0.016),
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(400.0, 800.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    zone(
                        ui,
                        Rect::from_min_max(pos2(0.0, ZONE_TOP), pos2(400.0, 500.0)),
                        Axes::Both,
                    );
                });
                egui::Window::new("dialog")
                    .fixed_pos(pos2(100.0, 250.0))
                    .fixed_size(vec2(200.0, 100.0))
                    .show(&ui.ctx().clone(), |ui| {
                        ui.label("hello");
                    });
                end_frame(ui.ctx());
            },
        );
        let delivered = rig.frame(down(p));
        assert_eq!(presses(&delivered), 1, "the press goes straight to the dialog");
    }
}
