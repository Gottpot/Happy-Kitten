//! Hand-drawn widgets and icons. Everything is painted from shapes so icons stay
//! crisp at any scale and no icon font or image assets are needed.

use crate::theme::*;
use egui::epaint::EllipseShape;
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Paw,
    Scissors,
    Trash,
    Undo,
    Redo,
    Folder,
    Sparkle,
    Note,
    Plus,
    Magnet,
    Play,
    Pause,
    SkipStart,
    SkipEnd,
    StepBack,
    StepForward,
    Save,
    New,
    Export,
    Speaker,
    SpeakerOff,
    Film,
    Text,
    Globe,
    Check,
    Image,
    Grid,
    List,
    Table,
    Sun,
    Bolt,
}

fn ellipse(p: &egui::Painter, c: Pos2, r: Vec2, angle: f32, fill: Color32) {
    p.add(Shape::Ellipse(EllipseShape {
        center: c,
        radius: r,
        fill,
        stroke: Stroke::NONE,
        angle,
    }));
}

/// A paw print: pad and four toes, toes point up. `r` is the half width.
pub fn paw(p: &egui::Painter, c: Pos2, r: f32, color: Color32) {
    let u = r / 0.8;
    ellipse(
        p,
        c + vec2(0.0, 0.30 * u),
        vec2(0.48 * u, 0.38 * u),
        0.0,
        color,
    );
    ellipse(
        p,
        c + vec2(-0.27 * u, 0.40 * u),
        vec2(0.27 * u, 0.27 * u),
        0.0,
        color,
    );
    ellipse(
        p,
        c + vec2(0.27 * u, 0.40 * u),
        vec2(0.27 * u, 0.27 * u),
        0.0,
        color,
    );
    ellipse(
        p,
        c + vec2(-0.66 * u, -0.06 * u),
        vec2(0.17 * u, 0.23 * u),
        -0.45,
        color,
    );
    ellipse(
        p,
        c + vec2(-0.23 * u, -0.50 * u),
        vec2(0.18 * u, 0.25 * u),
        -0.15,
        color,
    );
    ellipse(
        p,
        c + vec2(0.23 * u, -0.50 * u),
        vec2(0.18 * u, 0.25 * u),
        0.15,
        color,
    );
    ellipse(
        p,
        c + vec2(0.66 * u, -0.06 * u),
        vec2(0.17 * u, 0.23 * u),
        0.45,
        color,
    );
}

/// Ball of yarn with a loose thread.
pub fn yarn_ball(p: &egui::Painter, c: Pos2, r: f32) {
    p.circle_filled(c, r, YARN);
    p.circle_stroke(c, r, ink(r * 0.07));
    let light = Stroke::new(r * 0.09, Color32::from_rgb(255, 214, 226));
    for k in -1..=1 {
        let off = k as f32 * r * 0.38;
        let pts: Vec<Pos2> = (0..=14)
            .map(|i| {
                let t = i as f32 / 14.0;
                let x = -r * 0.85 + t * r * 1.7;
                let y = off + (t * std::f32::consts::PI).sin() * r * 0.28 - r * 0.1;
                pos2(c.x + x, c.y + y)
            })
            .filter(|q| (*q - c).length() < r * 0.9)
            .collect();
        p.add(Shape::line(pts, light));
    }
    let tail: Vec<Pos2> = (0..=16)
        .map(|i| {
            let t = i as f32 / 16.0;
            pos2(
                c.x + r * (0.6 + 1.1 * t),
                c.y + r * (0.7 + 0.25 * (t * 6.0).sin() * (1.0 - t * 0.4)),
            )
        })
        .collect();
    p.add(Shape::line(tail, Stroke::new(r * 0.12, YARN)));
}

fn poly(p: &egui::Painter, pts: Vec<Pos2>, fill: Color32) {
    p.add(Shape::convex_polygon(pts, fill, Stroke::NONE));
}

/// Draw `icon` filling `rect` (square, with a little inner margin).
pub fn draw_icon(p: &egui::Painter, icon: Icon, rect: Rect, color: Color32) {
    let c = rect.center();
    let s = rect.width().min(rect.height()) * 0.5;
    let line = Stroke::new((s * 0.2).max(1.4), color);
    let at = |x: f32, y: f32| pos2(c.x + x * s, c.y + y * s);
    match icon {
        Icon::Paw => paw(p, c, s * 0.9, color),
        Icon::Play => poly(
            p,
            vec![at(-0.45, -0.7), at(0.7, 0.0), at(-0.45, 0.7)],
            color,
        ),
        Icon::Pause => {
            p.rect_filled(
                Rect::from_center_size(at(-0.35, 0.0), vec2(s * 0.4, s * 1.4)),
                round(2),
                color,
            );
            p.rect_filled(
                Rect::from_center_size(at(0.35, 0.0), vec2(s * 0.4, s * 1.4)),
                round(2),
                color,
            );
        }
        Icon::SkipStart => {
            p.rect_filled(
                Rect::from_center_size(at(-0.6, 0.0), vec2(s * 0.25, s * 1.4)),
                round(2),
                color,
            );
            poly(
                p,
                vec![at(0.65, -0.65), at(-0.3, 0.0), at(0.65, 0.65)],
                color,
            );
        }
        Icon::SkipEnd => {
            p.rect_filled(
                Rect::from_center_size(at(0.6, 0.0), vec2(s * 0.25, s * 1.4)),
                round(2),
                color,
            );
            poly(
                p,
                vec![at(-0.65, -0.65), at(0.3, 0.0), at(-0.65, 0.65)],
                color,
            );
        }
        Icon::StepBack => {
            poly(p, vec![at(0.1, -0.65), at(-0.7, 0.0), at(0.1, 0.65)], color);
            poly(p, vec![at(0.8, -0.65), at(0.0, 0.0), at(0.8, 0.65)], color);
        }
        Icon::StepForward => {
            poly(
                p,
                vec![at(-0.1, -0.65), at(0.7, 0.0), at(-0.1, 0.65)],
                color,
            );
            poly(
                p,
                vec![at(-0.8, -0.65), at(0.0, 0.0), at(-0.8, 0.65)],
                color,
            );
        }
        Icon::Plus => {
            p.line_segment([at(-0.7, 0.0), at(0.7, 0.0)], line);
            p.line_segment([at(0.0, -0.7), at(0.0, 0.7)], line);
        }
        Icon::Scissors => {
            p.circle_stroke(at(-0.45, 0.6), s * 0.3, line);
            p.circle_stroke(at(0.45, 0.6), s * 0.3, line);
            p.line_segment([at(-0.3, 0.35), at(0.6, -0.8)], line);
            p.line_segment([at(0.3, 0.35), at(-0.6, -0.8)], line);
        }
        Icon::Trash => {
            p.line_segment([at(-0.75, -0.55), at(0.75, -0.55)], line);
            p.line_segment([at(-0.25, -0.55), at(-0.25, -0.8)], line);
            p.line_segment([at(0.25, -0.55), at(0.25, -0.8)], line);
            p.line_segment([at(-0.25, -0.8), at(0.25, -0.8)], line);
            p.add(Shape::closed_line(
                vec![
                    at(-0.55, -0.4),
                    at(0.55, -0.4),
                    at(0.45, 0.85),
                    at(-0.45, 0.85),
                ],
                line,
            ));
            p.line_segment(
                [at(-0.15, -0.1), at(-0.15, 0.6)],
                Stroke::new(line.width * 0.7, color),
            );
            p.line_segment(
                [at(0.15, -0.1), at(0.15, 0.6)],
                Stroke::new(line.width * 0.7, color),
            );
        }
        Icon::Undo | Icon::Redo => {
            let flip = if icon == Icon::Redo { -1.0 } else { 1.0 };
            let pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = -0.2 + i as f32 / 12.0 * 3.3;
                    at(
                        flip * (0.1 + a.cos() * 0.6),
                        0.05 - a.sin() * 0.55 * 0.9 + 0.2,
                    )
                })
                .collect();
            p.add(Shape::line(pts, line));
            poly(
                p,
                vec![
                    at(flip * -0.75, -0.55),
                    at(flip * -0.75, 0.15),
                    at(flip * -0.05, -0.2),
                ],
                color,
            );
        }
        Icon::Folder => {
            p.add(Shape::closed_line(
                vec![
                    at(-0.85, -0.55),
                    at(-0.25, -0.55),
                    at(0.0, -0.3),
                    at(0.85, -0.3),
                    at(0.85, 0.6),
                    at(-0.85, 0.6),
                ],
                line,
            ));
        }
        Icon::Sparkle => {
            let star = |cx: f32, cy: f32, r: f32| {
                let pts: Vec<Pos2> = (0..8)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::FRAC_PI_4;
                        let rr = if i % 2 == 0 { r } else { r * 0.3 };
                        at(cx + a.sin() * rr, cy - a.cos() * rr)
                    })
                    .collect();
                poly(p, pts, color);
            };
            star(-0.2, 0.1, 0.85);
            star(0.6, -0.55, 0.38);
            star(0.55, 0.65, 0.3);
        }
        Icon::Note => {
            p.circle_filled(at(-0.35, 0.55), s * 0.34, color);
            p.circle_filled(at(0.55, 0.35), s * 0.34, color);
            p.line_segment([at(-0.05, 0.55), at(-0.05, -0.7)], line);
            p.line_segment([at(0.85, 0.35), at(0.85, -0.85)], line);
            p.line_segment(
                [at(-0.05, -0.7), at(0.85, -0.85)],
                Stroke::new(line.width * 1.5, color),
            );
        }
        Icon::Magnet => {
            let pts: Vec<Pos2> = (0..=12)
                .map(|i| {
                    let a = std::f32::consts::PI + i as f32 / 12.0 * std::f32::consts::PI;
                    at(a.cos() * 0.65, -0.1 + -(a.sin() * 0.7) * -1.0)
                })
                .collect();
            p.add(Shape::line(pts, Stroke::new(line.width * 1.6, color)));
            p.line_segment(
                [at(-0.65, -0.1), at(-0.65, 0.65)],
                Stroke::new(line.width * 1.6, color),
            );
            p.line_segment(
                [at(0.65, -0.1), at(0.65, 0.65)],
                Stroke::new(line.width * 1.6, color),
            );
        }
        Icon::Save => {
            p.add(Shape::closed_line(
                vec![
                    at(-0.8, -0.8),
                    at(0.55, -0.8),
                    at(0.8, -0.55),
                    at(0.8, 0.8),
                    at(-0.8, 0.8),
                ],
                line,
            ));
            p.rect_stroke(
                Rect::from_center_size(at(0.0, -0.45), vec2(s * 1.0, s * 0.5)),
                round(1),
                line,
                StrokeKind::Middle,
            );
            p.rect_stroke(
                Rect::from_center_size(at(0.0, 0.4), vec2(s * 1.1, s * 0.6)),
                round(2),
                line,
                StrokeKind::Middle,
            );
        }
        Icon::New => {
            p.add(Shape::closed_line(
                vec![
                    at(-0.6, -0.85),
                    at(0.2, -0.85),
                    at(0.65, -0.4),
                    at(0.65, 0.85),
                    at(-0.6, 0.85),
                ],
                line,
            ));
            p.line_segment([at(0.2, -0.85), at(0.2, -0.4)], line);
            p.line_segment([at(0.2, -0.4), at(0.65, -0.4)], line);
        }
        Icon::Export => {
            p.line_segment([at(0.0, 0.5), at(0.0, -0.8)], line);
            poly(
                p,
                vec![at(-0.5, -0.4), at(0.0, -0.95), at(0.5, -0.4)],
                color,
            );
            p.add(Shape::line(
                vec![at(-0.8, 0.1), at(-0.8, 0.85), at(0.8, 0.85), at(0.8, 0.1)],
                line,
            ));
        }
        Icon::Speaker | Icon::SpeakerOff => {
            poly(
                p,
                vec![
                    at(-0.8, -0.3),
                    at(-0.4, -0.3),
                    at(0.1, -0.75),
                    at(0.1, 0.75),
                    at(-0.4, 0.3),
                    at(-0.8, 0.3),
                ],
                color,
            );
            if icon == Icon::Speaker {
                let arc: Vec<Pos2> = (0..=8)
                    .map(|i| {
                        let a = -0.9 + i as f32 / 8.0 * 1.8;
                        at(0.2 + a.cos() * 0.5, a.sin() * 0.6)
                    })
                    .collect();
                p.add(Shape::line(arc, line));
            } else {
                p.line_segment([at(0.35, -0.35), at(0.95, 0.35)], line);
                p.line_segment([at(0.35, 0.35), at(0.95, -0.35)], line);
            }
        }
        Icon::Image => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(s * 1.7, s * 1.45)),
                round(3),
                line,
                StrokeKind::Middle,
            );
            p.circle_filled(at(-0.4, -0.3), s * 0.17, color);
            p.add(Shape::line(
                vec![
                    at(-0.85, 0.55),
                    at(-0.2, -0.05),
                    at(0.2, 0.35),
                    at(0.45, 0.1),
                    at(0.85, 0.55),
                ],
                Stroke::new(line.width * 0.8, color),
            ));
        }
        Icon::Check => {
            p.add(Shape::line(
                vec![at(-0.7, 0.05), at(-0.2, 0.6), at(0.75, -0.55)],
                Stroke::new(line.width * 1.3, color),
            ));
        }
        Icon::Grid => {
            for (x, y) in [(-0.45, -0.45), (0.45, -0.45), (-0.45, 0.45), (0.45, 0.45)] {
                p.rect_stroke(
                    Rect::from_center_size(at(x, y), vec2(s * 0.7, s * 0.7)),
                    round(2),
                    line,
                    StrokeKind::Middle,
                );
            }
        }
        Icon::List => {
            for y in [-0.6, 0.0, 0.6] {
                p.circle_filled(at(-0.7, y), s * 0.12, color);
                p.line_segment([at(-0.35, y), at(0.8, y)], line);
            }
        }
        Icon::Bolt => {
            p.add(Shape::convex_polygon(
                vec![
                    at(0.25, -0.95),
                    at(-0.55, 0.10),
                    at(-0.05, 0.10),
                    at(-0.30, 0.95),
                    at(0.60, -0.25),
                    at(0.08, -0.25),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Sun => {
            p.circle_filled(c, s * 0.38, color);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let (sn, cs) = a.sin_cos();
                p.line_segment([at(cs * 0.62, sn * 0.62), at(cs * 0.92, sn * 0.92)], line);
            }
        }
        Icon::Table => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(s * 1.7, s * 1.5)),
                round(2),
                line,
                StrokeKind::Middle,
            );
            p.line_segment(
                [at(-0.85, -0.25), at(0.85, -0.25)],
                Stroke::new(line.width * 0.7, color),
            );
            p.line_segment(
                [at(-0.85, 0.3), at(0.85, 0.3)],
                Stroke::new(line.width * 0.7, color),
            );
            p.line_segment(
                [at(-0.2, -0.75), at(-0.2, 0.75)],
                Stroke::new(line.width * 0.7, color),
            );
        }
        Icon::Text => {
            p.line_segment([at(-0.75, -0.7), at(0.75, -0.7)], line);
            p.line_segment([at(0.0, -0.7), at(0.0, 0.8)], line);
            p.line_segment([at(-0.3, 0.8), at(0.3, 0.8)], line);
        }
        Icon::Globe => {
            p.circle_stroke(c, s * 0.85, line);
            p.add(Shape::line(
                vec![at(0.0, -0.85), at(0.45, -0.4), at(0.45, 0.4), at(0.0, 0.85)],
                Stroke::new(line.width * 0.7, color),
            ));
            p.add(Shape::line(
                vec![
                    at(0.0, -0.85),
                    at(-0.45, -0.4),
                    at(-0.45, 0.4),
                    at(0.0, 0.85),
                ],
                Stroke::new(line.width * 0.7, color),
            ));
            p.line_segment(
                [at(-0.85, 0.0), at(0.85, 0.0)],
                Stroke::new(line.width * 0.7, color),
            );
        }
        Icon::Film => {
            p.rect_stroke(
                Rect::from_center_size(c, vec2(s * 1.7, s * 1.5)),
                round(3),
                line,
                StrokeKind::Middle,
            );
            for k in 0..3 {
                let y = -0.5 + k as f32 * 0.5;
                p.rect_filled(
                    Rect::from_center_size(at(-0.6, y), vec2(s * 0.2, s * 0.2)),
                    round(1),
                    color,
                );
                p.rect_filled(
                    Rect::from_center_size(at(0.6, y), vec2(s * 0.2, s * 0.2)),
                    round(1),
                    color,
                );
            }
        }
    }
}

/// Rounded pill button with optional icon. Returns the click response.
pub fn pill(
    ui: &mut Ui,
    icon: Option<Icon>,
    label: &str,
    selected: bool,
    enabled: bool,
) -> Response {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley =
        (!label.is_empty()).then(|| ui.painter().layout_no_wrap(label.to_string(), font, BROWN));
    let icon_w = if icon.is_some() { 20.0 } else { 0.0 };
    let gap = if icon.is_some() && galley.is_some() {
        6.0
    } else {
        0.0
    };
    let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
    let size = vec2((icon_w + gap + text_w + 24.0).max(36.0), 34.0);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let hovered = resp.hovered() && enabled;
    let fill = if selected {
        PINK
    } else if hovered {
        PINK_SOFT
    } else {
        PEACH
    };
    let alpha = if enabled { 255 } else { 110 };
    let p = ui.painter();
    p.rect_filled(rect, round(17), fill.gamma_multiply(alpha as f32 / 255.0));
    p.rect_stroke(
        rect,
        round(17),
        Stroke::new(
            if hovered || selected { 2.0 } else { 1.5 },
            BROWN.gamma_multiply(alpha as f32 / 255.0),
        ),
        StrokeKind::Inside,
    );
    let ink_color = BROWN.gamma_multiply(alpha as f32 / 255.0);
    let mut x = rect.left() + (rect.width() - (icon_w + gap + text_w)) / 2.0;
    if let Some(ic) = icon {
        draw_icon(
            p,
            ic,
            Rect::from_min_size(pos2(x, rect.center().y - 10.0), vec2(20.0, 20.0)),
            ink_color,
        );
        x += icon_w + gap;
    }
    if let Some(g) = galley {
        p.galley(pos2(x, rect.center().y - g.size().y / 2.0), g, ink_color);
    }
    resp
}

/// Round icon button.
pub fn round_button(
    ui: &mut Ui,
    icon: Icon,
    diameter: f32,
    selected: bool,
    enabled: bool,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::splat(diameter),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let hovered = resp.hovered() && enabled;
    let a = if enabled { 1.0 } else { 0.4 };
    let r = diameter / 2.0 - 1.0 + if hovered { 1.0 } else { 0.0 };
    let fill = if selected {
        PINK
    } else if hovered {
        PINK_SOFT
    } else {
        PEACH
    };
    let p = ui.painter();
    p.circle_filled(rect.center(), r, fill.gamma_multiply(a));
    p.circle_stroke(
        rect.center(),
        r,
        Stroke::new(if hovered { 2.2 } else { 1.8 }, BROWN.gamma_multiply(a)),
    );
    draw_icon(
        p,
        icon,
        Rect::from_center_size(rect.center(), Vec2::splat(diameter * 0.5)),
        BROWN.gamma_multiply(a),
    );
    resp
}

/// The big kitten-head play / pause button.
pub fn kitten_play_button(ui: &mut Ui, playing: bool, diameter: f32) -> Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(diameter * 1.25, diameter * 1.2), Sense::click());
    let hovered = resp.hovered();
    let c = pos2(rect.center().x, rect.bottom() - diameter * 0.5 - 1.0);
    let r = diameter * 0.5 * if hovered { 1.04 } else { 1.0 };
    let p = ui.painter();
    let outline = Stroke::new(2.4, BROWN);
    // ears
    for sx in [-1.0f32, 1.0] {
        let base_l = pos2(c.x + sx * r * 0.95, c.y - r * 0.35);
        let base_r = pos2(c.x + sx * r * 0.28, c.y - r * 0.95);
        let tip = pos2(c.x + sx * r * 0.85, c.y - r * 1.2);
        p.add(Shape::convex_polygon(
            vec![base_l, tip, base_r],
            PINK,
            outline,
        ));
        let k = 0.55;
        let ctr = pos2(
            (base_l.x + base_r.x + tip.x) / 3.0,
            (base_l.y + base_r.y + tip.y) / 3.0,
        );
        let shrink = |q: Pos2| pos2(ctr.x + (q.x - ctr.x) * k, ctr.y + (q.y - ctr.y) * k);
        p.add(Shape::convex_polygon(
            vec![shrink(base_l), shrink(tip), shrink(base_r)],
            PINK_SOFT,
            Stroke::NONE,
        ));
    }
    // whiskers
    for sx in [-1.0f32, 1.0] {
        for k in 0..3 {
            let y = c.y + r * (0.05 + 0.2 * k as f32);
            p.line_segment(
                [
                    pos2(c.x + sx * r * 0.92, y),
                    pos2(c.x + sx * r * 1.22, y + (k as f32 - 1.0) * r * 0.12),
                ],
                Stroke::new(1.6, BROWN_SOFT),
            );
        }
    }
    // face
    p.circle_filled(c, r, if hovered { PINK_SOFT } else { PINK });
    p.circle_stroke(c, r, outline);
    // cheeks
    for sx in [-1.0f32, 1.0] {
        ellipse(
            p,
            pos2(c.x + sx * r * 0.62, c.y + r * 0.42),
            vec2(r * 0.17, r * 0.11),
            0.0,
            Color32::from_rgb(255, 150, 170),
        );
    }
    // eyes
    for sx in [-1.0f32, 1.0] {
        p.circle_filled(pos2(c.x + sx * r * 0.38, c.y - r * 0.38), r * 0.07, BROWN);
    }
    let icon_rect = Rect::from_center_size(pos2(c.x, c.y + r * 0.1), Vec2::splat(r * 0.85));
    draw_icon(
        p,
        if playing { Icon::Pause } else { Icon::Play },
        icon_rect,
        BROWN,
    );
    resp
}

/// Rounded panel frame used for the three zones.
pub fn card_frame(fill: Color32) -> egui::Frame {
    egui::Frame::new()
        .fill(fill)
        .stroke(ink(2.0))
        .corner_radius(round(18))
        .inner_margin(egui::Margin::same(12))
}

/// Click or drag in the little frame to place something on the picture (`pos` is 0..1 of the frame).
pub fn position_picker(ui: &mut Ui, pos: &mut [f32; 2]) {
    let (r, resp) = ui.allocate_exact_size(vec2(66.0, 66.0), Sense::click_and_drag());
    let p = ui.painter();
    p.rect_filled(r, round(8), CREAM);
    p.rect_stroke(r, round(8), ink(1.4), StrokeKind::Inside);
    for gy in 0..3 {
        for gx in 0..3 {
            p.circle_filled(
                r.min + vec2(11.0 + gx as f32 * 22.0, 11.0 + gy as f32 * 22.0),
                4.0,
                PEACH_DEEP,
            );
        }
    }
    let dot = r.min + vec2(pos[0].clamp(0.0, 1.0) * 66.0, pos[1].clamp(0.0, 1.0) * 66.0);
    p.circle_filled(dot, 5.5, PINK_DEEP);
    p.circle_stroke(dot, 5.5, ink(1.5));
    if let (true, Some(at)) = (
        resp.clicked() || resp.dragged(),
        resp.interact_pointer_pos(),
    ) {
        let q = (at - r.min) / 66.0;
        *pos = [q.x.clamp(0.0, 1.0), q.y.clamp(0.0, 1.0)];
    }
}
