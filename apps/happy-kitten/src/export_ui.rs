//! "Film exportieren" dialog and the progress window.

use crate::app::HappyKitten;
use crate::export::{self, ExportJob, ExportSettings};
use crate::i18n::tr;
use crate::kit::{self, Icon};
use crate::theme::*;
use crate::trf;
use hk_media::Codec;

pub const RESOLUTIONS: [(&str, u32); 3] = [
    ("HD · 720 Pixel kurze Seite", 720),
    ("Full HD · 1080", 1080),
    ("4K · 2160", 2160),
];
pub const QUALITIES: [(&str, f64); 3] = [
    ("Gut (kleine Datei)", 0.6),
    ("Sehr gut", 1.0),
    ("Beste Qualität", 1.7),
];

/// Shapes for the export: "wie Projekt" first, then fixed ones (width : height).
pub const FORMATS: [(&str, u32, u32); 5] = [
    ("Wie das Projekt", 0, 0),
    ("16:9 Querformat", 16, 9),
    ("9:16 Short · Reels · TikTok", 9, 16),
    ("1:1 Quadrat", 1, 1),
    ("4:5 Hochformat", 4, 5),
];

/// Frame rates for the film: (label, numerator, denominator); the first entry follows the project.
pub const FPS_CHOICES: [(&str, u32, u32); 9] = [
    ("Wie das Projekt", 0, 1),
    ("23,976 (Kino, NTSC)", 24000, 1001),
    ("24 (Kino, Midjourney)", 24, 1),
    ("25 (PAL)", 25, 1),
    ("29,97 (NTSC)", 30000, 1001),
    ("30", 30, 1),
    ("50", 50, 1),
    ("59,94", 60000, 1001),
    ("60", 60, 1),
];
/// AAC (as the Mac encodes it) only allows rates up to 48 kHz.
pub const AUDIO_RATES: [(&str, u32); 3] = [
    ("48 kHz (Standard für Video)", 48_000),
    ("44,1 kHz (Musik, CD)", 44_100),
    ("32 kHz (klein, Sprache)", 32_000),
];
pub const AUDIO_KBPS: [u32; 4] = [128, 192, 256, 320];

#[derive(Clone)]
pub struct ExportOptions {
    /// Length of the shorter side: 720, 1080 or 2160 pixels.
    pub res: usize,
    pub quality: usize,
    pub hevc: bool,
    pub format: usize,
    /// Index into [`FPS_CHOICES`].
    pub fps: usize,
    /// Video bit rate in Mbit/s chosen by hand; `None` = automatic from the quality.
    pub mbit: Option<f32>,
    /// Index into [`AUDIO_RATES`] and [`AUDIO_KBPS`].
    pub audio_rate: usize,
    pub audio_kbps: usize,
}

impl Default for ExportOptions {
    fn default() -> Self {
        ExportOptions {
            res: 1,
            quality: 1,
            hevc: false,
            format: 0,
            fps: 0,
            mbit: None,
            audio_rate: 0,
            audio_kbps: 1,
        }
    }
}

impl ExportOptions {
    /// Pixel size: the shorter side is the chosen resolution (so 9:16 at 1080 is 1080×1920).
    pub fn size(&self, project_size: (u32, u32)) -> (u32, u32) {
        let short = RESOLUTIONS.get(self.res).map_or(1080, |r| r.1) as f64;
        let (fw, fh) = match FORMATS.get(self.format) {
            Some((_, w, h)) if *w > 0 => (*w as f64, *h as f64),
            _ => (project_size.0.max(1) as f64, project_size.1.max(1) as f64),
        };
        let (w, h) = if fw >= fh {
            (short * fw / fh, short)
        } else {
            (short, short * fh / fw)
        };
        (
            ((w.round() as u32) & !1).max(2),
            ((h.round() as u32) & !1).max(2),
        )
    }

    /// The frame rate of the film.
    pub fn film_fps(&self, project_fps: hk_core::Fps) -> hk_core::Fps {
        match FPS_CHOICES.get(self.fps) {
            Some((_, num, den)) if *num > 0 => hk_core::Fps {
                num: *num,
                den: *den,
            },
            _ => project_fps,
        }
    }

    /// Video bit rate in bit/s: the hand-set value, or about 10 Mbit/s for 1080p30 at "Sehr gut",
    /// scaled by picture area and frame rate.
    pub fn bitrate(&self, size: (u32, u32), fps: hk_core::Fps) -> u32 {
        if let Some(m) = self.mbit.filter(|m| m.is_finite()) {
            return (m.clamp(0.5, 300.0) as f64 * 1_000_000.0) as u32;
        }
        let area = size.0 as f64 * size.1 as f64 / (1920.0 * 1080.0);
        let q = QUALITIES.get(self.quality).map_or(1.0, |q| q.1);
        let codec = if self.hevc { 0.65 } else { 1.0 };
        let rate = (fps.as_f64() / 30.0).clamp(0.7, 2.0);
        (10_000_000.0 * area * q * codec * rate).clamp(1_000_000.0, 120_000_000.0) as u32
    }

    pub fn audio_rate_hz(&self) -> u32 {
        AUDIO_RATES.get(self.audio_rate).map_or(48_000, |r| r.1)
    }

    pub fn audio_bits(&self) -> u32 {
        AUDIO_KBPS.get(self.audio_kbps).map_or(192, |k| *k) * 1000
    }

    /// Everything the export worker needs, for a project of this shape and frame rate.
    pub fn settings(
        &self,
        project_size: (u32, u32),
        project_fps: hk_core::Fps,
        path: std::path::PathBuf,
    ) -> ExportSettings {
        let size = self.size(project_size);
        let fps = self.film_fps(project_fps);
        ExportSettings {
            width: size.0,
            height: size.1,
            bitrate: self.bitrate(size, fps),
            codec: if self.hevc { Codec::Hevc } else { Codec::H264 },
            path,
            fps,
            audio_rate: self.audio_rate_hz(),
            audio_bitrate: self.audio_bits(),
        }
    }
}

impl HappyKitten {
    pub fn open_export_dialog(&mut self) {
        if self.export_job.is_some() {
            return;
        }
        if self.duration().0 == 0 {
            self.say("Die Timeline ist leer – zieh erst etwas hinein");
            return;
        }
        self.stop();
        self.export_dialog = Some(ExportOptions::default());
    }

    fn begin_export(&mut self, opts: &ExportOptions) {
        if self.audio.busy(&self.doc.project) {
            self.say("Der Ton wird noch geladen – gleich nochmal versuchen");
            return;
        }
        let Some(mut path) = rfd::FileDialog::new()
            .add_filter(tr("MP4-Video"), &["mp4"])
            .set_file_name(format!("{}.mp4", self.doc.project.name))
            .save_file()
        else {
            return;
        };
        if path.extension().is_none() {
            path.set_extension("mp4");
        }
        let settings = opts.settings(self.doc.project.size, self.doc.project.fps, path);
        self.start_export(settings);
    }

    pub fn start_export(&mut self, settings: ExportSettings) {
        let (device, queue) = self.engine.gpu();
        let project = self.doc.project.clone();
        let plan = self.audio.plan(&project);
        let pcm = self.audio.snapshot();
        self.export_job = Some(export::start(project, plan, pcm, device, queue, settings));
        self.export_dialog = None;
    }

    pub fn export_ui(&mut self, ctx: &egui::Context) {
        if let Some(mut opts) = self.export_dialog.clone() {
            let mut keep_open = true;
            let mut go = false;
            let mut fit = self.doc.project.fit;
            let size = opts.size(self.doc.project.size);
            let secs = self.duration().secs();
            let film_fps = opts.film_fps(self.doc.project.fps);
            let video_bits = opts.bitrate(size, film_fps);
            let est_mb = (video_bits + opts.audio_bits()) as f64 / 8.0 * secs / 1_000_000.0;
            egui::Window::new(tr("Film exportieren"))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(kit::card_frame(CREAM).inner_margin(egui::Margin::same(20)))
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.horizontal(|ui| {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                        kit::paw(ui.painter(), r.center(), 14.0, PINK_DEEP);
                        ui.label(egui::RichText::new(tr("Film exportieren")).size(24.0));
                    });
                    ui.add_space(8.0);
                    egui::Grid::new("export-grid")
                        .num_columns(2)
                        .spacing([16.0, 12.0])
                        .show(ui, |ui| {
                            ui.label(tr("Format"));
                            egui::ComboBox::from_id_salt("exp-shape")
                                .selected_text(tr(FORMATS[opts.format].0))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, (name, _, _)) in FORMATS.iter().enumerate() {
                                        ui.selectable_value(&mut opts.format, i, tr(name));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Bild anpassen"));
                            egui::ComboBox::from_id_salt("exp-fit")
                                .selected_text(tr(fit.name()))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for f in hk_core::FitMode::ALL {
                                        ui.selectable_value(&mut fit, f, tr(f.name()));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Auflösung"));
                            egui::ComboBox::from_id_salt("exp-res")
                                .selected_text(tr(RESOLUTIONS[opts.res].0))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, (name, _)) in RESOLUTIONS.iter().enumerate() {
                                        ui.selectable_value(&mut opts.res, i, tr(name));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Bildrate"));
                            egui::ComboBox::from_id_salt("exp-fps")
                                .selected_text(if opts.fps == 0 {
                                    trf!(
                                        "Wie das Projekt ({fps})",
                                        fps = fps_label(self.doc.project.fps)
                                    )
                                } else {
                                    tr(FPS_CHOICES[opts.fps].0).to_string()
                                })
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, (name, _, _)) in FPS_CHOICES.iter().enumerate() {
                                        ui.selectable_value(&mut opts.fps, i, tr(name));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Qualität"));
                            egui::ComboBox::from_id_salt("exp-q")
                                .selected_text(tr(QUALITIES[opts.quality].0))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, (name, _)) in QUALITIES.iter().enumerate() {
                                        ui.selectable_value(&mut opts.quality, i, tr(name));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Bitrate"));
                            ui.horizontal(|ui| {
                                let mut custom = opts.mbit.is_some();
                                if ui.checkbox(&mut custom, tr("selbst festlegen")).changed() {
                                    opts.mbit = custom.then(|| {
                                        (video_bits as f32 / 1_000_000.0).round().max(1.0)
                                    });
                                }
                                match &mut opts.mbit {
                                    Some(m) => {
                                        ui.spacing_mut().slider_width = 110.0;
                                        ui.add(
                                            egui::Slider::new(m, 1.0..=150.0)
                                                .logarithmic(true)
                                                .suffix(" Mbit/s")
                                                .fixed_decimals(0),
                                        );
                                    }
                                    None => {
                                        ui.label(
                                            egui::RichText::new(trf!(
                                                "automatisch ≈ {mbit} Mbit/s",
                                                mbit = format!("{:.0}", video_bits as f64 / 1e6)
                                            ))
                                            .color(BROWN_SOFT),
                                        );
                                    }
                                }
                            });
                            ui.end_row();
                            ui.label(tr("Audio"));
                            egui::ComboBox::from_id_salt("exp-arate")
                                .selected_text(tr(AUDIO_RATES[opts.audio_rate].0))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, (name, _)) in AUDIO_RATES.iter().enumerate() {
                                        ui.selectable_value(&mut opts.audio_rate, i, tr(name));
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Audio-Bitrate"));
                            egui::ComboBox::from_id_salt("exp-akbps")
                                .selected_text(format!("{} kbit/s", AUDIO_KBPS[opts.audio_kbps]))
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    for (i, k) in AUDIO_KBPS.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut opts.audio_kbps,
                                            i,
                                            format!("{k} kbit/s"),
                                        );
                                    }
                                });
                            ui.end_row();
                            ui.label(tr("Datei"));
                            egui::ComboBox::from_id_salt("exp-fmt")
                                .selected_text(if opts.hevc {
                                    tr("MP4 · HEVC (kleiner)")
                                } else {
                                    tr("MP4 · H.264 (läuft überall)")
                                })
                                .width(220.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut opts.hevc,
                                        false,
                                        tr("MP4 · H.264 (läuft überall)"),
                                    );
                                    ui.selectable_value(
                                        &mut opts.hevc,
                                        true,
                                        tr("MP4 · HEVC (kleiner)"),
                                    );
                                });
                            ui.end_row();
                        });
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(trf!(
                            "{w}×{h} · {fps} Bilder/s · {secs} s · ca. {mb} MB",
                            w = size.0,
                            h = size.1,
                            fps = fps_label(film_fps),
                            secs = format!("{secs:.0}"),
                            mb = format!("{est_mb:.0}")
                        ))
                        .color(BROWN_SOFT)
                        .size(14.0),
                    );
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if kit::pill(
                            ui,
                            Some(Icon::Export),
                            tr("Speicherort wählen & los"),
                            true,
                            true,
                        )
                        .clicked()
                        {
                            go = true;
                        }
                        if kit::pill(ui, None, tr("Abbrechen"), false, true).clicked() {
                            keep_open = false;
                        }
                    });
                });
            if fit != self.doc.project.fit {
                self.doc.edit(|p| p.fit = fit);
            }
            if go {
                self.begin_export(&opts);
            } else if keep_open {
                self.export_dialog = Some(opts);
            } else {
                self.export_dialog = None;
            }
        }

        if self.export_job.is_some() {
            self.export_progress(ctx);
        }
    }

    fn export_progress(&mut self, ctx: &egui::Context) {
        let mut finished: Option<Result<(), String>> = None;
        let mut cancel = false;
        if let Some(job) = &self.export_job {
            finished = job.poll();
            let frac = job.fraction();
            let elapsed = job.started.elapsed().as_secs_f32();
            let eta = if frac > 0.02 {
                elapsed * (1.0 - frac) / frac
            } else {
                0.0
            };
            egui::Window::new(tr("Export läuft"))
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(kit::card_frame(CREAM).inner_margin(egui::Margin::same(20)))
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.label(egui::RichText::new(tr("Das Kätzchen rendert …")).size(22.0));
                    ui.add_space(8.0);
                    ui.add(
                        egui::ProgressBar::new(frac)
                            .fill(PINK_DEEP)
                            .show_percentage()
                            .animate(true),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(format!(
                            "{}{}",
                            trf!("{secs} s vergangen", secs = format!("{elapsed:.0}")),
                            if eta > 0.0 {
                                trf!(" · noch ca. {secs} s", secs = format!("{eta:.0}"))
                            } else {
                                String::new()
                            }
                        ))
                        .color(BROWN_SOFT),
                    );
                    ui.add_space(8.0);
                    if kit::pill(ui, None, tr("Abbrechen"), false, true).clicked() {
                        cancel = true;
                    }
                });
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if cancel && let Some(job) = &self.export_job {
            job.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(result) = finished {
            let path = self.export_job.take().map(|j: ExportJob| j.settings.path);
            match (result, path) {
                (Ok(()), Some(p)) => {
                    self.say(trf!(
                        "Export fertig ✓ {name}",
                        name = p
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default()
                    ));
                    if std::env::var_os("HK_EXPORT").is_none() {
                        export::reveal_in_finder(&p);
                    }
                }
                (Err(e), _) => self.say(format!("Export: {e}")),
                _ => {}
            }
        }
    }
}

/// "24", "23,976", "29,97" ...
pub fn fps_label(fps: hk_core::Fps) -> String {
    let v = fps.as_f64();
    if (v - v.round()).abs() < 0.001 {
        format!("{}", v.round() as u32)
    } else {
        format!("{v:.3}").trim_end_matches('0').replace('.', ",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hk_core::Fps;

    #[test]
    fn fps_follows_project_unless_chosen() {
        let mut o = ExportOptions::default();
        assert_eq!(o.film_fps(Fps::FPS_30), Fps::FPS_30);
        o.fps = 2;
        assert_eq!(o.film_fps(Fps::FPS_30), Fps { num: 24, den: 1 });
        o.fps = 1;
        assert_eq!(
            o.film_fps(Fps::FPS_30),
            Fps {
                num: 24000,
                den: 1001
            }
        );
    }

    #[test]
    fn bitrate_is_automatic_or_by_hand_and_scales_with_size_and_rate() {
        let mut o = ExportOptions::default();
        let hd = o.bitrate((1920, 1080), Fps::FPS_30);
        assert!((9_000_000..=11_000_000).contains(&hd));
        assert!(o.bitrate((3840, 2160), Fps::FPS_30) > hd * 3);
        assert!(o.bitrate((1920, 1080), Fps::FPS_60) > hd);
        assert!(o.bitrate((1920, 1080), Fps { num: 24, den: 1 }) < hd);
        o.mbit = Some(5.0);
        assert_eq!(o.bitrate((1920, 1080), Fps::FPS_30), 5_000_000);
        o.mbit = Some(f32::NAN);
        assert!(
            o.bitrate((1920, 1080), Fps::FPS_30) >= 1_000_000,
            "nonsense input falls back to automatic"
        );
    }

    #[test]
    fn audio_choices_and_labels() {
        let mut o = ExportOptions::default();
        assert_eq!(o.audio_rate_hz(), 48_000);
        o.audio_rate = 1;
        assert_eq!(o.audio_rate_hz(), 44_100);
        o.audio_kbps = 3;
        assert_eq!(o.audio_bits(), 320_000);
        assert_eq!(fps_label(Fps { num: 24, den: 1 }), "24");
        assert_eq!(
            fps_label(Fps {
                num: 24000,
                den: 1001
            }),
            "23,976"
        );
        assert_eq!(
            fps_label(Fps {
                num: 30000,
                den: 1001
            }),
            "29,97"
        );
    }
}
