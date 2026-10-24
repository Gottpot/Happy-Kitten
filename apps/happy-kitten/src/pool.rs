//! Media pool (left): files (project or global), text, blends and audio.

use crate::app::{HappyKitten, Payload, PoolTab, Selection, TextPick};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::presets;
use crate::theme::*;
use crate::trf;
use egui::{Align2, Color32, Rect, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use hk_core::{Align, FONTS, Id, TextFx, TextSpec, Tick, TransitionKind};
use std::time::Instant;

const CARD_W: f32 = 150.0;
const THUMB_H: f32 = 84.0;

fn to_color(c: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
}

fn from_color(c: Color32) -> [u8; 4] {
    c.to_srgba_unmultiplied()
}

impl HappyKitten {
    pub fn pool_panel(&mut self, ui: &mut Ui) {
        let frame = egui::Frame::new().fill(CREAM).inner_margin(egui::Margin {
            left: 12,
            right: 6,
            top: 12,
            bottom: 12,
        });
        egui::Panel::left("pool")
            .default_size(400.0)
            .size_range(320.0..=600.0)
            .resizable(true)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                kit::card_frame(PEACH).show(ui, |ui| {
                    ui.set_min_size(ui.available_size());
                    self.pool_tabs(ui);
                    ui.add_space(6.0);
                    match self.tab {
                        PoolTab::Files => self.pool_files(ui, false),
                        PoolTab::Audio => self.pool_audio(ui),
                        PoolTab::Color => self.pool_color(ui),
                        PoolTab::Effects => self.pool_effects(ui),
                        PoolTab::Text => self.pool_text(ui),
                        PoolTab::Graphics => self.pool_graphics(ui),
                    }
                });
            });
    }

    fn pool_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            let w = (ui.available_width() - 30.0) / 6.0;
            for (tab, icon, label) in [
                (PoolTab::Files, Icon::Film, tr("Dateien")),
                (PoolTab::Text, Icon::Text, tr("Text")),
                (PoolTab::Graphics, Icon::Image, tr("Grafik")),
                (PoolTab::Effects, Icon::Sparkle, tr("Blenden")),
                (PoolTab::Color, Icon::Sun, tr("Farbe")),
                (PoolTab::Audio, Icon::Note, tr("Audio")),
            ] {
                let (rect, resp) = ui.allocate_exact_size(vec2(w, 58.0), Sense::click());
                let selected = self.tab == tab;
                let fill = if selected {
                    PINK
                } else if resp.hovered() {
                    PINK_SOFT
                } else {
                    CREAM
                };
                let p = ui.painter();
                p.rect_filled(rect, round(16), fill);
                p.rect_stroke(
                    rect,
                    round(16),
                    Stroke::new(if selected { 2.2 } else { 1.5 }, BROWN),
                    StrokeKind::Inside,
                );
                kit::draw_icon(
                    p,
                    icon,
                    Rect::from_center_size(
                        pos2(rect.center().x, rect.top() + 21.0),
                        vec2(26.0, 26.0),
                    ),
                    BROWN,
                );
                // longer words in other languages shrink to fit the card
                let mut size = 12.5;
                let width = |size: f32| {
                    p.layout_no_wrap(label.to_string(), egui::FontId::proportional(size), BROWN)
                        .size()
                        .x
                };
                let natural = width(size);
                if natural > rect.width() - 8.0 {
                    size = (size * (rect.width() - 8.0) / natural).max(9.0);
                }
                p.text(
                    pos2(rect.center().x, rect.bottom() - 11.0),
                    Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(size),
                    BROWN,
                );
                if resp.clicked() {
                    self.tab = tab;
                    if tab == PoolTab::Audio && !self.selected_sound_clips().is_empty() {
                        self.audio_settings = true;
                    }
                }
            }
        });
    }

    // ----- blends -------------------------------------------------------

    fn pool_effects(&mut self, ui: &mut Ui) {
        ui.label(
            egui::RichText::new(tr("Blende zwischen zwei Clips ziehen."))
                .size(12.5)
                .color(BROWN_SOFT),
        );
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
            let mut custom = self.blend_duration.is_some();
            if ui.checkbox(&mut custom, tr("Eigene Dauer")).changed() {
                self.blend_duration = custom.then_some(1.0);
            }
            if let Some(d) = &mut self.blend_duration {
                ui.spacing_mut().slider_width = 130.0;
                ui.add(
                    egui::Slider::new(d, 0.2..=3.0)
                        .suffix(" s")
                        .fixed_decimals(1),
                );
            } else {
                ui.label(
                    egui::RichText::new(tr("jede Blende mit ihrer eigenen Länge"))
                        .size(12.0)
                        .color(BROWN_SOFT),
                );
            }
        });
        let selected = self.selected_clips();
        if selected.is_empty() {
            ui.label(egui::RichText::new(tr("Tipp: Clips mit dem Rahmen markieren, Blende anklicken = an allen Schnittpunkten.")).size(12.0).color(BROWN_SOFT));
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Clip(s) gewählt: Klick auf eine Blende setzt sie an allen Schnittpunkten der Auswahl.", n = selected.len()))
                    .size(12.5)
                    .color(BROWN),
            );
        }
        ui.add_space(2.0);
        let mut now_hover: Option<TransitionKind> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.label(egui::RichText::new(tr("Katzen-Blenden")).size(16.0));
                self.effect_grid(ui, &TransitionKind::CAT, &mut now_hover);
                ui.add_space(8.0);
                ui.label(egui::RichText::new(tr("Klassische Blenden")).size(16.0));
                self.effect_grid(ui, &TransitionKind::CLASSIC, &mut now_hover);
                for (title, kinds) in [
                    ("Rauschen und Zellen", &TransitionKind::NOISE[..]),
                    ("Wellen und Optik", &TransitionKind::OPTICS[..]),
                    ("Kurven und Formen", &TransitionKind::CURVES[..]),
                    ("Manga-Look", &TransitionKind::MANGA[..]),
                    ("3D und Physik", &TransitionKind::PHYSICS[..]),
                ] {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(tr(title)).size(16.0));
                    self.effect_grid(ui, kinds, &mut now_hover);
                }
            });
        // Hover plays the blend on its card; leaving restores the still preview.
        let prev = self.effect_hover.map(|(k, _)| k);
        if prev != now_hover {
            if let Some(k) = prev {
                self.engine.animate_effect_card(k, k.card_progress());
            }
            self.effect_hover = now_hover.map(|k| (k, Instant::now()));
        }
        if let Some((k, start)) = self.effect_hover {
            let t = (start.elapsed().as_secs_f32() * 0.7) % 1.2;
            self.engine.animate_effect_card(k, t.min(1.0));
            ui.ctx().request_repaint();
        }
    }

    fn effect_grid(
        &mut self,
        ui: &mut Ui,
        kinds: &[TransitionKind],
        now_hover: &mut Option<TransitionKind>,
    ) {
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
                if let Some(tex) = self.engine.effect_card(kind) {
                    egui::Image::new((tex, thumb.size()))
                        .corner_radius(round(10))
                        .paint_at(ui, thumb);
                }
                let p = ui.painter();
                if kind.is_cat() {
                    kit::paw(
                        p,
                        pos2(rect.left() + 17.0, rect.bottom() - 31.0),
                        7.0,
                        PINK_DEEP,
                    );
                } else {
                    p.circle_filled(
                        pos2(rect.left() + 17.0, rect.bottom() - 31.0),
                        5.0,
                        BROWN_SOFT,
                    );
                }
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
                resp.dnd_set_drag_payload(Payload::Effect(kind));
                if resp.clicked() {
                    self.apply_effect_to_selection(kind);
                }
                resp.on_hover_text(tr("Ziehen und zwischen zwei Clips fallen lassen – oder anklicken, um sie an allen Schnittpunkten der Auswahl einzufügen"));
            }
        });
    }

    // ----- text ---------------------------------------------------------

    pub fn text_preset(&self, pick: TextPick) -> Option<TextSpec> {
        if pick.global {
            self.lib.presets.get(pick.index).map(|p| p.spec.clone())
        } else {
            presets::builtin()
                .into_iter()
                .nth(pick.index)
                .map(|(_, s)| s)
        }
    }

    fn pool_text(&mut self, ui: &mut Ui) {
        let selected_text = self
            .selected_clips()
            .into_iter()
            .find(|id| self.doc.project.clip(*id).is_some_and(|(_, c)| c.is_text()));
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if selected_text.is_some() {
                self.preset_row(ui);
            }
            if let Some(id) = selected_text {
                self.text_editor(ui, id);
                ui.add_space(10.0);
                ui.label(egui::RichText::new(tr("Vorlagen")).size(18.0));
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                if kit::pill(ui, Some(Icon::Plus), tr("Text"), true, true).on_hover_text(tr("Neuen Text am Zeiger einfügen")).clicked() {
                    self.add_text_at_playhead(TextSpec::default());
                }
                if kit::pill(ui, Some(Icon::Folder), "SRT · LRC · TXT", false, true)
                    .on_hover_text(tr("Untertitel oder Lyrics importieren: jede Zeile wird ein Text-Clip"))
                    .clicked()
                {
                    self.import_subtitles_dialog();
                }
            });
            ui.label(egui::RichText::new(tr("Vorlage auf eine Spur ziehen oder ＋ anklicken.")).size(12.5).color(BROWN_SOFT));
            ui.add_space(2.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                for (i, (name, spec)) in presets::builtin().into_iter().enumerate() {
                    self.text_card(ui, name, &spec, TextPick { global: false, index: i });
                }
            });
            if !self.lib.presets.is_empty() {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(tr("Meine Vorlagen (global)")).size(16.0));
                let mine: Vec<(String, TextSpec)> = self.lib.presets.iter().map(|p| (p.name.clone(), p.spec.clone())).collect();
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
                    for (i, (name, spec)) in mine.into_iter().enumerate() {
                        self.text_card(ui, &name, &spec, TextPick { global: true, index: i });
                    }
                });
            }
            if selected_text.is_none() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(tr("Wähle einen Text-Clip in der Timeline, um Schrift, Farben und Effekte zu ändern.")).color(BROWN_SOFT));
            }
        });
    }

    /// Insert a text clip at the playhead on the top-most video track that has room.
    pub fn add_text_at_playhead(&mut self, spec: TextSpec) {
        let dur = Tick::from_secs(4.0);
        let at = self.playhead;
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
                    return p.add_text_clip(*t, at, dur, spec.clone());
                }
            }
            match tracks.first() {
                Some(t) => p.add_text_clip(*t, at, dur, spec.clone()),
                None => Err(hk_core::EditError::NoSuchTrack),
            }
        });
        match res {
            Ok(id) => self.selection = Selection::Clip(id),
            Err(e) => self.say(e.to_string()),
        }
    }

    fn text_card(&mut self, ui: &mut Ui, name: &str, spec: &TextSpec, pick: TextPick) {
        let (rect, resp) =
            ui.allocate_exact_size(vec2(CARD_W, THUMB_H + 30.0), Sense::click_and_drag());
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
        p.rect_filled(thumb, round(10), Color32::from_rgb(150, 190, 235));
        p.rect_filled(
            Rect::from_min_max(
                pos2(thumb.left(), thumb.center().y + 10.0),
                thumb.right_bottom(),
            ),
            round(10),
            Color32::from_rgb(120, 190, 120),
        );
        // a little preview of the look: background box, outline (drawn as 8 shifted copies), fill
        let size = 26.0;
        let font = egui::FontId::proportional(size);
        let sample = ellipsize(spec.text.lines().next().unwrap_or("Aa"), 9);
        let g = p.layout_no_wrap(sample, font.clone(), Color32::WHITE);
        let c = thumb.center();
        if spec.background[3] > 0 {
            p.rect_filled(
                Rect::from_center_size(c, g.size() + vec2(16.0, 6.0)),
                round(8),
                to_color(spec.background),
            );
        }
        let pos = c - g.size() / 2.0;
        if spec.outline_width > 0.0 {
            let o = (spec.outline_width * size * 0.5).clamp(1.0, 4.0);
            let og = p.layout_no_wrap(g.text().to_string(), font.clone(), to_color(spec.outline));
            for k in 0..8 {
                let a = k as f32 * std::f32::consts::FRAC_PI_4;
                p.galley(
                    pos + vec2(a.cos(), a.sin()) * o,
                    og.clone(),
                    to_color(spec.outline),
                );
            }
        }
        let fg = p.layout_no_wrap(g.text().to_string(), font, to_color(spec.color));
        p.galley(pos, fg, to_color(spec.color));
        p.text(
            pos2(rect.left() + 10.0, rect.bottom() - 15.0),
            Align2::LEFT_CENTER,
            ellipsize(name, 17),
            egui::FontId::proportional(13.0),
            BROWN,
        );
        let plus = Rect::from_center_size(thumb.left_top() + vec2(18.0, 18.0), vec2(26.0, 26.0));
        if hovered {
            let p = ui.painter();
            p.circle_filled(plus.center(), 13.0, PINK);
            p.circle_stroke(plus.center(), 13.0, ink(1.8));
            kit::draw_icon(
                p,
                Icon::Plus,
                Rect::from_center_size(plus.center(), vec2(15.0, 15.0)),
                BROWN,
            );
        }
        resp.dnd_set_drag_payload(Payload::Text(pick));
        if ((resp.clicked()
            && resp
                .interact_pointer_pos()
                .is_some_and(|p| plus.contains(p)))
            || resp.double_clicked())
            && let Some(spec) = self.text_preset(pick)
        {
            self.add_text_at_playhead(spec);
        }
        // "apply to the selected texts": a button on the card and the same in the context menu
        let targets = self.selected_text_clips();
        let n = targets.len();
        let apply = Rect::from_center_size(
            pos2(rect.right() - 20.0, rect.bottom() - 17.0),
            vec2(26.0, 26.0),
        );
        if n > 0 {
            let hot = ui
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| apply.contains(p));
            let p = ui.painter();
            p.circle_filled(apply.center(), 13.0, if hot { GOLD } else { PINK });
            p.circle_stroke(apply.center(), 13.0, ink(1.8));
            kit::draw_icon(
                p,
                Icon::Check,
                Rect::from_center_size(apply.center(), vec2(15.0, 15.0)),
                BROWN,
            );
            if hot {
                egui::Tooltip::always_open(
                    ui.ctx().clone(),
                    ui.layer_id(),
                    egui::Id::new(("apply-tip", pick.index, pick.global)),
                    egui::PopupAnchor::Pointer,
                )
                .show(|ui| {
                    ui.label(trf!("Auf {n} gewählte Texte anwenden", n = n));
                });
            }
            if resp.clicked()
                && resp
                    .interact_pointer_pos()
                    .is_some_and(|q| apply.contains(q))
            {
                self.apply_text_preset(pick, &targets);
            }
        }
        resp.context_menu(|ui| {
            let label = if n == 1 {
                tr("Auf den gewählten Text anwenden").to_string()
            } else {
                trf!("Auf {n} gewählte Texte anwenden", n = n)
            };
            if ui.add_enabled(n > 0, egui::Button::new(label)).clicked() {
                self.apply_text_preset(pick, &targets);
                ui.close();
            }
            if ui
                .button(tr("Neuen Text mit dieser Vorlage einfügen"))
                .clicked()
            {
                if let Some(spec) = self.text_preset(pick) {
                    self.add_text_at_playhead(spec);
                }
                ui.close();
            }
            if pick.global {
                ui.separator();
                ui.label(tr("Name der Vorlage"));
                let mut changed = false;
                if let Some(p) = self.lib.presets.get_mut(pick.index) {
                    changed = ui.text_edit_singleline(&mut p.name).changed();
                }
                if changed {
                    self.save_library();
                }
                if ui.button(tr("Vorlage löschen")).clicked() {
                    self.lib.remove_preset(pick.index);
                    self.save_library();
                    ui.close();
                }
            }
        });
    }

    /// "Vorlage anwenden": pick a saved or built-in style and put it on all selected texts.
    fn preset_row(&mut self, ui: &mut Ui) {
        let targets = self.selected_text_clips();
        let names: Vec<(TextPick, String)> = presets::builtin()
            .into_iter()
            .enumerate()
            .map(|(i, (n, _))| {
                (
                    TextPick {
                        global: false,
                        index: i,
                    },
                    n.to_string(),
                )
            })
            .chain(self.lib.presets.iter().enumerate().map(|(i, p)| {
                (
                    TextPick {
                        global: true,
                        index: i,
                    },
                    format!("★ {}", p.name),
                )
            }))
            .collect();
        let current = self
            .preset_pick
            .and_then(|pk| names.iter().find(|(k, _)| *k == pk).map(|(_, n)| n.clone()))
            .unwrap_or_else(|| tr("Vorlage wählen …").to_string());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            egui::ComboBox::from_id_salt("preset-pick").selected_text(current).width(190.0).show_ui(ui, |ui| {
                for (pk, name) in &names {
                    ui.selectable_value(&mut self.preset_pick, Some(*pk), name);
                }
            });
            let n = targets.len();
            let label = if n > 1 { trf!("Auf {n} Texte anwenden", n = n) } else { tr("Anwenden").to_string() };
            if kit::pill(ui, Some(Icon::Check), &label, true, self.preset_pick.is_some())
                .on_hover_text(tr("Überträgt die Vorlage auf alle gewählten Texte (Wortlaut und, ohne Haken unten, Position bleiben)"))
                .clicked()
                && let Some(pk) = self.preset_pick
            {
                self.apply_text_preset(pk, &targets);
            }
        });
        ui.checkbox(
            &mut self.preset_with_pos,
            tr("Position der Vorlage mitnehmen"),
        );
        ui.add_space(6.0);
    }

    /// Text clips among the selected clips.
    pub fn selected_text_clips(&self) -> Vec<Id> {
        self.selected_clips()
            .into_iter()
            .filter(|id| self.doc.project.clip(*id).is_some_and(|(_, c)| c.is_text()))
            .collect()
    }

    /// Give each of `targets` the look of a saved or built-in text style (their words stay).
    pub fn apply_text_preset(&mut self, pick: TextPick, targets: &[Id]) {
        let Some(spec) = self.text_preset(pick) else {
            return;
        };
        let with_pos = self.preset_with_pos;
        let n = self
            .doc
            .edit(|p| p.apply_style_to_texts(targets, &spec, with_pos));
        if n == 0 {
            self.say("Wähle zuerst Text-Clips aus");
        } else {
            self.say(trf!("Vorlage auf {n} Text(e) angewendet", n = n));
        }
    }

    fn text_editor(&mut self, ui: &mut Ui, id: Id) {
        let Some(orig) = self.doc.project.clip(id).and_then(|(_, c)| c.text.clone()) else {
            return;
        };
        let track = self.doc.project.clip(id).map(|(t, _)| t.id);
        let mut s = orig.clone();
        let others: Vec<Id> = self
            .selected_clips()
            .into_iter()
            .filter(|o| *o != id && self.doc.project.clip(*o).is_some_and(|(_, c)| c.is_text()))
            .collect();
        if others.is_empty() {
            ui.label(egui::RichText::new(tr("Text bearbeiten")).size(18.0));
        } else {
            ui.label(
                egui::RichText::new(trf!("{n} Texte bearbeiten", n = others.len() + 1)).size(18.0),
            );
            ui.label(
                egui::RichText::new(tr("Schrift, Farben, Position und Effekte gelten für alle gewählten Texte, der Wortlaut nur für den ersten."))
                    .size(12.0)
                    .color(BROWN_SOFT),
            );
        }
        ui.spacing_mut().slider_width = 200.0;
        ui.add(
            egui::TextEdit::multiline(&mut s.text)
                .font(egui::FontId::new(17.0, text_family(&s.font)))
                .desired_rows(3)
                .desired_width(f32::INFINITY)
                .hint_text(tr("Dein Text – jede Zeile bleibt eine Zeile")),
        );
        egui::Grid::new("text-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(tr("Schrift"));
                let current = FONTS.iter().find(|f| f.0 == s.font).map_or("?", |f| f.1);
                egui::ComboBox::from_id_salt("font")
                    .selected_text(
                        egui::RichText::new(tr(current))
                            .family(text_family(&s.font))
                            .size(17.0),
                    )
                    .width(210.0)
                    .show_ui(ui, |ui| {
                        for (fid, label) in FONTS {
                            let ok = hk_text::font_available(fid);
                            ui.add_enabled_ui(ok, |ui| {
                                // every entry is written in its own font, so the list is a preview
                                let text = if ok {
                                    tr(label).to_string()
                                } else {
                                    trf!("{label} (nicht installiert)", label = tr(label))
                                };
                                ui.selectable_value(
                                    &mut s.font,
                                    fid.to_string(),
                                    egui::RichText::new(text)
                                        .family(text_family(fid))
                                        .size(18.0),
                                );
                            });
                        }
                    });
                ui.end_row();
                ui.label(tr("Größe"));
                ui.add(egui::Slider::new(&mut s.size, 0.03..=0.35).show_value(false));
                ui.end_row();
                ui.label(tr("Farbe"));
                ui.horizontal_wrapped(|ui| {
                    let mut c = to_color(s.color);
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut c,
                        egui::color_picker::Alpha::OnlyBlend,
                    )
                    .changed()
                    {
                        s.color = from_color(c);
                    }
                    for sw in presets::PALETTE {
                        let (r, resp) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
                        ui.painter().circle_filled(
                            r.center(),
                            7.0,
                            Color32::from_rgb(sw[0], sw[1], sw[2]),
                        );
                        ui.painter().circle_stroke(r.center(), 7.0, ink(1.0));
                        if resp.clicked() {
                            s.color = [sw[0], sw[1], sw[2], 255];
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Kontur"));
                ui.horizontal(|ui| {
                    let mut c = to_color(s.outline);
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut c,
                        egui::color_picker::Alpha::OnlyBlend,
                    )
                    .changed()
                    {
                        s.outline = from_color(c);
                    }
                    ui.add(egui::Slider::new(&mut s.outline_width, 0.0..=0.3).show_value(false));
                });
                ui.end_row();
                ui.label(tr("Hintergrund"));
                ui.horizontal(|ui| {
                    let mut c = to_color(s.background);
                    if egui::color_picker::color_edit_button_srgba(
                        ui,
                        &mut c,
                        egui::color_picker::Alpha::OnlyBlend,
                    )
                    .changed()
                    {
                        s.background = from_color(c);
                    }
                    if kit::pill(ui, None, tr("ohne"), s.background[3] == 0, true).clicked() {
                        s.background[3] = 0;
                    }
                    ui.checkbox(&mut s.shadow, tr("Schatten"));
                });
                ui.end_row();
                ui.label(tr("Ausrichtung"));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for (a, label) in [
                        (Align::Left, tr("Links")),
                        (Align::Center, tr("Mitte")),
                        (Align::Right, tr("Rechts")),
                    ] {
                        if kit::pill(ui, None, label, s.align == a, true).clicked() {
                            s.align = a;
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Zeilenabstand"));
                ui.add(egui::Slider::new(&mut s.line_spacing, 0.9..=2.0).show_value(false));
                ui.end_row();
                ui.label(tr("Position"));
                ui.horizontal(|ui| {
                    // click or drag in the little frame to place the text
                    let (r, resp) =
                        ui.allocate_exact_size(vec2(66.0, 66.0), Sense::click_and_drag());
                    let p = ui.painter();
                    p.rect_filled(r, round(8), CREAM);
                    p.rect_stroke(r, round(8), ink(1.4), StrokeKind::Inside);
                    for gy in 0..3 {
                        for gx in 0..3 {
                            let c = r.min + vec2(11.0 + gx as f32 * 22.0, 11.0 + gy as f32 * 22.0);
                            p.circle_filled(c, 4.0, PEACH_DEEP);
                        }
                    }
                    let dot = r.min
                        + vec2(
                            s.pos[0].clamp(0.0, 1.0) * 66.0,
                            s.pos[1].clamp(0.0, 1.0) * 66.0,
                        );
                    p.circle_filled(dot, 5.5, PINK_DEEP);
                    p.circle_stroke(dot, 5.5, ink(1.5));
                    if let (true, Some(pos)) = (
                        resp.clicked() || resp.dragged(),
                        resp.interact_pointer_pos(),
                    ) {
                        let q = (pos - r.min) / 66.0;
                        s.pos = [q.x.clamp(0.05, 0.95), q.y.clamp(0.05, 0.95)];
                    }
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut s.pos[0], 0.0..=1.0)
                                .text("x")
                                .show_value(false),
                        );
                        ui.add(
                            egui::Slider::new(&mut s.pos[1], 0.0..=1.0)
                                .text("y")
                                .show_value(false),
                        );
                    });
                });
                ui.end_row();
            });

        ui.add_space(6.0);
        ui.label(egui::RichText::new(tr("Ein- und Ausblenden")).size(16.0));
        egui::Grid::new("fx-grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                ui.label(tr("Einblenden"));
                ui.horizontal(|ui| {
                    fx_combo(ui, "fx-in", &mut s.fx_in);
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
                    fx_combo(ui, "fx-out", &mut s.fx_out);
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
            let key = id.wrapping_mul(0x9E37_79B9) ^ 0x7e57;
            let others = others.clone();
            self.doc.edit_merge(key, |p| {
                if let Some(t) = p.text_of_mut(id) {
                    *t = s.clone();
                }
                // the look follows to the other selected texts, field by field (their words stay)
                for other in others {
                    if let Some(t) = p.text_of_mut(other) {
                        t.apply_diff(&orig, &s);
                    }
                }
            });
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            if kit::pill(ui, Some(Icon::Globe), tr("Als Vorlage speichern"), false, true).on_hover_text(tr("Merkt sich den Stil in der globalen Bibliothek, für alle Projekte")).clicked() {
                let name = trf!("Mein Stil {n}", n = self.lib.presets.len() + 1);
                self.lib.add_preset(name, s.clone());
                self.save_library();
                self.say("Vorlage gespeichert – sie liegt oben unter „Meine Vorlagen“");
            }
            if let Some(track) = track {
                if kit::pill(ui, None, tr("Stil für alle Texte der Spur"), false, true).on_hover_text(tr("Überträgt Schrift, Farben, Position und Effekte auf jeden Text dieser Spur")).clicked() {
                    let style = s.clone();
                    self.doc.edit(|p| p.restyle_track_texts(track, &style));
                }
                if kit::pill(ui, None, tr("Zeilen bunt färben"), false, true).on_hover_text(tr("Jeder Text der Spur bekommt der Reihe nach eine Pastellfarbe")).clicked() {
                    self.doc.edit(|p| {
                        if let Some(t) = p.tracks.iter_mut().find(|t| t.id == track) {
                            for (i, c) in t.clips.iter_mut().filter(|c| c.is_text()).enumerate() {
                                if let Some(spec) = &mut c.text {
                                    let col = presets::PALETTE[1 + i % 9];
                                    spec.color = [col[0], col[1], col[2], 255];
                                }
                            }
                        }
                    });
                }
            }
        });
    }
}

pub(crate) fn fx_combo(ui: &mut Ui, salt: &str, fx: &mut TextFx) {
    egui::ComboBox::from_id_salt(salt)
        .selected_text(tr(fx.name()))
        .width(130.0)
        .show_ui(ui, |ui| {
            for f in TextFx::ALL {
                ui.selectable_value(fx, f, tr(f.name()));
            }
        });
}

pub(crate) fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
