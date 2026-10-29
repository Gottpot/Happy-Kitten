//! Colour tab: temperature and tint of the selected picture clips.

use crate::app::HappyKitten;
use crate::i18n::tr;
use crate::kit;
use crate::pool::ellipsize;
use crate::theme::*;
use crate::trf;
use egui::Ui;
use hk_core::{Grade, Id, MediaKind};

impl HappyKitten {
    /// Selected clips that show a picture: videos and stills, not text, graphics or audio.
    pub fn selected_picture_clips(&self) -> Vec<Id> {
        self.selected_clips()
            .into_iter()
            .filter(|id| {
                self.doc.project.clip(*id).is_some_and(|(_, c)| {
                    !c.is_layer()
                        && self
                            .doc
                            .project
                            .media(c.media)
                            .is_some_and(|m| matches!(m.kind, MediaKind::Video | MediaKind::Image))
                })
            })
            .collect()
    }

    pub fn pool_color(&mut self, ui: &mut Ui) {
        let pictures = self.selected_picture_clips();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match pictures.first() {
                Some(id) => self.color_editor(ui, *id, &pictures),
                None => {
                    ui.label(
                        egui::RichText::new(tr(
                            "Wähle einen Video-Clip oder ein Bild in der Timeline, um Farbtemperatur und Farbton zu ändern.",
                        ))
                        .color(BROWN_SOFT),
                    );
                }
            });
    }

    fn color_editor(&mut self, ui: &mut Ui, id: Id, all: &[Id]) {
        let Some((orig, media)) = self.doc.project.clip(id).map(|(_, c)| (c.grade, c.media)) else {
            return;
        };
        let others: Vec<Id> = all.iter().copied().filter(|o| *o != id).collect();
        let name = self
            .doc
            .project
            .media(media)
            .map(|m| m.name.clone())
            .unwrap_or_default();
        if others.is_empty() {
            ui.label(
                egui::RichText::new(trf!("Farbe anpassen: {name}", name = ellipsize(&name, 22)))
                    .size(18.0),
            );
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Clips anpassen", n = others.len() + 1)).size(18.0),
            );
            ui.label(
                egui::RichText::new(tr("Änderungen gelten für alle gewählten Clips."))
                    .size(12.0)
                    .color(BROWN_SOFT),
            );
        }
        let mut g = orig;
        ui.spacing_mut().slider_width = 200.0;
        let row = |ui: &mut Ui, label: &str, low: &str, high: &str, value: &mut f32| {
            ui.label(label);
            ui.vertical(|ui| {
                let mut pct = *value * 100.0;
                if ui
                    .add(egui::Slider::new(&mut pct, -100.0..=100.0).fixed_decimals(0))
                    .changed()
                {
                    *value = pct / 100.0;
                }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(low).size(12.0).color(BROWN_SOFT));
                    ui.add_space((ui.available_width() - 60.0).max(0.0));
                    ui.label(egui::RichText::new(high).size(12.0).color(BROWN_SOFT));
                });
            });
            ui.end_row();
        };
        egui::Grid::new("color-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                row(
                    ui,
                    tr("Temperatur"),
                    tr("Kalt"),
                    tr("Warm"),
                    &mut g.temperature,
                );
                row(ui, tr("Farbton"), tr("Grün"), tr("Magenta"), &mut g.tint);
            });
        if g != orig {
            let g = g.sanitized();
            let key = id.wrapping_mul(0x9E37_79B9) ^ 0xC010;
            let others = others.clone();
            self.doc.edit_merge(key, |p| {
                if let Some(x) = p.grade_of_mut(id) {
                    *x = g;
                }
                for other in others {
                    if let Some(x) = p.grade_of_mut(other) {
                        x.apply_diff(&orig, &g);
                    }
                }
            });
        }
        ui.add_space(6.0);
        if kit::pill(ui, None, tr("Zurücksetzen"), false, true)
            .on_hover_text(tr("Temperatur und Farbton auf neutral"))
            .clicked()
        {
            let ids: Vec<Id> = std::iter::once(id).chain(others.iter().copied()).collect();
            self.doc.edit(|p| {
                for i in ids {
                    if let Some(x) = p.grade_of_mut(i) {
                        *x = Grade::default();
                    }
                }
            });
        }
    }
}
