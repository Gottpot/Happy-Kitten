use crate::audio::AudioEngine;
use crate::engine::Engine;
use crate::export::ExportJob;
use crate::export_ui::ExportOptions;
use crate::i18n::tr;
use crate::import::Scope;
use crate::kit::{self, Icon};
use crate::theme::*;
use crate::trf;
use eframe::egui_wgpu::RenderState;
use hk_core::{
    Document, GlobalLibrary, Id, MediaKind, MediaSource, Project, TextFx, TextSpec, Tick,
    TrackKind, Transition, TransitionKind,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PoolTab {
    Files,
    Text,
    Graphics,
    Effects,
    Audio,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selection {
    None,
    Clip(Id),
    /// The blend in front of this clip.
    Transition(Id),
}

/// What is being dragged around with the mouse.
#[derive(Clone, Copy, Debug)]
pub enum Payload {
    Media(Id),
    /// An entry of the global library (becomes a project media on drop).
    LibMedia(Id),
    /// An image that is to be laid over the video as a graphic.
    Overlay(Id),
    Effect(TransitionKind),
    Text(TextPick),
}

/// Which text look is being dragged: a built-in one or one saved in the library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPick {
    pub global: bool,
    pub index: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gesture {
    None,
    Move {
        clip: Id,
        grab: Tick,
    },
    /// Several selected clips move together; their start times at drag begin are in `group_origins`.
    MoveGroup {
        primary: Id,
        grab: Tick,
    },
    TrimStart(Id),
    TrimEnd(Id),
}

/// A rubber-band selection being dragged on empty timeline space.
#[derive(Clone)]
pub struct Marquee {
    pub start: egui::Pos2,
    /// Keep the selection that existed before (shift / cmd held).
    pub base: Vec<Id>,
}

pub struct HappyKitten {
    pub doc: Document,
    pub engine: Engine,
    pub audio: AudioEngine,
    pub playhead: Tick,
    pub playing: bool,
    pub selection: Selection,
    /// More selected clips besides the one in `selection` (rubber band, shift-click).
    pub extra: std::collections::HashSet<Id>,
    pub marquee: Option<Marquee>,
    pub group_origins: Vec<(Id, Tick)>,
    /// Where inside the trim handle the drag began (so the edge does not jump to the mouse).
    pub trim_grab: Tick,
    /// Length for newly placed blends; `None` = each blend's own default.
    pub blend_duration: Option<f32>,
    /// The text style chosen in the "Vorlage anwenden" list, and whether it also moves the text.
    pub about: bool,
    pub preset_pick: Option<TextPick>,
    pub preset_with_pos: bool,
    pub pool: crate::pool_files::PoolView,
    pub tab: PoolTab,
    pub scope: Scope,
    pub lib: GlobalLibrary,
    pub lib_exists: std::collections::HashMap<Id, (bool, Instant)>,
    /// Timeline zoom in pixels per second.
    pub pps: f32,
    pub snap: bool,
    pub gesture: Gesture,
    pub toast: Option<(String, Instant)>,
    pub logo: egui::TextureHandle,
    pub effect_hover: Option<(TransitionKind, Instant)>,
    pub export_dialog: Option<ExportOptions>,
    pub export_job: Option<ExportJob>,
    /// Developer aid: `HK_SCREENSHOT=out.png` saves the window after a moment and quits.
    shot: Option<(PathBuf, u32)>,
    script: Option<crate::devscript::DevScript>,
    last_autosave: Instant,
}

const LOGO_PNG: &[u8] = include_bytes!("../../../assets/logo.png");

impl HappyKitten {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::i18n::set_lang(crate::i18n::initial());
        theme_init(&cc.egui_ctx);
        let render_state: &RenderState = cc.wgpu_render_state.as_ref().expect("wgpu backend");
        let doc = Document::new(demo_project());
        let engine = Engine::new(render_state, doc.project.size);
        let logo = load_logo(&cc.egui_ctx);
        let mut app = HappyKitten {
            doc,
            engine,
            audio: AudioEngine::new(),
            playhead: Tick::ZERO,
            playing: false,
            selection: Selection::None,
            extra: Default::default(),
            marquee: None,
            group_origins: Vec::new(),
            trim_grab: Tick::ZERO,
            blend_duration: None,
            about: false,
            preset_pick: None,
            preset_with_pos: false,
            pool: Default::default(),
            tab: PoolTab::Files,
            scope: Scope::Project,
            lib: GlobalLibrary::default(),
            lib_exists: Default::default(),
            pps: 70.0,
            snap: true,
            gesture: Gesture::None,
            toast: None,
            logo,
            effect_hover: None,
            export_dialog: None,
            export_job: None,
            shot: std::env::var_os("HK_SCREENSHOT").map(|p| (PathBuf::from(p), 0)),
            script: crate::devscript::DevScript::from_env(),
            last_autosave: Instant::now(),
        };
        if !app.is_dev_run()
            && let Some(dir) = crate::autosave::dir()
        {
            app.lib = GlobalLibrary::load(&dir.join("library.json"));
        }
        // A leftover autosave means the last session did not end cleanly.
        if app.shot.is_none()
            && app.script.is_none()
            && let Some(doc) = crate::autosave::recover()
        {
            app.doc = doc;
            app.say("Nach dem letzten Absturz wiederhergestellt – bitte speichern");
        }
        if app.script.is_some() {
            if std::env::var("HK_SCRIPT").is_ok_and(|n| n == "text" || n == "select") {
                app.doc = Document::new(demo_project());
            } else {
                app.doc = Document::new(demo_media_only());
            }
        }
        if std::env::var_os("HK_GLOBAL").is_some() {
            app.scope = Scope::Global;
        }
        // Developer aid: HK_IMPORT=a.mp4:b.png imports and appends to the timeline.
        if let Some(list) = std::env::var_os("HK_IMPORT") {
            let before = app.doc.project.media.len();
            app.import_paths(std::env::split_paths(&list).collect());
            let new: Vec<Id> = app
                .doc
                .project
                .media
                .iter()
                .skip(before)
                .map(|m| m.id)
                .collect();
            for id in new {
                if let Some(secs) = std::env::var("HK_OVERLAY")
                    .ok()
                    .and_then(|v| v.parse::<f64>().ok())
                {
                    app.playhead = Tick::from_secs(secs);
                    app.add_overlay_at_playhead(id);
                } else {
                    app.append_media(id);
                }
            }
        }
        app
    }

    /// True when a developer hook (screenshot, script) drives the app: nothing is written to disk.
    pub fn is_dev_run(&self) -> bool {
        self.shot.is_some() || self.script.is_some()
    }

    fn shot_frame(&self) -> u32 {
        if self.script.is_some() { 160 } else { 90 }
    }

    fn developer_screenshot(&mut self, ctx: &egui::Context) {
        let Some(shot) = &mut self.shot else { return };
        shot.1 += 1;
        let (path, frames) = (shot.0.clone(), shot.1);
        if let Some(secs) = std::env::var("HK_PLAYHEAD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|_| frames < 3)
        {
            self.playhead = Tick::from_secs(secs);
        }
        if std::env::var_os("HK_GLOBAL").is_some() {
            self.scope = Scope::Global;
        }
        if frames < 3 {
            if let Ok(v) = std::env::var("HK_VIEW") {
                self.pool.mode = match v.as_str() {
                    "list" => crate::pool_files::ViewMode::List,
                    "table" => crate::pool_files::ViewMode::Table,
                    _ => crate::pool_files::ViewMode::Tiles,
                };
            }
            if let Ok(v) = std::env::var("HK_CANVAS")
                && let Some((w, h)) = hk_core::CANVAS_SHAPES
                    .iter()
                    .find(|c| c.0 == v)
                    .map(|c| (c.1, c.2))
            {
                self.doc.project.set_canvas(hk_core::preview_size(w, h));
            }
            if let Ok(v) = std::env::var("HK_FIT") {
                self.doc.project.fit = match v.as_str() {
                    "cover" => hk_core::FitMode::Cover,
                    "blur" => hk_core::FitMode::Blur,
                    _ => hk_core::FitMode::Contain,
                };
            }
        }
        if let Ok(tab) = std::env::var("HK_TAB") {
            self.tab = match tab.as_str() {
                "effects" => PoolTab::Effects,
                "audio" => PoolTab::Audio,
                "text" => PoolTab::Text,
                "graphics" => PoolTab::Graphics,
                _ => PoolTab::Files,
            };
        }
        if frames == 5 && std::env::var_os("HK_SELECT_TEXTS").is_some() {
            let ids: Vec<Id> = self
                .doc
                .project
                .tracks
                .iter()
                .flat_map(|t| t.clips.iter())
                .filter(|c| c.is_text())
                .map(|c| c.id)
                .collect();
            self.select_many(ids);
        }
        if frames == 20
            && let Some(i) = std::env::var("HK_APPLY")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
        {
            let targets = self.selected_text_clips();
            self.apply_text_preset(
                crate::app::TextPick {
                    global: false,
                    index: i,
                },
                &targets,
            );
        }
        if frames == 5 && std::env::var_os("HK_ABOUT").is_some() {
            self.about = true;
        }
        if frames == 5 && std::env::var_os("HK_DIALOG").is_some() {
            self.open_export_dialog();
        }
        if frames == 30 && std::env::var_os("HK_PLAY").is_some() {
            self.toggle_play();
        }
        if frames % 20 == 0 && self.playing {
            eprintln!(
                "play: playhead={:.3}s audio_clock={:?}",
                self.playhead.secs(),
                self.audio.position().map(|t| t.secs())
            );
        }
        if frames == 20
            && let Some(path) = std::env::var_os("HK_EXPORT")
        {
            let opts = ExportOptions {
                res: 0,
                quality: 1,
                hevc: std::env::var_os("HK_HEVC").is_some(),
                format: std::env::var("HK_FORMAT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                fps: std::env::var("HK_FPS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                mbit: std::env::var("HK_MBIT").ok().and_then(|v| v.parse().ok()),
                audio_rate: std::env::var("HK_ARATE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                audio_kbps: 1,
            };
            let settings = opts.settings(
                self.doc.project.size,
                self.doc.project.fps,
                PathBuf::from(path),
            );
            self.start_export(settings);
        }
        if let Some(job) = &self.export_job
            && std::env::var_os("HK_EXPORT").is_some()
            && frames % 30 == 0
        {
            eprintln!("export: {:.0}%", job.fraction() * 100.0);
        }
        if std::env::var_os("HK_EXPORT").is_some() && frames > 25 && self.export_job.is_none() {
            eprintln!(
                "export finished: {:?}",
                self.toast.as_ref().map(|t| t.0.clone())
            );
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if matches!(frames, 12 | 44 | 70 | 150) && std::env::var_os("HK_DUMP").is_some() {
            eprintln!("--- frame {frames}");
            eprintln!("selected: {:?}", self.selected_clips());
            for t in &self.doc.project.tracks {
                for c in &t.clips {
                    eprintln!(
                        "clip {} track={} start={:.2} dur={:.2} in={:.2} transition={:?}",
                        c.id,
                        t.name,
                        c.start.secs(),
                        c.duration.secs(),
                        c.source_in.secs(),
                        c.transition_in.map(|t| (t.kind, t.duration.secs()))
                    );
                }
            }
        }
        if frames == self.shot_frame() && std::env::var_os("HK_EXPORT").is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = image {
            let [w, h] = img.size;
            if let Some(out) = image::RgbaImage::from_raw(w as u32, h as u32, img.as_raw().to_vec())
            {
                out.save(&path).ok();
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }

    pub fn set_selection(&mut self, s: Selection) {
        self.selection = s;
        self.extra.clear();
    }

    /// Every selected clip, in timeline order (earliest start first).
    pub fn selected_clips(&self) -> Vec<Id> {
        let mut ids: Vec<Id> = match self.selection {
            Selection::Clip(id) => std::iter::once(id)
                .chain(self.extra.iter().copied())
                .collect(),
            _ => Vec::new(),
        };
        ids.retain(|id| self.doc.project.clip(*id).is_some());
        ids.sort_by_key(|id| self.doc.project.clip(*id).map(|(_, c)| (c.start, c.id)));
        ids.dedup();
        ids
    }

    pub fn is_selected(&self, id: Id) -> bool {
        self.selection == Selection::Clip(id) || self.extra.contains(&id)
    }

    /// Select exactly these clips; the earliest one becomes the primary.
    pub fn select_many(&mut self, mut ids: Vec<Id>) {
        ids.sort_by_key(|id| self.doc.project.clip(*id).map(|(_, c)| (c.start, c.id)));
        ids.dedup();
        match ids.split_first() {
            None => self.set_selection(Selection::None),
            Some((first, rest)) => {
                self.selection = Selection::Clip(*first);
                self.extra = rest.iter().copied().collect();
            }
        }
    }

    pub fn toggle_selected(&mut self, id: Id) {
        let mut ids = self.selected_clips();
        match ids.iter().position(|x| *x == id) {
            Some(i) => {
                ids.remove(i);
            }
            None => ids.push(id),
        }
        self.select_many(ids);
    }

    pub fn select_all_clips(&mut self) {
        let ids: Vec<Id> = self
            .doc
            .project
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter().map(|c| c.id))
            .collect();
        self.select_many(ids);
    }

    pub fn say(&mut self, msg: impl Into<String>) {
        self.toast = Some((crate::i18n::tr_dyn(&msg.into()), Instant::now()));
    }

    pub fn duration(&self) -> Tick {
        self.doc.project.duration()
    }

    pub fn set_playhead(&mut self, t: Tick) {
        let fps = self.doc.project.fps;
        self.playhead = t
            .clamp(Tick::ZERO, self.duration())
            .snap_to_frame(fps)
            .max(Tick::ZERO);
        if self.playing {
            self.audio.seek(self.playhead);
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.audio.pause();
    }

    pub fn toggle_play(&mut self) {
        if self.playing {
            self.stop();
        } else if self.duration() > Tick::ZERO {
            if self.playhead >= self.duration() {
                self.playhead = Tick::ZERO;
            }
            self.playing = true;
            self.audio.play(self.playhead);
        }
    }

    fn reset_media_caches(&mut self) {
        self.engine.reset();
        self.audio.reset();
    }

    // ----- file actions -------------------------------------------------

    /// True when it is fine to throw the current project away.
    fn confirm_discard(&mut self) -> bool {
        if !self.doc.is_dirty() {
            return true;
        }
        let answer = rfd::MessageDialog::new()
            .set_title(tr("Ungespeicherte Änderungen"))
            .set_description(tr(
                "Das Projekt hat Änderungen, die noch nicht gespeichert sind. Trotzdem fortfahren?",
            ))
            .set_buttons(rfd::MessageButtons::YesNoCancelCustom(
                tr("Speichern").into(),
                tr("Verwerfen").into(),
                tr("Abbrechen").into(),
            ))
            .show();
        match answer {
            rfd::MessageDialogResult::Custom(c) if c == tr("Speichern") => {
                self.save_project(false);
                !self.doc.is_dirty()
            }
            rfd::MessageDialogResult::Custom(c) => c == tr("Verwerfen"),
            _ => false,
        }
    }

    pub fn new_project(&mut self) {
        if !self.confirm_discard() {
            return;
        }
        self.reset_media_caches();
        self.doc = Document::new(demo_media_only());
        self.playhead = Tick::ZERO;
        self.stop();
        self.set_selection(Selection::None);
        self.say("Neues Projekt angelegt");
    }

    pub fn open_project(&mut self) {
        if !self.confirm_discard() {
            return;
        }
        let Some(path) = rfd::FileDialog::new()
            .add_filter(tr("Happy-Kitten-Projekt"), &["kitten"])
            .pick_file()
        else {
            return;
        };
        match Document::open(&path) {
            Ok(doc) => {
                self.reset_media_caches();
                self.doc = doc;
                self.playhead = Tick::ZERO;
                self.stop();
                self.set_selection(Selection::None);
                self.say("Projekt geöffnet");
            }
            Err(e) => self.say(format!("Öffnen fehlgeschlagen: {e}")),
        }
    }

    pub fn save_project(&mut self, ask: bool) {
        let path = match (&self.doc.path, ask) {
            (Some(p), false) => p.clone(),
            _ => {
                let Some(p) = rfd::FileDialog::new()
                    .add_filter(tr("Happy-Kitten-Projekt"), &["kitten"])
                    .set_file_name(format!("{}.kitten", self.doc.project.name))
                    .save_file()
                else {
                    return;
                };
                p
            }
        };
        match self.doc.save(&path) {
            Ok(()) => self.say("Gespeichert ✓"),
            Err(e) => self.say(format!("Speichern fehlgeschlagen: {e}")),
        }
    }

    pub fn save_still(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .set_file_name("standbild.png")
            .save_file()
        else {
            return;
        };
        let (_, (w, h)) = self.engine.preview_texture();
        match self.engine.read_preview().and_then(|px| {
            image::RgbaImage::from_raw(w, h, px).ok_or_else(|| "Bildgröße".to_string())
        }) {
            Ok(img) => match img.save(&path) {
                Ok(()) => self.say("Standbild gespeichert ✓"),
                Err(e) => self.say(format!("Standbild fehlgeschlagen: {e}")),
            },
            Err(e) => self.say(format!("Standbild fehlgeschlagen: {e}")),
        }
    }

    // ----- edit actions -------------------------------------------------

    pub fn split_at_playhead(&mut self) {
        let at = self.playhead;
        let selected = self.selected_clips();
        let n = self.doc.edit(|p| {
            let hit: Vec<Id> = selected
                .iter()
                .copied()
                .filter(|id| {
                    p.clip(*id)
                        .is_some_and(|(_, c)| at > c.start && at < c.end())
                })
                .collect();
            if hit.is_empty() {
                p.split_at(at)
            } else {
                hit.into_iter()
                    .filter(|id| p.split_clip(*id, at).is_ok())
                    .count()
            }
        });
        if n == 0 {
            self.say("Setze den Zeiger auf einen Clip, um ihn zu teilen");
        }
    }

    pub fn delete_selection(&mut self, ripple: bool) {
        match self.selection {
            Selection::Clip(_) => {
                let mut ids = self.selected_clips();
                // later clips first, so closing gaps does not shift the ones still to delete
                ids.reverse();
                self.doc.edit(|p| {
                    for id in ids {
                        p.delete_clip(id, ripple).ok();
                    }
                });
                self.set_selection(Selection::None);
            }
            Selection::Transition(id) => {
                self.doc.edit(|p| p.set_transition(id, None).ok());
                self.set_selection(Selection::Clip(id));
            }
            Selection::None => {}
        }
    }

    pub fn first_track(&self, kind: TrackKind) -> Option<Id> {
        self.doc
            .project
            .tracks
            .iter()
            .find(|t| t.kind == kind)
            .map(|t| t.id)
    }

    /// Append a media item at the end of its matching track.
    pub fn append_media(&mut self, media: Id) {
        let Some(kind) = self.doc.project.media(media).map(|m| {
            if m.kind == MediaKind::Audio {
                TrackKind::Audio
            } else {
                TrackKind::Video
            }
        }) else {
            return;
        };
        let Some(track) = self.first_track(kind) else {
            return;
        };
        let end = self.doc.project.track(track).map_or(Tick::ZERO, |t| {
            t.clips.iter().map(|c| c.end()).fold(Tick::ZERO, Tick::max)
        });
        let res = self.doc.edit(|p| p.add_clip(track, media, end));
        match res {
            Ok(id) => self.set_selection(Selection::Clip(id)),
            Err(e) => self.say(e.to_string()),
        }
    }

    /// A blend of `kind` with the length chosen in the Blenden tab (or its own default).
    pub fn make_transition(&self, kind: TransitionKind) -> Transition {
        match self.blend_duration {
            Some(secs) => Transition::with_duration(kind, Tick::from_secs(secs as f64)),
            None => Transition::new(kind),
        }
    }

    /// Put `kind` on every cut between selected neighbouring clips.
    pub fn apply_effect_to_selection(&mut self, kind: TransitionKind) {
        let selected = self.selected_clips();
        if selected.is_empty() {
            self.say("Wähle erst Clips aus, oder zieh die Blende auf einen Schnittpunkt");
            return;
        }
        let tr = self.make_transition(kind);
        let n = self.doc.edit(|p| p.apply_transition_to_cuts(&selected, tr));
        if n == 0 {
            self.say("Zwischen den gewählten Clips liegt kein Schnittpunkt");
        } else {
            self.say(trf!(
                "{name} an {n} Schnittpunkt(en) eingefügt",
                name = crate::i18n::tr(kind.name()),
                n = n
            ));
        }
    }

    pub fn set_canvas(&mut self, w: u32, h: u32) {
        let size = hk_core::preview_size(w, h);
        if self.doc.project.size != size {
            self.doc.edit(|p| p.set_canvas(size));
        }
    }

    pub fn apply_effect(&mut self, kind: TransitionKind, right_clip: Id) {
        let tr = self.make_transition(kind);
        let res = self.doc.edit(|p| p.set_transition(right_clip, Some(tr)));
        match res {
            Ok(()) => {
                self.set_selection(Selection::Transition(right_clip));
                self.say(trf!(
                    "{name} eingefügt",
                    name = crate::i18n::tr(kind.name())
                ));
            }
            Err(e) => self.say(e.to_string()),
        }
    }

    // ----- per-frame logic ---------------------------------------------

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        use egui::{Key, Modifiers};
        let (space, split, del, undo, redo, save, open, import, left, right, home, end, all) = ctx
            .input_mut(|i| {
                let cmd = Modifiers::COMMAND;
                (
                    i.consume_key(Modifiers::NONE, Key::Space),
                    i.consume_key(Modifiers::NONE, Key::S) || i.consume_key(cmd, Key::B),
                    i.consume_key(Modifiers::NONE, Key::Delete)
                        || i.consume_key(Modifiers::NONE, Key::Backspace),
                    i.consume_key(cmd, Key::Z),
                    i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z),
                    i.consume_key(cmd, Key::S),
                    i.consume_key(cmd, Key::O),
                    i.consume_key(cmd, Key::I),
                    i.consume_key(Modifiers::NONE, Key::ArrowLeft),
                    i.consume_key(Modifiers::NONE, Key::ArrowRight),
                    i.consume_key(Modifiers::NONE, Key::Home),
                    i.consume_key(Modifiers::NONE, Key::End),
                    i.consume_key(Modifiers::COMMAND, Key::A),
                )
            });
        let fps = self.doc.project.fps;
        let frame = Tick::from_frames(1, fps);
        if space {
            self.toggle_play();
        }
        if split {
            self.split_at_playhead();
        }
        if del {
            let ripple = ctx.input(|i| i.modifiers.shift);
            self.delete_selection(ripple);
        }
        // Shift+Cmd+Z must win over Cmd+Z, which also matches it in egui.
        if redo {
            self.doc.redo();
        } else if undo {
            self.doc.undo();
        }
        if save {
            self.save_project(false);
        }
        if open {
            self.open_project();
        }
        if import {
            self.import_dialog();
        }
        if left {
            self.stop();
            self.set_playhead(self.playhead - frame);
        }
        if right {
            self.stop();
            self.set_playhead(self.playhead + frame);
        }
        if home {
            self.set_playhead(Tick::ZERO);
        }
        if end {
            self.set_playhead(self.duration());
        }
        if all {
            self.select_all_clips();
        }
    }

    /// Copy unsaved work aside every 30 s (see `autosave`).
    fn autosave_tick(&mut self) {
        if self.shot.is_some()
            || self.script.is_some()
            || self.last_autosave.elapsed() < Duration::from_secs(30)
        {
            return;
        }
        self.last_autosave = Instant::now();
        if self.doc.is_dirty() {
            crate::autosave::write(&self.doc);
        }
    }

    fn tick_playback(&mut self, ctx: &egui::Context) {
        if !self.playing {
            return;
        }
        // The audio clock is the master when there is an output device.
        match self.audio.position() {
            Some(t) => self.playhead = t,
            None => {
                let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.1) as f64;
                self.playhead += Tick::from_secs(dt);
            }
        }
        if self.playhead >= self.duration() {
            self.playhead = self.duration();
            self.stop();
        }
        ctx.request_repaint();
    }

    fn dropped_files(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        if !files.is_empty() {
            self.import_paths(files);
        }
    }

    /// Globe + country code of the current language; opens a list of all languages.
    fn language_button(&mut self, ui: &mut egui::Ui) {
        let current = crate::i18n::lang();
        let resp = kit::pill(ui, Some(Icon::Globe), current.code(), false, true)
            .on_hover_text(tr("Sprache"));
        egui::Popup::menu(&resp).show(|ui| {
            ui.set_min_width(150.0);
            for l in crate::i18n::Lang::ALL {
                let text = format!("{}   {}", l.code(), l.native_name());
                if ui.selectable_label(l == current, text).clicked() {
                    crate::i18n::set_lang(l);
                    crate::i18n::save(l);
                    ui.close();
                }
            }
        });
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        let frame = egui::Frame::new()
            .fill(PINK)
            .inner_margin(egui::Margin::symmetric(14, 8));
        egui::Panel::top("header")
            .exact_size(78.0)
            .show_separator_line(false)
            .frame(frame)
            .show(ui, |ui| {
                let full = ui.max_rect();
                let p = ui.painter().clone();
                p.line_segment(
                    [
                        full.left_bottom() + egui::vec2(-20.0, 7.0),
                        full.right_bottom() + egui::vec2(20.0, 7.0),
                    ],
                    ink(2.5),
                );
                // centre: kitten logo, name, yarn ball
                let title = format!(
                    "Happy-Kitten v{}",
                    env!("CARGO_PKG_VERSION")
                        .rsplit_once('.')
                        .map_or("1.0", |(a, _)| a)
                );
                let font = heading_font(34.0);
                let galley = p.layout_no_wrap(title, font, BROWN);
                let cy = full.center().y - 2.0;
                let tw = galley.size().x;
                let cx = full.center().x;
                // cat badge
                let badge = egui::Rect::from_center_size(
                    egui::pos2(cx - tw / 2.0 - 46.0, cy),
                    egui::vec2(60.0, 56.0),
                );
                p.rect_filled(badge.expand(3.0), round(18), CREAM);
                p.rect_stroke(
                    badge.expand(3.0),
                    round(18),
                    ink(2.0),
                    egui::StrokeKind::Inside,
                );
                egui::Image::new(&self.logo)
                    .uv(egui::Rect::from_min_max(
                        egui::pos2(0.215, 0.07),
                        egui::pos2(0.795, 0.60),
                    ))
                    .corner_radius(round(15))
                    .paint_at(ui, badge);
                // shadow + text
                let text_pos = egui::pos2(cx - tw / 2.0, cy - galley.size().y / 2.0);
                p.galley_with_override_text_color(
                    text_pos + egui::vec2(2.0, 2.5),
                    galley.clone(),
                    CREAM,
                );
                p.galley(text_pos, galley, BROWN);
                kit::yarn_ball(&p, egui::pos2(cx + tw / 2.0 + 40.0, cy + 2.0), 22.0);
                let about_hit = egui::Rect::from_min_max(
                    egui::pos2(badge.left() - 6.0, cy - 34.0),
                    egui::pos2(cx + tw / 2.0 + 68.0, cy + 34.0),
                );
                if ui
                    .interact(about_hit, egui::Id::new("about-hit"), egui::Sense::click())
                    .on_hover_text(tr("Über Happy-Kitten"))
                    .clicked()
                {
                    self.about = true;
                }

                // left: file buttons (room for the macOS traffic lights)
                let left = egui::Rect::from_min_size(
                    egui::pos2(full.left() + 86.0, full.center().y - 20.0),
                    egui::vec2(430.0, 40.0),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
                    ui.horizontal_centered(|ui| {
                        if kit::pill(ui, Some(Icon::New), tr("Neu"), false, true).clicked() {
                            self.new_project();
                        }
                        if kit::pill(ui, Some(Icon::Folder), tr("Öffnen"), false, true)
                            .on_hover_text("Cmd+O")
                            .clicked()
                        {
                            self.open_project();
                        }
                        if kit::pill(ui, Some(Icon::Save), tr("Speichern"), false, true)
                            .on_hover_text("Cmd+S")
                            .clicked()
                        {
                            self.save_project(false);
                        }
                    });
                });
                // right
                let right = egui::Rect::from_min_size(
                    egui::pos2(full.right() - 580.0, full.center().y - 20.0),
                    egui::vec2(566.0, 40.0),
                );
                ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        self.language_button(ui);
                        if kit::pill(ui, Some(Icon::Film), tr("Exportieren"), true, true)
                            .on_hover_text(tr("Film als MP4 speichern"))
                            .clicked()
                        {
                            self.open_export_dialog();
                        }
                        if kit::pill(ui, Some(Icon::Export), tr("Standbild"), false, true)
                            .on_hover_text(tr("Aktuelles Vorschaubild als PNG speichern"))
                            .clicked()
                        {
                            self.save_still();
                        }
                        if kit::pill(ui, Some(Icon::Plus), tr("Import"), false, true)
                            .on_hover_text("Cmd+I")
                            .clicked()
                        {
                            self.import_dialog();
                        }
                    });
                });
            });
    }
}

impl eframe::App for HappyKitten {
    fn on_exit(&mut self) {
        // A clean quit: nothing to recover next time.
        crate::autosave::remove();
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if let Some(script) = &mut self.script {
            raw_input.events.extend(script.events());
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested())
            && self.shot.is_none()
            && !self.confirm_discard()
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        self.dropped_files(ctx);
        self.handle_keys(ctx);
        self.tick_playback(ctx);
        self.autosave_tick();
        if self
            .toast
            .as_ref()
            .is_some_and(|(_, t)| t.elapsed() > Duration::from_secs(4))
        {
            self.toast = None;
        }
        // keep the playhead inside the project after edits
        if self.playhead > self.duration() {
            self.playhead = self.duration();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        for m in self
            .doc
            .project
            .media
            .iter()
            .map(|m| m.id)
            .collect::<Vec<_>>()
        {
            self.engine.want_thumb(&self.doc.project, m);
        }
        if self.audio.sync(&self.doc.project) {
            ctx.request_repaint();
        }
        if let Some(e) = self.audio.errors.pop() {
            self.say(e);
        }
        self.engine.ensure_size(self.doc.project.size);
        let loading = self.engine.pump(&ctx);
        self.engine.render(&self.doc.project, self.playhead);
        if self.playing {
            self.engine.prefetch(&self.doc.project, self.playhead, 4);
        }
        if loading {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if let Some(err) = self.engine.errors.pop() {
            self.say(err);
        }

        self.header(ui);
        self.timeline_panel(ui);
        self.pool_panel(ui);
        self.preview_panel(ui);
        self.drag_ghost(&ctx);
        self.export_ui(&ctx);
        self.about_ui(&ctx);
        self.developer_screenshot(&ctx);

        let title = format!(
            "{} – Happy-Kitten{}",
            self.doc.project.name,
            if self.doc.is_dirty() { " •" } else { "" }
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}

fn theme_init(ctx: &egui::Context) {
    crate::theme::apply(ctx);
}

fn load_logo(ctx: &egui::Context) -> egui::TextureHandle {
    let img = image::load_from_memory(LOGO_PNG)
        .map(|i| i.to_rgba8())
        .unwrap_or_else(|_| image::RgbaImage::from_pixel(8, 8, image::Rgba([255, 190, 207, 255])));
    let color = egui::ColorImage::from_rgba_unmultiplied(
        [img.width() as usize, img.height() as usize],
        img.as_raw(),
    );
    ctx.load_texture("logo", color, egui::TextureOptions::LINEAR)
}

/// Project with the four demo scenes in the pool but nothing on the timeline.
pub fn demo_media_only() -> Project {
    let mut p = Project::default();
    for i in 0..hk_media::SAMPLE_COUNT {
        p.add_media(
            hk_media::sample_name(i).to_string(),
            MediaSource::Sample(i),
            MediaKind::Image,
            Tick::from_secs(5.0),
            (1280, 720),
        );
    }
    p
}

/// First-run project: three demo clips joined by two paw transitions.
pub fn demo_project() -> Project {
    let mut p = demo_media_only();
    let Some(track) = p
        .tracks
        .iter()
        .find(|t| t.kind == TrackKind::Video)
        .map(|t| t.id)
    else {
        return p;
    };
    let media: Vec<Id> = p.media.iter().map(|m| m.id).collect();
    let mut clips = Vec::new();
    for (i, m) in media.iter().take(3).enumerate() {
        if let Ok(id) = p.add_clip(track, *m, Tick::from_secs(i as f64 * 5.0)) {
            clips.push(id);
        }
    }
    if let Some(&c) = clips.get(1) {
        p.set_transition(c, Some(Transition::new(TransitionKind::ClawWipe)))
            .ok();
    }
    if let Some(&c) = clips.get(2) {
        p.set_transition(c, Some(Transition::new(TransitionKind::PawSpin)))
            .ok();
    }
    if let Ok(top) = p.add_track_of(TrackKind::Video) {
        let title = TextSpec {
            text: "Happy-Kitten".into(),
            size: 0.16,
            pos: [0.5, 0.4],
            fx_in: TextFx::Pixelate,
            fx_in_dur: Tick::from_secs(1.0),
            fx_out: TextFx::Swirl,
            fx_out_dur: Tick::from_secs(1.0),
            ..Default::default()
        };
        p.add_text_clip(top, Tick::from_secs(0.5), Tick::from_secs(4.0), title)
            .ok();
    }
    p
}
