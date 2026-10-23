//! Graphics tab: PNGs and other pictures laid over the video, with size, position, rotation,
//! opacity and the same fade effects as text.

use crate::app::{HappyKitten, Payload, PoolTab, Selection};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::pool::{ellipsize, fx_combo};
use crate::theme::*;
use crate::trf;
use egui::{Align2, Color32, Rect, Sense, StrokeKind, Ui, pos2, vec2};
use hk_core::{Id, MediaKind, MediaSource, OverlaySpec, Tick};

const CARD_W: f32 = 150.0;
const THUMB_H: f32 = 84.0;

impl HappyKitten {
    /// Lay `media` over the picture at the playhead on the top-most video track that has room.
    pub fn add_overlay_at_playhead(&mut self, media: Id) {
        let at = self.playhead;
        let dur = Tick::from_secs(5.0);
        let tracks: Vec<Id> = self
            .doc
            .project
            .tracks
            .iter()
            .rev()
            .filter(|t| t.kind == hk_core::TrackKind::Video)
            .map(|t| t.id)
            .collect();
        let res = self.doc.edit(|p| {
            for t in &tracks {
                if p.free_start(*t, None, at, dur) == Some(at) {
                    return p.add_overlay_clip(*t, media, at);
                }
            }
            match tracks.first() {
                Some(t) => p.add_overlay_clip(*t, media, at),
                None => Err(hk_core::EditError::NoSuchTrack),
            }
        });
        match res {
            Ok(id) => {
                self.set_selection(Selection::Clip(id));
                self.tab = PoolTab::Graphics;
            }
            Err(e) => self.say(e.to_string()),
        }
    }

    /// Pick pictures, add them to the project and lay each one over the video at the playhead.
    pub fn add_graphics_dialog(&mut self) {
        let Some(paths) = rfd::FileDialog::new()
            .set_title(tr(
                "Grafiken wählen (PNG mit Transparenz funktioniert am besten)",
            ))
            .add_filter(
                tr("Bilder"),
                &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"],
            )
            .pick_files()
        else {
            return;
        };
        for path in paths {
            let probed = match hk_media::probe(&path) {
                Ok(p) if p.kind == MediaKind::Image => p,
                Ok(_) => {
                    self.say("Das ist keine Bilddatei");
                    continue;
                }
                Err(e) => {
                    self.say(e.to_string());
                    continue;
                }
            };
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Grafik".into());
            let media = self.doc.edit(|p| {
                let id = p.ensure_media(
                    name,
                    MediaSource::File(path.clone()),
                    probed.kind,
                    probed.duration,
                    probed.size,
                );
                p.set_media_alpha(id, probed.alpha);
                id
            });
            self.add_overlay_at_playhead(media);
        }
    }

    pub fn pool_graphics(&mut self, ui: &mut Ui) {
        let selected = self.selected_clips().into_iter().find(|id| {
            self.doc
                .project
                .clip(*id)
                .is_some_and(|(_, c)| c.is_overlay())
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if let Some(id) = selected {
                self.graphic_editor(ui, id);
                ui.add_space(10.0);
                ui.label(egui::RichText::new(tr("Weitere Grafiken")).size(18.0));
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if kit::pill(ui, Some(Icon::Plus), tr("Grafik (PNG)"), true, true)
                    .on_hover_text(tr("Bild wählen und über das Video legen – Transparenz bleibt erhalten"))
                    .clicked()
                {
                    self.add_graphics_dialog();
                }
            });
            ui.label(
                egui::RichText::new(tr("Karte auf eine Videospur ziehen oder ＋ anklicken. Bilder mit Transparenz werden automatisch zu Grafiken."))
                    .size(12.5)
                    .color(BROWN_SOFT),
            );
            ui.add_space(2.0);
            let mut items: Vec<(Id, String, bool)> =
                self.doc.project.media.iter().filter(|m| m.kind == MediaKind::Image).map(|m| (m.id, m.name.clone(), m.alpha)).collect();
            items.sort_by_key(|(_, _, alpha)| !alpha);
            if items.is_empty() && selected.is_none() {
                ui.add_space(14.0);
                ui.label(egui::RichText::new(tr("Noch keine Bilder im Projekt. Importiere ein PNG oder wähle „Grafik (PNG)“.")).color(BROWN_SOFT));
            }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                for (id, name, alpha) in items {
                    self.graphic_card(ui, id, &name, alpha);
                }
            });
        });
    }

    fn graphic_card(&mut self, ui: &mut Ui, id: Id, name: &str, alpha: bool) {
        let (rect, resp) =
            ui.allocate_exact_size(vec2(CARD_W, THUMB_H + 34.0), Sense::click_and_drag());
        let thumb = Rect::from_min_size(rect.min + vec2(6.0, 6.0), vec2(CARD_W - 12.0, THUMB_H));
        let hovered = resp.hovered();
        let p = ui.painter();
        p.rect_filled(rect, round(14), if hovered { PINK_SOFT } else { CREAM });
        p.rect_stroke(
            rect,
            round(14),
            ink(if hovered { 2.0 } else { 1.5 }),
            StrokeKind::Inside,
        );
        // checkerboard behind the picture so transparency is visible
        let cell = 10.0;
        let (cols, rows) = (
            (thumb.width() / cell).ceil() as i32,
            (thumb.height() / cell).ceil() as i32,
        );
        let clip = p.with_clip_rect(thumb);
        clip.rect_filled(thumb, round(10), Color32::from_gray(238));
        for gy in 0..rows {
            for gx in 0..cols {
                if (gx + gy) % 2 == 0 {
                    clip.rect_filled(
                        Rect::from_min_size(
                            thumb.min + vec2(gx as f32 * cell, gy as f32 * cell),
                            vec2(cell, cell),
                        ),
                        0.0,
                        Color32::from_gray(214),
                    );
                }
            }
        }
        if let Some(tex) = self.engine.thumb(id) {
            let size = tex.size_vec2();
            let k = (thumb.width() / size.x).min(thumb.height() / size.y);
            egui::Image::new((tex.id(), size * k))
                .paint_at(ui, Rect::from_center_size(thumb.center(), size * k));
        }
        let p = ui.painter();
        p.text(
            pos2(rect.left() + 10.0, rect.bottom() - 16.0),
            Align2::LEFT_CENTER,
            ellipsize(name, 17),
            egui::FontId::proportional(13.0),
            BROWN,
        );
        if alpha {
            let tag = Rect::from_min_size(
                pos2(thumb.right() - 54.0, thumb.top() + 4.0),
                vec2(50.0, 17.0),
            );
            p.rect_filled(tag, round(8), PINK);
            p.rect_stroke(tag, round(8), ink(1.2), StrokeKind::Inside);
            p.text(
                tag.center(),
                Align2::CENTER_CENTER,
                "PNG ✓",
                egui::FontId::proportional(11.0),
                BROWN,
            );
        }
        let plus = Rect::from_center_size(thumb.left_top() + vec2(18.0, 18.0), vec2(26.0, 26.0));
        if hovered {
            p.circle_filled(plus.center(), 13.0, PINK);
            p.circle_stroke(plus.center(), 13.0, ink(1.8));
            kit::draw_icon(
                p,
                Icon::Plus,
                Rect::from_center_size(plus.center(), vec2(15.0, 15.0)),
                BROWN,
            );
        }
        resp.dnd_set_drag_payload(Payload::Overlay(id));
        if (resp.clicked()
            && resp
                .interact_pointer_pos()
                .is_some_and(|q| plus.contains(q)))
            || resp.double_clicked()
        {
            self.add_overlay_at_playhead(id);
        }
        resp.on_hover_text(tr(
            "Auf eine Videospur ziehen: liegt als Grafik über dem Bild",
        ));
    }

    fn graphic_editor(&mut self, ui: &mut Ui, id: Id) {
        let Some(orig) = self
            .doc
            .project
            .clip(id)
            .and_then(|(_, c)| c.overlay.clone())
        else {
            return;
        };
        let others: Vec<Id> = self
            .selected_clips()
            .into_iter()
            .filter(|o| {
                *o != id
                    && self
                        .doc
                        .project
                        .clip(*o)
                        .is_some_and(|(_, c)| c.is_overlay())
            })
            .collect();
        let name = self
            .doc
            .project
            .clip(id)
            .and_then(|(_, c)| self.doc.project.media(c.media))
            .map(|m| m.name.clone())
            .unwrap_or_default();
        let mut s = orig.clone();
        if others.is_empty() {
            ui.label(
                egui::RichText::new(trf!(
                    "Grafik bearbeiten: {name}",
                    name = ellipsize(&name, 22)
                ))
                .size(18.0),
            );
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Grafiken bearbeiten", n = others.len() + 1))
                    .size(18.0),
            );
            ui.label(
                egui::RichText::new(tr("Änderungen gelten für alle gewählten Grafiken."))
                    .size(12.0)
                    .color(BROWN_SOFT),
            );
        }
        ui.spacing_mut().slider_width = 200.0;
        egui::Grid::new("overlay-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(tr("Größe"));
                let mut pct = s.scale * 100.0;
                if ui
                    .add(
                        egui::Slider::new(&mut pct, 3.0..=150.0)
                            .suffix(" %")
                            .fixed_decimals(0)
                            .logarithmic(true),
                    )
                    .changed()
                {
                    s.scale = pct / 100.0;
                }
                ui.end_row();
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
                ui.label(tr("Drehung"));
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::new(&mut s.rotation, -180.0..=180.0)
                            .suffix("°")
                            .fixed_decimals(0),
                    );
                    if kit::pill(ui, None, tr("gerade"), s.rotation == 0.0, true).clicked() {
                        s.rotation = 0.0;
                    }
                });
                ui.end_row();
                ui.label(tr("Deckkraft"));
                let mut o = s.opacity * 100.0;
                if ui
                    .add(
                        egui::Slider::new(&mut o, 0.0..=100.0)
                            .suffix(" %")
                            .fixed_decimals(0),
                    )
                    .changed()
                {
                    s.opacity = o / 100.0;
                }
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr("Ein- und Ausblenden")).size(16.0));
        egui::Grid::new("overlay-fx")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(tr("Einblenden"));
                ui.horizontal(|ui| {
                    fx_combo(ui, "ov-fx-in", &mut s.fx_in);
                    let mut d = s.fx_in_dur.secs() as f32;
                    if ui
                        .add(
                            egui::Slider::new(&mut d, 0.1..=3.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        )
                        .changed()
                    {
                        s.fx_in_dur = Tick::from_secs(d as f64);
                    }
                });
                ui.end_row();
                ui.label(tr("Ausblenden"));
                ui.horizontal(|ui| {
                    fx_combo(ui, "ov-fx-out", &mut s.fx_out);
                    let mut d = s.fx_out_dur.secs() as f32;
                    if ui
                        .add(
                            egui::Slider::new(&mut d, 0.1..=3.0)
                                .suffix(" s")
                                .fixed_decimals(1),
                        )
                        .changed()
                    {
                        s.fx_out_dur = Tick::from_secs(d as f64);
                    }
                });
                ui.end_row();
            });

        if s != orig {
            let key = id.wrapping_mul(0x9E37_79B9) ^ 0x06E7;
            let others = others.clone();
            self.doc.edit_merge(key, |p| {
                if let Some(o) = p.overlay_of_mut(id) {
                    *o = s.clone();
                }
                for other in others {
                    if let Some(o) = p.overlay_of_mut(other) {
                        o.apply_diff(&orig, &s);
                    }
                }
            });
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            if kit::pill(ui, None, tr("Zurücksetzen"), false, true)
                .on_hover_text(tr("Größe, Position, Drehung und Deckkraft auf Standard"))
                .clicked()
            {
                let ids: Vec<Id> = std::iter::once(id).chain(others.iter().copied()).collect();
                self.doc.edit(|p| {
                    for i in ids {
                        if let Some(o) = p.overlay_of_mut(i) {
                            let keep = o.clone();
                            *o = OverlaySpec {
                                fx_in: keep.fx_in,
                                fx_in_dur: keep.fx_in_dur,
                                fx_out: keep.fx_out,
                                fx_out_dur: keep.fx_out_dur,
                                ..Default::default()
                            };
                        }
                    }
                });
            }
            if kit::pill(ui, None, tr("Als Vollbild-Bild"), false, true)
                .on_hover_text(tr(
                    "Nicht mehr über das Video legen, sondern als Bild füllen",
                ))
                .clicked()
            {
                let ids: Vec<Id> = std::iter::once(id).chain(others.iter().copied()).collect();
                self.doc.edit(|p| {
                    for i in ids {
                        p.set_overlay(i, None).ok();
                    }
                });
            }
        });
    }
}
