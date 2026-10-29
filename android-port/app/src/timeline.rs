//! Timeline (bottom): toolbar, ruler, tracks with pastel clips, drag/trim/split,
//! transition badges and drop targets for media and cat effects.

use crate::app::{Gesture, HappyKitten, Payload, Selection};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::theme::*;
use crate::trf;
use egui::{Align2, Color32, CursorIcon, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use hk_core::{Id, MediaKind, TextSpec, Tick, TrackKind, Transition, TransitionKind};

const HEADER_W: f32 = 132.0;
/// Track header width on the phone: icon and mute button only.
const HEADER_W_PHONE: f32 = 60.0;
const RULER_H: f32 = 30.0;
const LANE_H: f32 = 58.0;
const LANE_GAP: f32 = 6.0;
const HANDLE_W: f32 = 9.0;
const SNAP_PX: f32 = 9.0;
/// Space before time zero so the playhead handle is not cut off.
const PAD_L: f32 = 14.0;

struct ClipView {
    id: Id,
    media: Id,
    start: Tick,
    end: Tick,
    source_in: Tick,
    color: u8,
    name: String,
    kind: MediaKind,
    transition: Option<Transition>,
    has_left: bool,
    is_text: bool,
    /// Set for effect clips.
    fx: Option<hk_core::FxKind>,
    is_overlay: bool,
    /// An image clip (can be switched between full picture and graphic).
    is_image: bool,
}

struct TrackView {
    id: Id,
    name: String,
    kind: TrackKind,
    muted: bool,
    clips: Vec<ClipView>,
}

enum Deferred {
    Select(Selection),
    Split(Id, Tick),
    Delete(Id, bool),
    ClearTransition(Id),
    DropMedia {
        media: Id,
        track: Id,
        at: Tick,
    },
    DropEffect {
        kind: TransitionKind,
        right_clip: Id,
    },
    DropText {
        spec: TextSpec,
        track: Id,
        at: Tick,
    },
    DropOverlay {
        media: Id,
        track: Id,
        at: Tick,
    },
    DropFx {
        kind: hk_core::FxKind,
        track: Id,
        at: Tick,
    },
    ToggleOverlay(Id),
    ApplyPreset(crate::app::TextPick, Id),
    DropLib {
        lib: Id,
        track: Id,
        at: Tick,
    },
    RemoveTrack(Id),
    Toggle(Id),
    /// Rubber-band result plus the selection to keep (shift held).
    SetMulti(Vec<Id>, Vec<Id>),
    Playhead(Tick),
    ToggleMute(Id),
}

impl HappyKitten {
    pub fn timeline_panel(&mut self, ui: &mut Ui) {
        let phone = self.phone;
        let screen_h = ui.ctx().content_rect().height();
        let m = if phone { 6 } else { 12 };
        let frame = egui::Frame::new().fill(CREAM).inner_margin(egui::Margin {
            left: m,
            right: m,
            top: 4,
            bottom: if phone { 4 } else { 12 },
        });
        let (default, range) = if phone {
            (screen_h * 0.26, 150.0..=screen_h * 0.6)
        } else {
            (350.0, 240.0..=640.0)
        };
        egui::Panel::bottom("timeline")
            .resizable(true)
            .default_size(default)
            .size_range(range)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                kit::card_frame(PEACH).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    self.timeline_toolbar(ui);
                    self.timeline_inspector(ui);
                    ui.add_space(2.0);
                    self.timeline_body(ui);
                });
            });
    }

    fn timeline_toolbar(&mut self, ui: &mut Ui) {
        if self.phone {
            // more buttons than the screen is wide: the row scrolls sideways
            egui::ScrollArea::horizontal()
                .id_salt("tl-toolbar")
                .auto_shrink([false, true])
                .show(ui, |ui| self.timeline_toolbar_row(ui));
        } else {
            self.timeline_toolbar_row(ui);
        }
    }

    fn timeline_toolbar_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if kit::round_button(ui, Icon::Undo, 34.0, false, self.doc.can_undo())
                .on_hover_text(tr("Rückgängig (Cmd+Z)"))
                .clicked()
            {
                self.doc.undo();
            }
            if kit::round_button(ui, Icon::Redo, 34.0, false, self.doc.can_redo())
                .on_hover_text(tr("Wiederholen (Cmd+Shift+Z)"))
                .clicked()
            {
                self.doc.redo();
            }
            ui.add_space(6.0);
            if kit::pill(ui, Some(Icon::Scissors), tr("Teilen"), false, true)
                .on_hover_text(tr("Clip am Zeiger teilen (S)"))
                .clicked()
            {
                self.split_at_playhead();
            }
            let can_delete = self.selection != Selection::None;
            if kit::pill(ui, Some(Icon::Trash), tr("Löschen"), false, can_delete)
                .on_hover_text(tr("Auswahl löschen (Entf), mit Shift Lücke schließen"))
                .clicked()
            {
                let ripple = ui.input(|i| i.modifiers.shift);
                self.delete_selection(ripple);
            }
            if kit::pill(ui, Some(Icon::Magnet), tr("Einrasten"), self.snap, true)
                .on_hover_text(tr("Clips rasten an Kanten und am Zeiger ein"))
                .clicked()
            {
                self.snap = !self.snap;
            }
            if kit::pill(ui, Some(Icon::Plus), tr("Videospur"), false, true)
                .on_hover_text(tr("Weitere Video-/Text-Spur oben auf den Stapel legen"))
                .clicked()
            {
                self.add_track(TrackKind::Video);
            }
            if kit::pill(ui, Some(Icon::Plus), tr("Audiospur"), false, true)
                .on_hover_text(tr("Weitere Audio-Spur"))
                .clicked()
            {
                self.add_track(TrackKind::Audio);
            }
            ui.add_space(6.0);
            ui.label(egui::RichText::new(tr("Zoom")).size(13.0).color(BROWN_SOFT));
            ui.add(
                egui::Slider::new(&mut self.pps, 15.0..=400.0)
                    .logarithmic(true)
                    .show_value(false),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some((msg, _)) = &self.toast {
                    ui.label(egui::RichText::new(msg).size(14.0).color(BROWN));
                }
            });
        });
    }

    fn timeline_inspector(&mut self, ui: &mut Ui) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::hover());
        let mut inner = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(vec2(4.0, 2.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let ui = &mut inner;
        match self.selection {
            Selection::Transition(id) => {
                let Some((kind, duration)) = self
                    .doc
                    .project
                    .clip(id)
                    .and_then(|(_, c)| c.transition_in)
                    .map(|t| (t.kind, t.duration))
                else {
                    self.set_selection(Selection::None);
                    return;
                };
                let max = self.doc.project.max_transition(id).secs().max(0.2) as f32;
                kit::draw_icon(
                    ui.painter(),
                    Icon::Paw,
                    Rect::from_center_size(
                        ui.cursor().left_center() + vec2(12.0, 0.0),
                        vec2(22.0, 22.0),
                    ),
                    PINK_DEEP,
                );
                ui.add_space(30.0);
                let mut new_kind = kind;
                egui::ComboBox::from_id_salt("tr-kind")
                    .selected_text(tr(kind.name()))
                    .width(170.0)
                    .show_ui(ui, |ui| {
                        for k in TransitionKind::ALL {
                            ui.selectable_value(&mut new_kind, k, tr(k.name()));
                        }
                    });
                if new_kind != kind {
                    self.doc.edit(|p| {
                        p.set_transition(
                            id,
                            Some(Transition {
                                kind: new_kind,
                                duration,
                            }),
                        )
                        .ok()
                    });
                }
                ui.label(tr("Dauer"));
                let mut secs = duration.secs() as f32;
                let r = ui.add(
                    egui::Slider::new(&mut secs, 0.2..=max)
                        .suffix(" s")
                        .fixed_decimals(1),
                );
                if r.drag_started() {
                    self.doc.begin_gesture();
                }
                if r.changed() {
                    let d = Tick::from_secs(secs as f64);
                    self.doc
                        .project
                        .set_transition(
                            id,
                            Some(Transition {
                                kind: new_kind,
                                duration: d,
                            }),
                        )
                        .ok();
                }
                if r.drag_stopped() {
                    self.doc.end_gesture();
                }
                if kit::pill(ui, Some(Icon::Trash), tr("Entfernen"), false, true).clicked() {
                    self.doc.edit(|p| p.set_transition(id, None).ok());
                    self.set_selection(Selection::Clip(id));
                }
            }
            Selection::Clip(_) if self.selected_clips().len() > 1 => {
                let n = self.selected_clips().len();
                kit::draw_icon(
                    ui.painter(),
                    Icon::Film,
                    Rect::from_center_size(
                        ui.cursor().left_center() + vec2(12.0, 0.0),
                        vec2(22.0, 22.0),
                    ),
                    BROWN,
                );
                ui.add_space(30.0);
                ui.label(egui::RichText::new(trf!("{n} Clips ausgewählt", n = n)).size(15.0));
                ui.label(
                    egui::RichText::new(tr("Ziehen = alle verschieben · Entf = alle löschen · Blenden-Tab: Blende für alle Schnittpunkte · Text-Tab: Stil für alle Texte"))
                        .size(12.0)
                        .color(BROWN_SOFT),
                );
            }
            Selection::Clip(id) => {
                let info = self.doc.project.clip(id).map(|(_, c)| {
                    (
                        c.duration.secs(),
                        match &c.text {
                            Some(t) => {
                                trf!("Text: {first}", first = t.text.lines().next().unwrap_or(""))
                            }
                            None => self
                                .doc
                                .project
                                .media(c.media)
                                .map(|m| m.name.clone())
                                .unwrap_or_default(),
                        },
                    )
                });
                if let Some((secs, name)) = info {
                    kit::draw_icon(
                        ui.painter(),
                        Icon::Film,
                        Rect::from_center_size(
                            ui.cursor().left_center() + vec2(12.0, 0.0),
                            vec2(22.0, 22.0),
                        ),
                        BROWN,
                    );
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new(format!("{name}  ·  {secs:.1} s")).size(15.0));
                    ui.label(
                        egui::RichText::new(tr("Ränder ziehen = kürzen · S = teilen"))
                            .size(12.5)
                            .color(BROWN_SOFT),
                    );
                } else {
                    self.set_selection(Selection::None);
                }
            }
            Selection::None => {
                ui.add_space(8.0);
                if !self.phone {
                    ui.label(egui::RichText::new(tr("Tipp: Ziehe Clips, Texte und Blenden aus dem Pool auf die Spuren, kürze sie an den Rändern und teile mit S.")).size(13.0).color(BROWN_SOFT));
                }
            }
        }
    }

    fn timeline_body(&mut self, ui: &mut Ui) {
        // Top of the screen = top of the stack: video tracks highest first, then audio tracks.
        let order: Vec<&hk_core::Track> = self
            .doc
            .project
            .tracks
            .iter()
            .rev()
            .filter(|t| t.kind == TrackKind::Video)
            .chain(
                self.doc
                    .project
                    .tracks
                    .iter()
                    .filter(|t| t.kind == TrackKind::Audio),
            )
            .collect();
        let tracks: Vec<TrackView> = order
            .into_iter()
            .map(|t| TrackView {
                id: t.id,
                name: t.name.clone(),
                kind: t.kind,
                muted: t.muted,
                clips: t
                    .clips
                    .iter()
                    .map(|c| {
                        let m = self.doc.project.media(c.media);
                        ClipView {
                            id: c.id,
                            media: c.media,
                            start: c.start,
                            end: c.end(),
                            source_in: c.source_in,
                            color: c.color,
                            name: match (&c.text, c.fx) {
                                (Some(t), _) => t.text.lines().next().unwrap_or("").to_string(),
                                (None, Some(f)) => tr(f.kind.name()).to_string(),
                                _ => m.map(|m| m.name.clone()).unwrap_or_else(|| "?".into()),
                            },
                            is_text: c.is_text(),
                            fx: c.fx.map(|f| f.kind),
                            is_overlay: c.is_overlay(),
                            is_image: !c.is_text() && m.is_some_and(|m| m.kind == MediaKind::Image),
                            kind: m.map_or(MediaKind::Image, |m| m.kind),
                            transition: c.transition_in,
                            has_left: self.doc.project.left_neighbour(c.id).is_some(),
                        }
                    })
                    .collect(),
            })
            .collect();
        let lanes_h = tracks.len() as f32 * (LANE_H + LANE_GAP);
        let mut deferred: Vec<Deferred> = Vec::new();
        egui::ScrollArea::vertical()
            .id_salt("tl-v")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let header_w = if self.phone { HEADER_W_PHONE } else { HEADER_W };
                    let (hrect, _) =
                        ui.allocate_exact_size(vec2(header_w, RULER_H + lanes_h), Sense::hover());
                    hk_touch::zone(
                        ui,
                        Rect::from_min_max(hrect.min + vec2(0.0, RULER_H), hrect.max),
                        hk_touch::Axes::Vertical,
                    );
                    for (i, t) in tracks.iter().enumerate() {
                        let r = Rect::from_min_size(
                            hrect.min + vec2(0.0, RULER_H + i as f32 * (LANE_H + LANE_GAP)),
                            vec2(header_w, LANE_H),
                        );
                        let hdr = ui.interact(r, egui::Id::new(("hdr", t.id)), Sense::click());
                        hdr.context_menu(|ui| {
                            if ui.button(tr("Spur löschen (wenn leer)")).clicked() {
                                deferred.push(Deferred::RemoveTrack(t.id));
                                ui.close();
                            }
                        });
                        let p = ui.painter();
                        p.rect_filled(
                            r,
                            round(14),
                            if t.kind == TrackKind::Video {
                                Color32::from_rgb(255, 214, 224)
                            } else {
                                Color32::from_rgb(226, 214, 250)
                            },
                        );
                        p.rect_stroke(r, round(14), ink(1.5), StrokeKind::Inside);
                        let icon = if t.kind == TrackKind::Video {
                            Icon::Paw
                        } else {
                            Icon::Note
                        };
                        kit::draw_icon(
                            p,
                            icon,
                            Rect::from_center_size(
                                r.left_center() + vec2(22.0, -9.0),
                                vec2(24.0, 24.0),
                            ),
                            BROWN,
                        );
                        if !self.phone {
                            p.text(
                                r.left_center() + vec2(40.0, -9.0),
                                Align2::LEFT_CENTER,
                                &t.name,
                                egui::FontId::proportional(14.5),
                                BROWN,
                            );
                        }
                        let mute = Rect::from_center_size(
                            r.left_center() + vec2(22.0, 18.0),
                            vec2(24.0, 20.0),
                        );
                        let resp = ui.interact(mute, egui::Id::new(("mute", t.id)), Sense::click());
                        let icon = match (t.kind, t.muted) {
                            (TrackKind::Video, false) => Icon::Film,
                            (TrackKind::Video, true) => Icon::SpeakerOff,
                            (TrackKind::Audio, false) => Icon::Speaker,
                            (TrackKind::Audio, true) => Icon::SpeakerOff,
                        };
                        kit::draw_icon(
                            ui.painter(),
                            icon,
                            mute.shrink(2.0),
                            if resp.hovered() { BROWN } else { BROWN_SOFT },
                        );
                        if resp
                            .on_hover_text(if t.kind == TrackKind::Video {
                                tr("Spur ein-/ausblenden")
                            } else {
                                tr("Spur stumm schalten")
                            })
                            .clicked()
                        {
                            deferred.push(Deferred::ToggleMute(t.id));
                        }
                    }
                    egui::ScrollArea::horizontal()
                        .id_salt("tl-scroll")
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            self.timeline_canvas(ui, &tracks, lanes_h, &mut deferred);
                        });
                });
            });
        self.run_deferred(deferred);
    }

    fn timeline_canvas(
        &mut self,
        ui: &mut Ui,
        tracks: &[TrackView],
        lanes_h: f32,
        deferred: &mut Vec<Deferred>,
    ) {
        let ctx = ui.ctx().clone();
        let pps = self.pps as f64;
        let fps = self.doc.project.fps;
        let visible_w = ui.available_width();
        let content_secs = (self.duration().secs() + 12.0).max(visible_w as f64 / pps);
        let (rect, _) = ui.allocate_exact_size(
            vec2((content_secs * pps) as f32 + PAD_L, RULER_H + lanes_h),
            Sense::hover(),
        );
        let x_of = |t: Tick| rect.left() + PAD_L + (t.secs() * pps) as f32;
        let t_of = |x: f32| Tick::from_secs(((x - rect.left() - PAD_L) as f64 / pps).max(0.0));

        // zoom with pinch / ctrl+scroll while the pointer is over the timeline
        if ctx
            .pointer_latest_pos()
            .is_some_and(|p| rect.contains(p) || ui.max_rect().contains(p))
        {
            let z = ctx.input(|i| i.zoom_delta());
            if (z - 1.0).abs() > 1e-4 {
                self.pps = (self.pps * z).clamp(15.0, 400.0);
            }
        }

        // ---- ruler
        let ruler = Rect::from_min_size(rect.min, vec2(rect.width(), RULER_H));
        // Below the ruler a swipe scrolls (also over clips); a long press grabs the clip.
        {
            let vis = ui.clip_rect();
            let top = (rect.top() + RULER_H).max(vis.top());
            hk_touch::zone(
                ui,
                Rect::from_min_max(pos2(vis.left(), top), vis.right_bottom()),
                hk_touch::Axes::Both,
            );
        }
        {
            let p = ui.painter();
            p.rect_filled(ruler, round(10), BEIGE);
            let step = [0.5, 1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0]
                .into_iter()
                .find(|s| (*s * pps) >= 70.0)
                .unwrap_or(600.0);
            let first = (ui.clip_rect().left() - rect.left() - PAD_L).max(0.0) as f64 / pps;
            let last = (ui.clip_rect().right() - rect.left() - PAD_L) as f64 / pps;
            let mut s = (first / step).floor() * step;
            while s <= last {
                let x = rect.left() + PAD_L + (s * pps) as f32;
                p.line_segment(
                    [
                        pos2(x, ruler.bottom() - 10.0),
                        pos2(x, ruler.bottom() - 2.0),
                    ],
                    Stroke::new(1.5, BROWN_SOFT),
                );
                let label = format!("{}:{:02}", (s / 60.0) as u32, (s % 60.0).floor() as u32);
                let label = if step < 1.0 && s.fract() > 0.01 {
                    format!("{label}.5")
                } else {
                    label
                };
                p.text(
                    pos2(x + 4.0, ruler.top() + 11.0),
                    Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(11.5),
                    BROWN,
                );
                let sub = step / 5.0;
                for k in 1..5 {
                    let sx = x + (sub * k as f64 * pps) as f32;
                    p.line_segment(
                        [
                            pos2(sx, ruler.bottom() - 5.0),
                            pos2(sx, ruler.bottom() - 2.0),
                        ],
                        Stroke::new(1.0, PEACH_DEEP),
                    );
                }
                s += step;
            }
        }
        let ruler_resp = ui.interact(ruler, egui::Id::new("ruler"), Sense::click_and_drag());
        if (ruler_resp.clicked() || ruler_resp.dragged())
            && let Some(pos) = ruler_resp.interact_pointer_pos()
        {
            self.stop();
            deferred.push(Deferred::Playhead(t_of(pos.x)));
        }

        let group_ids: std::collections::HashSet<Id> = match self.gesture {
            Gesture::MoveGroup { .. } => self.group_origins.iter().map(|(id, _)| *id).collect(),
            _ => Default::default(),
        };
        let snap_edges: Vec<Tick> = {
            let mut v = vec![Tick::ZERO, self.playhead];
            for t in tracks {
                for c in t.clips.iter().filter(|c| !group_ids.contains(&c.id)) {
                    v.push(c.start);
                    v.push(c.end);
                }
            }
            v
        };
        let modifier = ctx.input(|i| i.modifiers.shift || i.modifiers.command);

        // ---- lanes
        let payload = egui::DragAndDrop::payload::<Payload>(&ctx);
        let pointer = ctx.pointer_latest_pos();
        let lane_rects: Vec<Rect> = (0..tracks.len())
            .map(|i| {
                Rect::from_min_size(
                    pos2(
                        rect.left(),
                        rect.top() + RULER_H + i as f32 * (LANE_H + LANE_GAP) + LANE_GAP / 2.0,
                    ),
                    vec2(rect.width(), LANE_H),
                )
            })
            .collect();
        for (i, t) in tracks.iter().enumerate() {
            let lane = lane_rects[i];
            let p = ui.painter();
            let fill = if t.kind == TrackKind::Video {
                Color32::from_rgb(255, 244, 236)
            } else {
                Color32::from_rgb(250, 243, 252)
            };
            p.rect_filled(lane, round(14), fill);
            p.rect_stroke(
                lane,
                round(14),
                Stroke::new(1.2, PEACH_DEEP),
                StrokeKind::Inside,
            );
            let resp = ui.interact(lane, egui::Id::new(("lane", t.id)), Sense::click_and_drag());
            if resp.drag_started()
                && let Some(pos) = pointer
            {
                let base = if modifier {
                    self.selected_clips()
                } else {
                    Vec::new()
                };
                self.marquee = Some(crate::app::Marquee { start: pos, base });
            }
            if resp.clicked()
                && let Some(pos) = resp.interact_pointer_pos()
            {
                deferred.push(Deferred::Select(Selection::None));
                deferred.push(Deferred::Playhead(t_of(pos.x)));
            }
            // drop previews and drops
            if let (Some(pl), Some(pos)) = (payload.as_deref().copied(), pointer) {
                let info: Option<(bool, Tick)> = match pl {
                    Payload::Media(m) => self.doc.project.media(m).map(|item| {
                        let dur = if item.is_unbounded() {
                            Tick::from_secs(5.0)
                        } else {
                            item.duration
                        };
                        (
                            (item.kind == MediaKind::Audio) == (t.kind == TrackKind::Audio),
                            dur,
                        )
                    }),
                    Payload::LibMedia(l) => self.lib.item(l).map(|item| {
                        let dur = if item.kind == MediaKind::Image {
                            Tick::from_secs(5.0)
                        } else {
                            item.duration
                        };
                        (
                            (item.kind == MediaKind::Audio) == (t.kind == TrackKind::Audio),
                            dur,
                        )
                    }),
                    Payload::Text(_) => Some((t.kind == TrackKind::Video, Tick::from_secs(4.0))),
                    Payload::Overlay(_) => Some((t.kind == TrackKind::Video, Tick::from_secs(5.0))),
                    Payload::Fx(_) => Some((t.kind == TrackKind::Video, Tick::from_secs(5.0))),
                    Payload::Effect(_) => None,
                };
                if let (Some((ok, dur)), true) = (info, lane.contains(pos)) {
                    let wanted =
                        self.snap_span(t_of(pos.x), dur, &snap_edges, Tick(-1), Tick(-1), pps);
                    let at = self
                        .doc
                        .project
                        .free_start(t.id, None, wanted, dur)
                        .unwrap_or(Tick::ZERO);
                    let ghost = Rect::from_min_max(
                        pos2(x_of(at), lane.top() + 5.0),
                        pos2(x_of(at + dur), lane.bottom() - 5.0),
                    );
                    let col = if ok {
                        PINK
                    } else {
                        Color32::from_rgb(230, 140, 140)
                    };
                    let p = ui.painter();
                    p.rect_filled(ghost, round(11), col.gamma_multiply(0.45));
                    p.rect_stroke(
                        ghost,
                        round(11),
                        Stroke::new(2.0, BROWN),
                        StrokeKind::Inside,
                    );
                    if let Some(dropped) = resp.dnd_release_payload::<Payload>() {
                        match *dropped {
                            Payload::Media(media) => deferred.push(Deferred::DropMedia {
                                media,
                                track: t.id,
                                at: wanted,
                            }),
                            Payload::LibMedia(lib) => deferred.push(Deferred::DropLib {
                                lib,
                                track: t.id,
                                at: wanted,
                            }),
                            Payload::Text(pick) => {
                                if let Some(spec) = self.text_preset(pick) {
                                    deferred.push(Deferred::DropText {
                                        spec,
                                        track: t.id,
                                        at: wanted,
                                    });
                                }
                            }
                            Payload::Overlay(media) => deferred.push(Deferred::DropOverlay {
                                media,
                                track: t.id,
                                at: wanted,
                            }),
                            Payload::Fx(kind) => deferred.push(Deferred::DropFx {
                                kind,
                                track: t.id,
                                at: wanted,
                            }),
                            Payload::Effect(_) => {}
                        }
                    }
                }
            }
        }

        // effect drop: highlight cuts, find the nearest one
        let mut nearest_cut: Option<(Id, f32)> = None;
        if let (Some(Payload::Effect(_)), Some(pos)) = (payload.as_deref().copied(), pointer) {
            for (i, t) in tracks.iter().enumerate() {
                if t.kind != TrackKind::Video {
                    continue;
                }
                let lane = lane_rects[i];
                for c in t.clips.iter().filter(|c| c.has_left) {
                    let cx = x_of(c.start);
                    let d = if lane.expand2(vec2(60.0, 0.0)).contains(pos) {
                        (pos.x - cx).abs()
                    } else {
                        f32::MAX
                    };
                    if nearest_cut.is_none_or(|(_, bd)| d < bd) && d < 60.0 {
                        nearest_cut = Some((c.id, d));
                    }
                    let near = nearest_cut.is_some_and(|(id, _)| id == c.id);
                    let p = ui.painter();
                    let pulse = 1.0 + 0.12 * ((ctx.input(|i| i.time) * 6.0).sin() as f32);
                    p.circle_filled(
                        pos2(cx, lane.center().y),
                        16.0 * pulse,
                        if near { GOLD } else { PINK_SOFT },
                    );
                    p.circle_stroke(pos2(cx, lane.center().y), 16.0 * pulse, ink(2.0));
                    kit::draw_icon(
                        p,
                        Icon::Plus,
                        Rect::from_center_size(pos2(cx, lane.center().y), vec2(14.0, 14.0)),
                        BROWN,
                    );
                }
            }
            ctx.request_repaint();
        }

        // ---- clips
        for (i, t) in tracks.iter().enumerate() {
            let lane = lane_rects[i];
            for c in &t.clips {
                let r = Rect::from_min_max(
                    pos2(x_of(c.start), lane.top() + 5.0),
                    pos2(x_of(c.end), lane.bottom() - 5.0),
                );
                let selected =
                    self.is_selected(c.id) || self.selection == Selection::Transition(c.id);
                let body = ui.interact(r, egui::Id::new(("clip", c.id)), Sense::click_and_drag());
                let handle_l = Rect::from_min_size(r.min, vec2(HANDLE_W, r.height()));
                let handle_r = Rect::from_min_size(
                    pos2(r.right() - HANDLE_W, r.top()),
                    vec2(HANDLE_W, r.height()),
                );
                let hl = ui.interact(handle_l, egui::Id::new(("trim-l", c.id)), Sense::drag());
                let hr = ui.interact(handle_r, egui::Id::new(("trim-r", c.id)), Sense::drag());
                let hover = body.hovered() || hl.hovered() || hr.hovered();

                // paint
                let color = if t.kind == TrackKind::Audio {
                    AUDIO_COLOR
                } else if c.fx.is_some() {
                    Color32::from_rgb(255, 206, 190)
                } else if c.is_text {
                    Color32::from_rgb(240, 214, 250)
                } else if c.is_overlay {
                    Color32::from_rgb(190, 236, 214)
                } else {
                    CLIP_COLORS[c.color as usize % CLIP_COLORS.len()]
                };
                {
                    let p = ui.painter();
                    if selected {
                        p.rect_stroke(
                            r.expand(3.0),
                            round(14),
                            Stroke::new(4.0, PINK_DEEP),
                            StrokeKind::Middle,
                        );
                    }
                    p.rect_filled(
                        r,
                        round(11),
                        if hover {
                            color.gamma_multiply(1.05)
                        } else {
                            color
                        },
                    );
                    p.rect_stroke(
                        r,
                        round(11),
                        ink(if selected { 2.5 } else { 1.6 }),
                        StrokeKind::Inside,
                    );
                }
                let inner = r.shrink2(vec2(6.0, 5.0));
                let mut text_x = inner.left() + 4.0;
                if c.is_text || c.fx.is_some() {
                    kit::draw_icon(
                        ui.painter(),
                        if c.fx.is_some() {
                            Icon::Bolt
                        } else {
                            Icon::Text
                        },
                        Rect::from_min_size(
                            pos2(inner.left() + 2.0, inner.top() + 2.0),
                            vec2(20.0, 20.0),
                        ),
                        BROWN,
                    );
                    text_x = inner.left() + 28.0;
                } else if c.kind != MediaKind::Audio {
                    let tw = (inner.height() * 16.0 / 9.0).min(inner.width() * 0.5);
                    if tw > 24.0 {
                        let tr = Rect::from_min_size(inner.min, vec2(tw, inner.height()));
                        match self.engine.thumb(c.media) {
                            Some(tex) => egui::Image::new(tex)
                                .corner_radius(round(7))
                                .paint_at(ui, tr),
                            None => {
                                ui.painter().rect_filled(
                                    tr,
                                    round(7),
                                    Color32::from_white_alpha(120),
                                );
                            }
                        }
                        text_x = tr.right() + 7.0;
                    }
                } else {
                    kit::draw_icon(
                        ui.painter(),
                        Icon::Note,
                        Rect::from_min_size(
                            pos2(inner.left() + 2.0, inner.top() + 2.0),
                            vec2(20.0, 20.0),
                        ),
                        BROWN,
                    );
                    text_x = inner.left() + 28.0;
                }
                if let Some(pcm) = self.audio.pcm(c.media) {
                    let area = if c.kind == MediaKind::Audio {
                        Rect::from_min_max(
                            pos2(r.left() + 6.0, r.top() + 30.0),
                            pos2(r.right() - 6.0, r.bottom() - 6.0),
                        )
                    } else {
                        Rect::from_min_max(
                            pos2(r.left() + 6.0, r.bottom() - 15.0),
                            pos2(r.right() - 6.0, r.bottom() - 5.0),
                        )
                    };
                    draw_waveform(
                        ui.painter(),
                        area,
                        ui.clip_rect(),
                        pcm,
                        c.source_in,
                        pps,
                        r.left(),
                    );
                }
                if c.is_overlay && r.width() > 50.0 {
                    kit::draw_icon(
                        ui.painter(),
                        Icon::Image,
                        Rect::from_center_size(
                            pos2(r.right() - 16.0, r.top() + 14.0),
                            vec2(16.0, 16.0),
                        ),
                        BROWN,
                    );
                }
                let clipped = ui.painter().with_clip_rect(r.shrink(3.0));
                if r.width() > 60.0 {
                    clipped.text(
                        pos2(text_x, inner.top() + 10.0),
                        Align2::LEFT_CENTER,
                        &c.name,
                        egui::FontId::proportional(13.5),
                        BROWN,
                    );
                    clipped.text(
                        pos2(text_x, inner.top() + 28.0),
                        Align2::LEFT_CENTER,
                        format!("{:.1} s", (c.end - c.start).secs()),
                        egui::FontId::proportional(11.5),
                        BROWN_SOFT,
                    );
                }
                if hover || selected {
                    let p = ui.painter();
                    for h in [handle_l, handle_r] {
                        p.rect_filled(
                            Rect::from_center_size(
                                h.center(),
                                vec2(4.0, (r.height() * 0.5).min(26.0)),
                            ),
                            round(2),
                            BROWN,
                        );
                    }
                }
                if hl.hovered() || hr.hovered() {
                    ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
                } else if body.hovered() {
                    ctx.set_cursor_icon(if body.dragged() {
                        CursorIcon::Grabbing
                    } else {
                        CursorIcon::Grab
                    });
                }

                // select / move
                if body.clicked() {
                    deferred.push(if modifier {
                        Deferred::Toggle(c.id)
                    } else {
                        Deferred::Select(Selection::Clip(c.id))
                    });
                }
                if body.drag_started() {
                    let group = self.is_selected(c.id) && self.selected_clips().len() > 1;
                    if !group {
                        deferred.push(Deferred::Select(Selection::Clip(c.id)));
                    }
                    let grab = body
                        .interact_pointer_pos()
                        .map_or(Tick::ZERO, |p| t_of(p.x) - c.start);
                    self.doc.begin_gesture();
                    if group {
                        self.group_origins = self
                            .selected_clips()
                            .into_iter()
                            .filter_map(|id| {
                                self.doc.project.clip(id).map(|(_, cl)| (id, cl.start))
                            })
                            .collect();
                        self.gesture = Gesture::MoveGroup {
                            primary: c.id,
                            grab,
                        };
                    } else {
                        self.gesture = Gesture::Move { clip: c.id, grab };
                    }
                }
                if body.dragged()
                    && let (Gesture::MoveGroup { primary, grab }, Some(pos)) =
                        (self.gesture, pointer)
                    && primary == c.id
                {
                    let dur = c.end - c.start;
                    let desired =
                        self.snap_span(t_of(pos.x) - grab, dur, &snap_edges, c.start, c.end, pps);
                    let origin = self
                        .group_origins
                        .iter()
                        .find(|(id, _)| *id == primary)
                        .map_or(c.start, |(_, s)| *s);
                    let origins = self.group_origins.clone();
                    self.doc.project.shift_clips(&origins, desired - origin);
                }
                if body.dragged()
                    && let (Gesture::Move { clip, grab }, Some(pos)) = (self.gesture, pointer)
                    && clip == c.id
                {
                    let dur = c.end - c.start;
                    let desired =
                        self.snap_span(t_of(pos.x) - grab, dur, &snap_edges, c.start, c.end, pps);
                    let target = lane_rects
                        .iter()
                        .position(|l| {
                            pos.y >= l.top() - LANE_GAP / 2.0
                                && pos.y <= l.bottom() + LANE_GAP / 2.0
                        })
                        .map(|k| &tracks[k])
                        .filter(|tt| tt.kind == t.kind)
                        .map_or(t.id, |tt| tt.id);
                    self.doc
                        .project
                        .move_clip(c.id, target, desired.max(Tick::ZERO))
                        .ok();
                }
                if body.drag_stopped()
                    && matches!(
                        self.gesture,
                        Gesture::Move { .. } | Gesture::MoveGroup { .. }
                    )
                {
                    self.doc.end_gesture();
                    self.gesture = Gesture::None;
                }

                // trim
                if hl.drag_started() {
                    self.trim_grab = hl
                        .interact_pointer_pos()
                        .map_or(Tick::ZERO, |p| c.start - t_of(p.x));
                    self.doc.begin_gesture();
                    self.gesture = Gesture::TrimStart(c.id);
                    deferred.push(Deferred::Select(Selection::Clip(c.id)));
                }
                if hl.dragged()
                    && self.gesture == Gesture::TrimStart(c.id)
                    && let Some(pos) = pointer
                {
                    let at = self.snap_point(
                        t_of(pos.x) + self.trim_grab,
                        &snap_edges,
                        &[c.start, c.end],
                        pps,
                    );
                    self.doc.project.trim_start(c.id, at).ok();
                }
                if hl.drag_stopped() && matches!(self.gesture, Gesture::TrimStart(_)) {
                    self.doc.end_gesture();
                    self.gesture = Gesture::None;
                }
                if hr.drag_started() {
                    self.trim_grab = hr
                        .interact_pointer_pos()
                        .map_or(Tick::ZERO, |p| c.end - t_of(p.x));
                    self.doc.begin_gesture();
                    self.gesture = Gesture::TrimEnd(c.id);
                    deferred.push(Deferred::Select(Selection::Clip(c.id)));
                }
                if hr.dragged()
                    && self.gesture == Gesture::TrimEnd(c.id)
                    && let Some(pos) = pointer
                {
                    let at = self.snap_point(
                        t_of(pos.x) + self.trim_grab,
                        &snap_edges,
                        &[c.start, c.end],
                        pps,
                    );
                    self.doc.project.trim_end(c.id, at).ok();
                }
                if hr.drag_stopped() && matches!(self.gesture, Gesture::TrimEnd(_)) {
                    self.doc.end_gesture();
                    self.gesture = Gesture::None;
                }

                // context menu
                body.context_menu(|ui| {
                    if ui.button(tr("Am Zeiger teilen")).clicked() {
                        deferred.push(Deferred::Split(c.id, self.playhead));
                        ui.close();
                    }
                    if ui.button(tr("Löschen")).clicked() {
                        deferred.push(Deferred::Delete(c.id, false));
                        ui.close();
                    }
                    if ui.button(tr("Löschen und Lücke schließen")).clicked() {
                        deferred.push(Deferred::Delete(c.id, true));
                        ui.close();
                    }
                    if c.transition.is_some() && ui.button(tr("Blende entfernen")).clicked() {
                        deferred.push(Deferred::ClearTransition(c.id));
                        ui.close();
                    }
                    if c.is_image {
                        let label = if c.is_overlay {
                            tr("Als Vollbild-Bild verwenden")
                        } else {
                            tr("Als Grafik über das Video legen")
                        };
                        if ui.button(label).clicked() {
                            deferred.push(Deferred::ToggleOverlay(c.id));
                            ui.close();
                        }
                    }
                    if c.is_text {
                        ui.separator();
                        let n_targets = if self.is_selected(c.id) {
                            self.selected_clips().len()
                        } else {
                            1
                        };
                        let title = if n_targets > 1 {
                            trf!("Vorlage auf {n} Clips anwenden", n = n_targets)
                        } else {
                            tr("Vorlage anwenden").to_string()
                        };
                        ui.menu_button(title, |ui| {
                            for (i, (name, _)) in crate::presets::builtin().into_iter().enumerate()
                            {
                                if ui.button(name).clicked() {
                                    deferred.push(Deferred::ApplyPreset(
                                        crate::app::TextPick {
                                            global: false,
                                            index: i,
                                        },
                                        c.id,
                                    ));
                                    ui.close();
                                }
                            }
                            if !self.lib.presets.is_empty() {
                                ui.separator();
                                for (i, p) in self.lib.presets.iter().enumerate() {
                                    if ui.button(&p.name).clicked() {
                                        deferred.push(Deferred::ApplyPreset(
                                            crate::app::TextPick {
                                                global: true,
                                                index: i,
                                            },
                                            c.id,
                                        ));
                                        ui.close();
                                    }
                                }
                            }
                        });
                    }
                });
            }
        }

        // ---- transition badges (above the clips)
        for (i, t) in tracks.iter().enumerate() {
            let lane = lane_rects[i];
            for c in &t.clips {
                let Some(tr) = c.transition else { continue };
                let cx = x_of(c.start);
                let half = (tr.duration.secs() * pps / 2.0) as f32;
                let band = Rect::from_min_max(
                    pos2(cx - half, lane.top() + 3.0),
                    pos2(cx + half, lane.bottom() - 3.0),
                );
                let selected = self.selection == Selection::Transition(c.id);
                let p = ui.painter();
                p.rect_filled(band, round(12), PINK_DEEP.gamma_multiply(0.55));
                p.rect_stroke(
                    band,
                    round(12),
                    Stroke::new(
                        if selected { 2.5 } else { 1.5 },
                        if selected { BROWN } else { PINK_DEEP },
                    ),
                    StrokeKind::Inside,
                );
                // drag either edge of the band to change the length of the blend
                for (n, ex) in [cx - half, cx + half].into_iter().enumerate() {
                    let edge = Rect::from_center_size(
                        pos2(ex, lane.center().y),
                        vec2(12.0, lane.height() - 8.0),
                    );
                    let er =
                        ui.interact(edge, egui::Id::new(("band-edge", c.id, n)), Sense::drag());
                    if er.hovered() || er.dragged() {
                        ctx.set_cursor_icon(CursorIcon::ResizeHorizontal);
                        ui.painter().rect_filled(
                            Rect::from_center_size(edge.center(), vec2(4.0, 24.0)),
                            round(2),
                            BROWN,
                        );
                    }
                    if er.drag_started() {
                        self.doc.begin_gesture();
                    }
                    if er.dragged()
                        && let Some(pos) = pointer
                    {
                        let secs = ((pos.x - cx).abs() as f64 / pps * 2.0).max(0.1);
                        self.doc
                            .project
                            .set_transition(
                                c.id,
                                Some(Transition {
                                    kind: tr.kind,
                                    duration: Tick::from_secs(secs),
                                }),
                            )
                            .ok();
                    }
                    if er.drag_stopped() {
                        self.doc.end_gesture();
                    }
                }
                let badge = Rect::from_center_size(pos2(cx, lane.center().y), vec2(34.0, 34.0));
                let resp = ui.interact(badge, egui::Id::new(("badge", c.id)), Sense::click());
                let p = ui.painter();
                p.circle_filled(badge.center(), 17.0, if selected { GOLD } else { PINK });
                p.circle_stroke(badge.center(), 17.0, ink(2.0));
                kit::paw(p, badge.center() + vec2(0.0, 1.0), 9.0, BROWN);
                if resp
                    .on_hover_text(format!(
                        "{} · {:.1} s",
                        crate::i18n::tr(tr.kind.name()),
                        tr.duration.secs()
                    ))
                    .clicked()
                {
                    deferred.push(Deferred::Select(Selection::Transition(c.id)));
                }
            }
        }

        if let Some((id, _)) = nearest_cut
            && let Some(Payload::Effect(kind)) = payload.as_deref().copied()
            && ctx.input(|i| i.pointer.any_released())
        {
            deferred.push(Deferred::DropEffect {
                kind,
                right_clip: id,
            });
        }

        // ---- rubber band selection
        if let Some(m) = self.marquee.clone() {
            let down = ctx.input(|i| i.pointer.primary_down());
            match (down, pointer) {
                (true, Some(pos)) => {
                    let band = Rect::from_two_pos(m.start, pos);
                    let mut hits = Vec::new();
                    for (i, t) in tracks.iter().enumerate() {
                        let lane = lane_rects[i];
                        for c in &t.clips {
                            let r = Rect::from_min_max(
                                pos2(x_of(c.start), lane.top() + 5.0),
                                pos2(x_of(c.end), lane.bottom() - 5.0),
                            );
                            if r.intersects(band) {
                                hits.push(c.id);
                            }
                        }
                    }
                    deferred.push(Deferred::SetMulti(hits, m.base.clone()));
                    let p = ui.painter();
                    p.rect_filled(band, round(6), PINK.gamma_multiply(0.30));
                    p.rect_stroke(
                        band,
                        round(6),
                        Stroke::new(1.8, PINK_DEEP),
                        StrokeKind::Inside,
                    );
                    ctx.request_repaint();
                }
                _ => self.marquee = None,
            }
        }

        // ---- playhead
        let px = x_of(self.playhead);
        let p = ui.painter();
        p.line_segment(
            [pos2(px, ruler.top() + 8.0), pos2(px, rect.bottom())],
            Stroke::new(2.4, BROWN),
        );
        p.circle_filled(pos2(px, ruler.top() + 12.0), 11.0, PINK_DEEP);
        p.circle_stroke(pos2(px, ruler.top() + 12.0), 11.0, ink(2.0));
        kit::paw(p, pos2(px, ruler.top() + 13.0), 6.0, CREAM);
        if self.playing
            && !ui
                .clip_rect()
                .expand2(vec2(-30.0, 0.0))
                .contains(pos2(px, ui.clip_rect().center().y))
        {
            ui.scroll_to_rect(
                Rect::from_center_size(pos2(px, rect.center().y), vec2(240.0, 10.0)),
                Some(egui::Align::Center),
            );
        }
        let _ = fps;
    }

    /// Snap a clip's start so its start or end meets a nearby edge.
    fn snap_span(
        &self,
        start: Tick,
        dur: Tick,
        edges: &[Tick],
        own_start: Tick,
        own_end: Tick,
        pps: f64,
    ) -> Tick {
        let fps = self.doc.project.fps;
        if !self.snap {
            return start.snap_to_frame(fps);
        }
        let limit = Tick::from_secs(SNAP_PX as f64 / pps);
        let mut best: Option<(Tick, Tick)> = None; // (distance, snapped start)
        for e in edges.iter().filter(|e| **e != own_start && **e != own_end) {
            for (d, s) in [
                ((start - *e).0.abs(), *e),
                (((start + dur) - *e).0.abs(), *e - dur),
            ] {
                if Tick(d) <= limit && best.is_none_or(|(bd, _)| Tick(d) < bd) {
                    best = Some((Tick(d), s));
                }
            }
        }
        best.map_or_else(|| start.snap_to_frame(fps), |(_, s)| s)
    }

    fn snap_point(&self, at: Tick, edges: &[Tick], own: &[Tick], pps: f64) -> Tick {
        let fps = self.doc.project.fps;
        if !self.snap {
            return at.snap_to_frame(fps);
        }
        let limit = Tick::from_secs(SNAP_PX as f64 / pps);
        edges
            .iter()
            .filter(|e| !own.contains(e))
            .filter(|e| Tick((at - **e).0.abs()) <= limit)
            .min_by_key(|e| (at - **e).0.abs())
            .copied()
            .unwrap_or_else(|| at.snap_to_frame(fps))
    }

    fn run_deferred(&mut self, actions: Vec<Deferred>) {
        for a in actions {
            match a {
                Deferred::Select(s) => {
                    if let Selection::Clip(id) = s
                        && let Some((_, c)) = self.doc.project.clip(id)
                    {
                        if c.is_text() {
                            self.tab = crate::app::PoolTab::Text;
                        } else if c.is_fx() {
                            self.tab = crate::app::PoolTab::Fx;
                        } else if c.is_overlay() {
                            self.tab = crate::app::PoolTab::Graphics;
                        }
                    }
                    self.set_selection(s);
                }
                Deferred::DropFx { kind, track, at } => {
                    let spec = hk_core::FxSpec::new(kind);
                    match self
                        .doc
                        .edit(|p| p.add_fx_clip(track, at, Tick::from_secs(5.0), spec))
                    {
                        Ok(id) => {
                            self.set_selection(Selection::Clip(id));
                            self.tab = crate::app::PoolTab::Fx;
                        }
                        Err(e) => self.say(e.to_string()),
                    }
                }
                Deferred::DropOverlay { media, track, at } => {
                    match self.doc.edit(|p| p.add_overlay_clip(track, media, at)) {
                        Ok(id) => {
                            self.set_selection(Selection::Clip(id));
                            self.tab = crate::app::PoolTab::Graphics;
                        }
                        Err(e) => self.say(e.to_string()),
                    }
                }
                Deferred::ToggleOverlay(id) => {
                    let on = self
                        .doc
                        .project
                        .clip(id)
                        .is_some_and(|(_, c)| c.is_overlay());
                    let res = self
                        .doc
                        .edit(|p| p.set_overlay(id, (!on).then(hk_core::OverlaySpec::default)));
                    match res {
                        Ok(()) if !on => {
                            self.set_selection(Selection::Clip(id));
                            self.tab = crate::app::PoolTab::Graphics;
                        }
                        Ok(()) => {}
                        Err(e) => self.say(e.to_string()),
                    }
                }
                Deferred::ApplyPreset(pick, clip) => {
                    let targets = if self.is_selected(clip) {
                        self.selected_clips()
                    } else {
                        vec![clip]
                    };
                    self.apply_text_preset(pick, &targets);
                }
                Deferred::DropText { spec, track, at } => {
                    match self
                        .doc
                        .edit(|p| p.add_text_clip(track, at, Tick::from_secs(4.0), spec))
                    {
                        Ok(id) => {
                            self.set_selection(Selection::Clip(id));
                            self.tab = crate::app::PoolTab::Text;
                        }
                        Err(e) => self.say(e.to_string()),
                    }
                }
                Deferred::DropLib { lib, track, at } => {
                    if let Some(media) = self.use_lib_item(lib) {
                        match self.doc.edit(|p| p.add_clip(track, media, at)) {
                            Ok(id) => self.set_selection(Selection::Clip(id)),
                            Err(e) => self.say(e.to_string()),
                        }
                    }
                }
                Deferred::RemoveTrack(id) => {
                    if let Err(e) = self.doc.edit(|p| p.remove_track(id)) {
                        self.say(e.to_string());
                    }
                }
                Deferred::Split(id, at) => {
                    let ok = self.doc.edit(|p| p.split_clip(id, at).is_ok());
                    if !ok {
                        self.say("Setze den Zeiger auf den Clip, um ihn zu teilen");
                    }
                }
                Deferred::Toggle(id) => self.toggle_selected(id),
                Deferred::SetMulti(ids, base) => {
                    let mut all = base;
                    all.extend(ids);
                    self.select_many(all);
                    let sel = self.selected_clips();
                    if !sel.is_empty()
                        && sel
                            .iter()
                            .all(|id| self.doc.project.clip(*id).is_some_and(|(_, c)| c.is_text()))
                    {
                        self.tab = crate::app::PoolTab::Text;
                    }
                }
                Deferred::Delete(id, true)
                    if self.is_selected(id) && self.selected_clips().len() > 1 =>
                {
                    self.selection = Selection::Clip(id);
                    self.delete_selection(true);
                }
                Deferred::Delete(id, false)
                    if self.is_selected(id) && self.selected_clips().len() > 1 =>
                {
                    self.selection = Selection::Clip(id);
                    self.delete_selection(false);
                }
                Deferred::Delete(id, ripple) => {
                    self.doc.edit(|p| p.delete_clip(id, ripple).ok());
                    if self.selection == Selection::Clip(id) {
                        self.set_selection(Selection::None);
                    }
                }
                Deferred::ClearTransition(id) => {
                    self.doc.edit(|p| p.set_transition(id, None).ok());
                }
                Deferred::DropMedia { media, track, at } => {
                    match self.doc.edit(|p| p.add_clip(track, media, at)) {
                        Ok(id) => self.set_selection(Selection::Clip(id)),
                        Err(e) => self.say(e.to_string()),
                    }
                }
                Deferred::DropEffect { kind, right_clip } => self.apply_effect(kind, right_clip),
                Deferred::Playhead(t) => self.set_playhead(t),
                Deferred::ToggleMute(id) => {
                    self.doc.edit(|p| {
                        if let Some(t) = p.tracks.iter_mut().find(|t| t.id == id) {
                            t.muted = !t.muted;
                        }
                    });
                }
            }
        }
    }
}

/// Peak bars for the visible part of a clip.
fn draw_waveform(
    p: &egui::Painter,
    area: Rect,
    visible: Rect,
    pcm: &hk_audio::Pcm,
    source_in: Tick,
    pps: f64,
    clip_left: f32,
) {
    let p = p.with_clip_rect(area.intersect(visible));
    let mid = area.center().y;
    let half = area.height() / 2.0;
    let blocks_per_sec = pcm.rate as f64 / hk_audio::PEAK_BLOCK as f64;
    let x0 = area.left().max(visible.left());
    let x1 = area.right().min(visible.right());
    let mut x = (x0 / 2.0).floor() * 2.0;
    while x < x1 {
        let t0 = source_in.secs() + (x - clip_left) as f64 / pps;
        let t1 = t0 + 2.0 / pps;
        let (i0, i1) = (
            (t0 * blocks_per_sec).floor().max(0.0) as usize,
            (t1 * blocks_per_sec).ceil().max(0.0) as usize,
        );
        let peak = pcm
            .peaks
            .get(i0..i1.max(i0 + 1).min(pcm.peaks.len()))
            .map_or(0.0, |s| s.iter().fold(0.0f32, |m, v| m.max(*v)));
        if peak > 0.0 {
            let h = (peak.min(1.0) * half).max(1.0);
            p.line_segment(
                [pos2(x + 1.0, mid - h), pos2(x + 1.0, mid + h)],
                Stroke::new(1.6, BROWN_SOFT.gamma_multiply(0.85)),
            );
        }
        x += 2.0;
    }
}

impl HappyKitten {
    fn add_track(&mut self, kind: TrackKind) {
        if let Err(e) = self.doc.edit(|p| p.add_track_of(kind)) {
            self.say(e.to_string());
        }
    }
}
