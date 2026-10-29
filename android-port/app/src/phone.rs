//! Phone layout: slim header with a menu, player on top, timeline below it and the tool tabs
//! (files, text, blends, ...) at the bottom. Tablets keep the desktop layout.

use crate::app::HappyKitten;
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::theme::*;
use egui::{Rect, Sense, Ui, pos2, vec2};

impl HappyKitten {
    /// Keeps the app out from under the status bar and the gesture bar.
    pub fn safe_area(&mut self, ui: &mut Ui, top: f32, bottom: f32) {
        if top > 0.5 {
            egui::Panel::top("safe-top")
                .exact_size(top)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(PINK))
                .show(ui, |_| {});
        }
        if bottom > 0.5 {
            egui::Panel::bottom("safe-bottom")
                .exact_size(bottom)
                .show_separator_line(false)
                .frame(egui::Frame::new().fill(CREAM))
                .show(ui, |_| {});
        }
    }

    pub fn header_phone(&mut self, ui: &mut Ui) {
        let frame = egui::Frame::new()
            .fill(PINK)
            .inner_margin(egui::Margin::symmetric(10, 6));
        egui::Panel::top("header")
            .exact_size(56.0)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                let full = ui.max_rect();
                ui.painter().line_segment(
                    [
                        full.left_bottom() + vec2(-20.0, 6.0),
                        full.right_bottom() + vec2(20.0, 6.0),
                    ],
                    ink(2.0),
                );
                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    // cat badge: tap for "about"
                    let (badge, resp) = ui.allocate_exact_size(vec2(44.0, 40.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(badge, round(14), CREAM);
                    p.rect_stroke(badge, round(14), ink(1.8), egui::StrokeKind::Inside);
                    egui::Image::new(&self.logo)
                        .uv(Rect::from_min_max(pos2(0.215, 0.07), pos2(0.795, 0.60)))
                        .corner_radius(round(12))
                        .paint_at(ui, badge.shrink(2.0));
                    if resp.clicked() {
                        self.about = true;
                    }
                    let name = if self.doc.is_dirty() {
                        format!("{} •", self.doc.project.name)
                    } else {
                        self.doc.project.name.clone()
                    };
                    ui.add(
                        egui::Label::new(egui::RichText::new(name).size(17.0).color(BROWN))
                            .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.menu_button(ui);
                        if kit::pill(ui, Some(Icon::Film), tr("Exportieren"), true, true).clicked()
                        {
                            self.open_export_dialog();
                        }
                    });
                });
            });
    }

    /// The "more" button: project actions that do not fit into the slim header.
    fn menu_button(&mut self, ui: &mut Ui) {
        let (rect, resp) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::click());
        let p = ui.painter();
        p.circle_filled(rect.center(), 19.0, CREAM);
        p.circle_stroke(rect.center(), 19.0, ink(1.8));
        for dy in [-7.0, 0.0, 7.0] {
            p.circle_filled(rect.center() + vec2(0.0, dy), 2.4, BROWN);
        }
        egui::Popup::menu(&resp).show(|ui| {
            ui.set_min_width(190.0);
            if ui.button(tr("Neu")).clicked() {
                self.new_project();
                ui.close();
            }
            if ui.button(tr("Öffnen")).clicked() {
                self.open_project();
                ui.close();
            }
            if ui.button(tr("Speichern")).clicked() {
                self.save_project(false);
                ui.close();
            }
            if ui.button(tr("Standbild")).clicked() {
                self.save_still();
                ui.close();
            }
            ui.separator();
            ui.label(egui::RichText::new(tr("Bildformat")).size(13.0).color(BROWN_SOFT));
            self.canvas_bar_row(ui);
            ui.separator();
            self.language_button(ui);
        });
    }

    /// Phone: the tool tabs sit in a panel at the bottom.
    pub fn pool_panel_phone(&mut self, ui: &mut Ui) {
        let screen_h = ui.ctx().content_rect().height();
        let frame = egui::Frame::new()
            .fill(CREAM)
            .inner_margin(egui::Margin::symmetric(6, 4));
        egui::Panel::bottom("pool")
            // fixed height: the content would otherwise push the panel taller than we want
            .exact_size(screen_h * 0.30)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                kit::card_frame(PEACH).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    self.pool_tabs(ui);
                    ui.add_space(4.0);
                    // The tools never grow the panel: what does not fit scrolls.
                    let room = ui.available_height().max(60.0);
                    egui::ScrollArea::vertical()
                        .id_salt("pool-phone")
                        .max_height(room)
                        .auto_shrink([false, false])
                        .show(ui, |ui| self.pool_content(ui));
                });
            });
    }
}
