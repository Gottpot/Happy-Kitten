//! Audio tab: sound settings of the selected clips (volume, fades, effect) and the sound files.

use crate::app::HappyKitten;
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::pool::ellipsize;
use crate::theme::*;
use crate::trf;
use egui::{Ui, vec2};
use hk_core::{AudioFx, AudioSpec, Id, MediaKind, Tick};

impl HappyKitten {
    /// Selected clips that make sound: audio clips and video clips, not text, graphics or stills.
    pub fn selected_sound_clips(&self) -> Vec<Id> {
        self.selected_clips()
            .into_iter()
            .filter(|id| {
                self.doc.project.clip(*id).is_some_and(|(_, c)| {
                    !c.is_layer()
                        && self
                            .doc
                            .project
                            .media(c.media)
                            .is_some_and(|m| matches!(m.kind, MediaKind::Audio | MediaKind::Video))
                })
            })
            .collect()
    }

    pub fn pool_audio(&mut self, ui: &mut Ui) {
        let sound = self.selected_sound_clips();
        // a freshly selected sound clip brings up its settings
        let primary = sound.first().copied();
        if primary != self.audio_seen {
            self.audio_seen = primary;
            if primary.is_some() {
                self.audio_settings = true;
            }
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            if kit::pill(
                ui,
                Some(Icon::Speaker),
                tr("Klang"),
                self.audio_settings,
                true,
            )
            .on_hover_text(tr(
                "Lautstärke, Ein- und Ausblenden und Effekte des gewählten Clips",
            ))
            .clicked()
            {
                self.audio_settings = true;
            }
            if kit::pill(
                ui,
                Some(Icon::Folder),
                tr("Dateien"),
                !self.audio_settings,
                true,
            )
            .clicked()
            {
                self.audio_settings = false;
            }
        });
        ui.add_space(6.0);
        if !self.audio_settings {
            self.pool_files(ui, true);
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match sound.first() {
                Some(id) => self.audio_editor(ui, *id, &sound),
                None => {
                    ui.label(
                        egui::RichText::new(tr(
                            "Wähle einen Audio- oder Video-Clip in der Timeline, um Lautstärke, Ein- und Ausblenden und Effekte zu ändern.",
                        ))
                        .color(BROWN_SOFT),
                    );
                }
            });
    }

    fn audio_editor(&mut self, ui: &mut Ui, id: Id, all: &[Id]) {
        let Some((orig, dur, media)) = self
            .doc
            .project
            .clip(id)
            .map(|(_, c)| (c.audio, c.duration, c.media))
        else {
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
                egui::RichText::new(trf!(
                    "Klang bearbeiten: {name}",
                    name = ellipsize(&name, 22)
                ))
                .size(18.0),
            );
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Clips bearbeiten", n = others.len() + 1)).size(18.0),
            );
            ui.label(
                egui::RichText::new(tr("Änderungen gelten für alle gewählten Clips."))
                    .size(12.0)
                    .color(BROWN_SOFT),
            );
        }
        let mut s: AudioSpec = orig;
        let max_fade = dur.secs().clamp(0.2, 30.0) as f32;
        ui.spacing_mut().slider_width = 200.0;
        egui::Grid::new("audio-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(tr("Lautstärke"));
                ui.horizontal(|ui| {
                    let mut pct = s.volume * 100.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut pct, 0.0..=200.0)
                                .suffix(" %")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        s.volume = pct / 100.0;
                    }
                    if kit::pill(ui, None, "100 %", s.volume == 1.0, true).clicked() {
                        s.volume = 1.0;
                    }
                });
                ui.end_row();
                ui.label(tr("Einblenden"));
                let mut fi = s.fade_in.secs() as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut fi, 0.0..=max_fade)
                            .suffix(" s")
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    s.fade_in = Tick::from_secs(fi as f64);
                }
                ui.end_row();
                ui.label(tr("Ausblenden"));
                let mut fo = s.fade_out.secs() as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut fo, 0.0..=max_fade)
                            .suffix(" s")
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    s.fade_out = Tick::from_secs(fo as f64);
                }
                ui.end_row();
            });
        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr("Effekt")).size(16.0));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for fx in AudioFx::ALL {
                if kit::pill(ui, None, tr(fx.name()), s.fx == fx, true)
                    .on_hover_text(tr(fx.blurb()))
                    .clicked()
                {
                    s.fx = fx;
                }
            }
        });
        ui.label(
            egui::RichText::new(tr(s.fx.blurb()))
                .size(12.5)
                .color(BROWN_SOFT),
        );
        ui.add_enabled_ui(s.fx != AudioFx::None, |ui| {
            egui::Grid::new("audio-fx-grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label(tr("Stärke"));
                    let mut pct = s.fx_amount * 100.0;
                    if ui
                        .add(
                            egui::Slider::new(&mut pct, 0.0..=100.0)
                                .suffix(" %")
                                .fixed_decimals(0),
                        )
                        .changed()
                    {
                        s.fx_amount = pct / 100.0;
                    }
                    ui.end_row();
                });
        });
        if s != orig {
            let s = s.sanitized();
            let key = id.wrapping_mul(0x9E37_79B9) ^ 0xA0D1;
            let others = others.clone();
            self.doc.edit_merge(key, |p| {
                if let Some(a) = p.audio_of_mut(id) {
                    *a = s;
                }
                for other in others {
                    if let Some(a) = p.audio_of_mut(other) {
                        a.apply_diff(&orig, &s);
                    }
                }
            });
        }
        ui.add_space(6.0);
        if kit::pill(ui, None, tr("Zurücksetzen"), false, true)
            .on_hover_text(tr(
                "Lautstärke, Ein- und Ausblenden und Effekt auf Standard",
            ))
            .clicked()
        {
            let ids: Vec<Id> = std::iter::once(id).chain(others.iter().copied()).collect();
            self.doc.edit(|p| {
                for i in ids {
                    if let Some(a) = p.audio_of_mut(i) {
                        *a = AudioSpec::default();
                    }
                }
            });
        }
    }
}
