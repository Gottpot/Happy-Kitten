//! Preview player (top right) with the big kitten play button.

use crate::app::{HappyKitten, Payload};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::theme::*;
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use hk_core::Tick;

impl HappyKitten {
    pub fn preview_panel(&mut self, ui: &mut Ui) {
        let phone = self.phone;
        let frame = egui::Frame::new().fill(CREAM).inner_margin(if phone {
            egui::Margin::symmetric(6, 4)
        } else {
            egui::Margin {
                left: 6,
                right: 12,
                top: 12,
                bottom: 12,
            }
        });
        egui::CentralPanel::default().frame(frame).show(ui, |ui| {
            kit::card_frame(PEACH).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                if !phone {
                    self.canvas_bar(ui);
                }
                let controls_h = if phone { 92.0 } else { 100.0 };
                let avail = ui.available_size();
                let screen_h = (avail.y - controls_h).max(80.0);
                let (tex, (tw, th)) = self.engine.preview_texture();
                let aspect = tw as f32 / th.max(1) as f32;
                let mut size = vec2(avail.x, avail.x / aspect);
                if size.y > screen_h {
                    size = vec2(screen_h * aspect, screen_h);
                }
                ui.vertical_centered(|ui| {
                    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(rect.expand(5.0), round(20), CREAM);
                    p.rect_stroke(rect.expand(5.0), round(20), ink(2.2), StrokeKind::Inside);
                    egui::Image::new((tex, size))
                        .corner_radius(round(14))
                        .paint_at(ui, rect);
                    if self.duration() == Tick::ZERO {
                        let p = ui.painter();
                        p.rect_filled(
                            rect,
                            round(14),
                            Color32::from_rgba_unmultiplied(255, 228, 208, 235),
                        );
                        egui::Image::new(&self.logo)
                            .corner_radius(round(14))
                            .paint_at(
                                ui,
                                Rect::from_center_size(
                                    rect.center() - vec2(0.0, 12.0),
                                    egui::Vec2::splat(rect.height() * 0.62),
                                ),
                            );
                        ui.painter().text(
                            pos2(rect.center().x, rect.bottom() - 22.0),
                            Align2::CENTER_CENTER,
                            if phone {
                                tr("Tippe aufs Kätzchen oben links und wähle Dateien")
                            } else {
                                tr("Zieh ein Medium auf die Timeline")
                            },
                            egui::FontId::proportional(15.0),
                            BROWN,
                        );
                    }
                    ui.add_space(8.0);
                    self.player_controls(ui);
                });
            });
        });
    }

    /// Canvas shape (16:9, 9:16 for Shorts, ...) and how pictures fill it.
    fn canvas_bar(&mut self, ui: &mut Ui) {
        if self.phone {
            egui::ScrollArea::horizontal()
                .id_salt("canvas-bar")
                .auto_shrink([false, true])
                .show(ui, |ui| self.canvas_bar_row(ui));
        } else {
            self.canvas_bar_row(ui);
        }
    }

    pub fn canvas_bar_row(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            if !self.phone {
                ui.label(
                    egui::RichText::new(tr("Bildformat"))
                        .size(13.0)
                        .color(BROWN_SOFT),
                );
            }
            let (cw, ch) = self.doc.project.size;
            for (label, w, h) in hk_core::CANVAS_SHAPES {
                let selected = cw as u64 * h as u64 == ch as u64 * w as u64;
                if kit::pill(ui, None, label, selected, true)
                    .on_hover_text(if h > w {
                        tr("Hochformat: Shorts, Reels, TikTok")
                    } else {
                        tr("Bildformat des Projekts")
                    })
                    .clicked()
                {
                    self.set_canvas(w, h);
                }
            }
            ui.add_space(6.0);
            let mut fit = self.doc.project.fit;
            egui::ComboBox::from_id_salt("canvas-fit")
                .selected_text(tr(fit.name()))
                .width(130.0)
                .show_ui(ui, |ui| {
                    for f in hk_core::FitMode::ALL {
                        ui.selectable_value(&mut fit, f, tr(f.name()));
                    }
                });
            if fit != self.doc.project.fit {
                self.doc.edit(|p| p.fit = fit);
            }
        });
    }

    fn player_controls(&mut self, ui: &mut Ui) {
        let fps = self.doc.project.fps;
        let total = self.duration();
        let phone = self.phone;
        let row_h = if phone { 84.0 } else { 76.0 };
        let (row, _) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::hover());
        let p = ui.painter().clone();
        let time_y = if phone { row.top() + 12.0 } else { row.center().y + 4.0 };
        if phone {
            p.text(
                pos2(row.center().x, time_y),
                Align2::CENTER_CENTER,
                format!("{} / {}", self.playhead.timecode(fps), total.timecode(fps)),
                egui::FontId::proportional(16.0),
                BROWN,
            );
        } else {
            p.text(
                row.left_center() + vec2(8.0, 4.0),
                Align2::LEFT_CENTER,
                self.playhead.timecode(fps),
                egui::FontId::proportional(24.0),
                BROWN,
            );
            p.text(
                row.right_center() + vec2(-8.0, 4.0),
                Align2::RIGHT_CENTER,
                format!("/ {}", total.timecode(fps)),
                egui::FontId::proportional(17.0),
                BROWN_SOFT,
            );
        }
        let (small, big, gap) = if phone { (40.0, 56.0, 10.0) } else { (44.0, 64.0, 12.0) };
        let buttons_w = 4.0 * small + (big + 18.0) + 4.0 * gap;
        let center = if phone {
            Rect::from_center_size(row.center() + vec2(0.0, 12.0), vec2(buttons_w, big + 6.0))
        } else {
            Rect::from_center_size(row.center(), vec2(buttons_w, row_h))
        };
        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(center)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                let has = total > Tick::ZERO;
                if kit::round_button(ui, Icon::SkipStart, small, false, has)
                    .on_hover_text(tr("Zum Anfang (Pos1)"))
                    .clicked()
                {
                    self.stop();
                    self.set_playhead(Tick::ZERO);
                }
                if kit::round_button(ui, Icon::StepBack, small, false, has)
                    .on_hover_text(tr("Ein Bild zurück (←)"))
                    .clicked()
                {
                    self.stop();
                    self.set_playhead(self.playhead - Tick::from_frames(1, fps));
                }
                if kit::kitten_play_button(ui, self.playing, big)
                    .on_hover_text(tr("Abspielen / Pause (Leertaste)"))
                    .clicked()
                {
                    self.toggle_play();
                }
                if kit::round_button(ui, Icon::StepForward, small, false, has)
                    .on_hover_text(tr("Ein Bild vor (→)"))
                    .clicked()
                {
                    self.stop();
                    self.set_playhead(self.playhead + Tick::from_frames(1, fps));
                }
                if kit::round_button(ui, Icon::SkipEnd, small, false, has)
                    .on_hover_text(tr("Zum Ende (Ende)"))
                    .clicked()
                {
                    self.stop();
                    self.set_playhead(total);
                }
            },
        );
    }

    /// Floating label that follows the mouse while something is dragged.
    pub fn drag_ghost(&mut self, ctx: &egui::Context) {
        let Some(payload) = egui::DragAndDrop::payload::<Payload>(ctx) else {
            return;
        };
        let Some(pos) = ctx.pointer_latest_pos() else {
            return;
        };
        let label = match *payload {
            Payload::Media(id) => self
                .doc
                .project
                .media(id)
                .map(|m| m.name.clone())
                .unwrap_or_default(),
            Payload::Effect(k) => tr(k.name()).to_string(),
            Payload::Fx(k) => tr(k.name()).to_string(),
            Payload::LibMedia(id) => self
                .lib
                .item(id)
                .map(|i| i.name.clone())
                .unwrap_or_default(),
            Payload::Text(pick) => self.text_preset(pick).map(|s| s.text).unwrap_or_default(),
            Payload::Overlay(id) => self
                .doc
                .project
                .media(id)
                .map(|m| m.name.clone())
                .unwrap_or_default(),
        };
        ctx.set_cursor_icon(egui::CursorIcon::Grabbing);
        let layer = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag-ghost"));
        let p = ctx.layer_painter(layer);
        let font = egui::FontId::proportional(14.0);
        let g = p.layout_no_wrap(label, font, BROWN);
        let r = Rect::from_min_size(pos + vec2(14.0, 10.0), g.size() + vec2(34.0, 14.0));
        p.rect_filled(r, round(16), PINK);
        p.rect_stroke(r, round(16), Stroke::new(2.0, BROWN), StrokeKind::Inside);
        kit::paw(&p, pos2(r.left() + 15.0, r.center().y), 6.0, BROWN);
        p.galley(
            pos2(r.left() + 28.0, r.center().y - g.size().y / 2.0),
            g,
            BROWN,
        );
    }
}
