//! Effects tab: overlay effects (flames, equalizers, glitch, light ...) as cards to drag onto a
//! video track, and the settings of the selected effect clips.

use crate::app::{HappyKitten, Payload, Selection};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::pool::ellipsize;
use crate::theme::*;
use crate::trf;
use egui::{Align2, Rect, Sense, StrokeKind, Ui, pos2, vec2};
use hk_core::{FxGroup, FxKind, FxSpec, Id, MediaKind, Tick, TrackKind};
use std::time::Instant;

const CARD_W: f32 = 150.0;
const THUMB_H: f32 = 84.0;
/// Seconds into the sample scene at which a card shows its still picture.
const CARD_SECS: f32 = 1.4;

/// What the "width" slider means for an effect.
fn width_label(kind: FxKind) -> &'static str {
    match kind {
        FxKind::FireFrame | FxKind::IceFrame | FxKind::LightningFrame | FxKind::NeonFrame => {
            "Breite"
        }
        FxKind::EqBars | FxKind::EqCircle | FxKind::EqWave | FxKind::BeatRings => "Größe",
        FxKind::Glitch => "Verschiebung",
        FxKind::CyberGrid => "Linienbreite",
        FxKind::LaserShow => "Strahlbreite",
        FxKind::LensFlare | FxKind::LightLeak => "Größe",
    }
}

impl HappyKitten {
    /// Selected clips that are effects.
    pub fn selected_fx_clips(&self) -> Vec<Id> {
        self.selected_clips()
            .into_iter()
            .filter(|id| self.doc.project.clip(*id).is_some_and(|(_, c)| c.is_fx()))
            .collect()
    }

    pub fn pool_fx(&mut self, ui: &mut Ui) {
        let selected = self.selected_fx_clips();
        let mut now_hover: Option<FxKind> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if let Some(id) = selected.first() {
                    self.fx_editor(ui, *id, &selected);
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(tr("Effekt hinzufügen")).size(18.0));
                }
                ui.label(
                    egui::RichText::new(tr(
                        "Auf eine Videospur ziehen oder anklicken. Der Effekt liegt über dem Bild darunter, seine Länge änderst du in der Timeline.",
                    ))
                    .size(12.5)
                    .color(BROWN_SOFT),
                );
                for group in FxGroup::ALL {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(tr(group.name())).size(16.0));
                    self.fx_grid(ui, FxKind::of_group(group), &mut now_hover);
                }
            });
        // Hover plays the effect on its card; leaving restores the still picture.
        let prev = self.fx_hover.map(|(k, _)| k);
        if prev != now_hover {
            if let Some(k) = prev {
                self.engine.animate_fx_card(k, CARD_SECS);
            }
            self.fx_hover = now_hover.map(|k| (k, Instant::now()));
        }
        if let Some((k, start)) = self.fx_hover {
            self.engine
                .animate_fx_card(k, CARD_SECS + start.elapsed().as_secs_f32());
            ui.ctx().request_repaint();
        }
    }

    fn fx_grid(&mut self, ui: &mut Ui, kinds: &[FxKind], now_hover: &mut Option<FxKind>) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
            for &kind in kinds {
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(CARD_W, THUMB_H + 50.0), Sense::click_and_drag());
                let thumb =
                    Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(CARD_W - 12.0, THUMB_H));
                let hovered = resp.hovered();
                if hovered {
                    *now_hover = Some(kind);
                }
                let p = ui.painter();
                p.rect_filled(rect, round(14), if hovered { PINK_SOFT } else { CREAM });
                p.rect_stroke(
                    rect,
                    round(14),
                    ink(if hovered { 2.0 } else { 1.5 }),
                    StrokeKind::Inside,
                );
                if let Some(tex) = self.engine.fx_card(kind) {
                    egui::Image::new((tex, thumb.size()))
                        .corner_radius(round(10))
                        .paint_at(ui, thumb);
                }
                let p = ui.painter();
                kit::draw_icon(
                    p,
                    Icon::Bolt,
                    Rect::from_center_size(pos2(rect.left() + 17.0, rect.bottom() - 31.0), vec2(14.0, 14.0)),
                    PINK_DEEP,
                );
                p.text(
                    pos2(rect.left() + 30.0, rect.bottom() - 31.0),
                    Align2::LEFT_CENTER,
                    tr(kind.name()),
                    egui::FontId::proportional(13.5),
                    BROWN,
                );
                p.text(
                    pos2(rect.left() + 10.0, rect.bottom() - 13.0),
                    Align2::LEFT_CENTER,
                    ellipsize(tr(kind.blurb()), 25),
                    egui::FontId::proportional(10.5),
                    BROWN_SOFT,
                );
                resp.dnd_set_drag_payload(Payload::Fx(kind));
                if resp.clicked() {
                    self.add_fx_at_playhead(kind);
                }
                resp.on_hover_text(tr(
                    "Auf eine Videospur ziehen – oder anklicken, um den Effekt am Zeiger einzufügen",
                ));
            }
        });
    }

    /// Put an effect clip at the playhead on the top-most video track that has room there; when
    /// none has, on a new video track on top.
    pub fn add_fx_at_playhead(&mut self, kind: FxKind) {
        let dur = Tick::from_secs(5.0);
        let at = self.playhead;
        let res = self.doc.edit(|p| {
            let tracks: Vec<Id> = p
                .tracks
                .iter()
                .rev()
                .filter(|t| t.kind == TrackKind::Video)
                .map(|t| t.id)
                .collect();
            for t in &tracks {
                if p.free_start(*t, None, at, dur) == Some(at) {
                    return p.add_fx_clip(*t, at, dur, FxSpec::new(kind));
                }
            }
            let track = p.add_track_of(TrackKind::Video)?;
            p.add_fx_clip(track, at, dur, FxSpec::new(kind))
        });
        match res {
            Ok(id) => self.set_selection(Selection::Clip(id)),
            Err(e) => self.say(e.to_string()),
        }
    }

    /// True when the timeline has any sound (an audio clip or a video clip).
    fn project_has_sound(&self) -> bool {
        self.doc
            .project
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .any(|c| {
                !c.is_layer()
                    && self
                        .doc
                        .project
                        .media(c.media)
                        .is_some_and(|m| matches!(m.kind, MediaKind::Audio | MediaKind::Video))
            })
    }

    fn fx_editor(&mut self, ui: &mut Ui, id: Id, all: &[Id]) {
        let Some(orig) = self.doc.project.clip(id).and_then(|(_, c)| c.fx) else {
            return;
        };
        let others: Vec<Id> = all.iter().copied().filter(|o| *o != id).collect();
        if others.is_empty() {
            ui.label(
                egui::RichText::new(trf!(
                    "Effekt bearbeiten: {name}",
                    name = ellipsize(tr(orig.kind.name()), 22)
                ))
                .size(18.0),
            );
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Effekte bearbeiten", n = others.len() + 1))
                    .size(18.0),
            );
            ui.label(
                egui::RichText::new(tr("Änderungen gelten für alle gewählten Effekte."))
                    .size(12.0)
                    .color(BROWN_SOFT),
            );
        }
        if orig.needs_audio() && !self.project_has_sound() {
            ui.label(
                egui::RichText::new(tr(
                    "Tipp: Dieser Effekt reagiert auf den Ton. Lege einen Audio- oder Video-Clip auf die Timeline.",
                ))
                .size(12.0)
                .color(PINK_DEEP),
            );
        }
        let params = orig.kind.params();
        let mut s = orig;
        ui.spacing_mut().slider_width = 190.0;
        egui::Grid::new("fx-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                if params.width {
                    ui.label(tr(width_label(s.kind)));
                    let mut pct = s.width * 100.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut pct, 0.0..=100.0)
                                .suffix(" %")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        s.width = pct / 100.0;
                    }
                    ui.end_row();
                }
                if params.speed {
                    ui.label(tr("Geschwindigkeit"));
                    ui.add(
                        egui::Slider::new(&mut s.speed, 0.0..=4.0)
                            .suffix(" ×")
                            .fixed_decimals(1),
                    );
                    ui.end_row();
                }
                ui.label(tr("Stärke"));
                let mut pct = s.intensity * 100.0;
                if ui
                    .add(
                        egui::Slider::new(&mut pct, 0.0..=200.0)
                            .suffix(" %")
                            .fixed_decimals(0),
                    )
                    .changed()
                {
                    s.intensity = pct / 100.0;
                }
                ui.end_row();
                if params.hue {
                    ui.label(tr("Farbe"));
                    let mut deg = s.hue * 360.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut deg, -180.0..=180.0)
                                .suffix("°")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        s.hue = deg / 360.0;
                    }
                    ui.end_row();
                }
                if params.beat {
                    ui.label(tr("Beat-Reaktion"))
                        .on_hover_text(tr("Wie stark der Effekt mit dem Bass der Musik zuckt"));
                    let mut pct = s.beat * 100.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut pct, 0.0..=100.0)
                                .suffix(" %")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        s.beat = pct / 100.0;
                    }
                    ui.end_row();
                }
                if params.pos {
                    ui.label(tr("Position"));
                    ui.horizontal(|ui| {
                        kit::position_picker(ui, &mut s.pos);
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Slider::new(&mut s.pos[0], -0.2..=1.2)
                                    .text("x")
                                    .show_value(false),
                            );
                            ui.add(
                                egui::Slider::new(&mut s.pos[1], -0.2..=1.2)
                                    .text("y")
                                    .show_value(false),
                            );
                        });
                    });
                    ui.end_row();
                }
                if params.mirror {
                    ui.label(tr("Spiegeln"));
                    ui.checkbox(&mut s.mirror, "");
                    ui.end_row();
                }
                ui.label(tr("Ein- und Ausblenden"));
                let mut fade = s.fade.secs() as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut fade, 0.0..=3.0)
                            .suffix(" s")
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    s.fade = Tick::from_secs(fade as f64);
                }
                ui.end_row();
            });
        if s != orig {
            let s = s.sanitized();
            let key = id.wrapping_mul(0x9E37_79B9) ^ 0xFA11;
            let others = others.clone();
            self.doc.edit_merge(key, |p| {
                if let Some(x) = p.fx_of_mut(id) {
                    *x = s;
                }
                for other in others {
                    if let Some(x) = p.fx_of_mut(other) {
                        x.apply_diff(&orig, &s);
                    }
                }
            });
        }
        ui.add_space(6.0);
        if kit::pill(ui, None, tr("Zurücksetzen"), false, true)
            .on_hover_text(tr("Alle Einstellungen dieses Effekts auf Standard"))
            .clicked()
        {
            let ids: Vec<Id> = std::iter::once(id).chain(others.iter().copied()).collect();
            self.doc.edit(|p| {
                for i in ids {
                    if let Some(x) = p.fx_of_mut(i) {
                        *x = FxSpec::new(x.kind);
                    }
                }
            });
        }
    }
}
