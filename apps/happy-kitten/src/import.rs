//! Importing: several files or whole folders at once, into this project or the
//! global library, plus subtitle / lyric files that become text clips.

use crate::app::{HappyKitten, PoolTab, Selection};
use crate::i18n::tr;
use crate::trf;
use hk_core::{MediaKind, MediaSource, TextSpec, Tick, TrackKind};
use std::path::{Path, PathBuf};

/// Where imported media go.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scope {
    /// Only this project.
    Project,
    /// The global library: usable from every project.
    Global,
}

const SUBTITLE_EXT: &[&str] = &["srt", "lrc", "txt"];
const MAX_FOLDER_FILES: usize = 500;

pub fn is_subtitle(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SUBTITLE_EXT.contains(&e.to_ascii_lowercase().as_str()))
}

/// Folders become the supported files inside them (two levels deep, sorted by name).
fn expand(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        for p in entries {
            if out.len() >= MAX_FOLDER_FILES
                || p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }
            if p.is_dir() {
                if depth < 2 {
                    walk(&p, depth + 1, out);
                }
            } else if hk_media::kind_of(&p).is_some() || is_subtitle(&p) {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            walk(&p, 0, &mut out);
        } else {
            out.push(p);
        }
    }
    out
}

impl HappyKitten {
    pub fn import_dialog(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title(tr("Dateien importieren (mehrere möglich)"))
            .add_filter(
                tr("Alle unterstützten Dateien"),
                &[
                    "mp4", "mov", "m4v", "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff",
                    "mp3", "wav", "m4a", "aac", "aif", "aiff", "flac", "srt", "lrc", "txt",
                ],
            )
            .add_filter(
                tr("Video und Bilder"),
                &[
                    "mp4", "mov", "m4v", "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff",
                ],
            )
            .add_filter(
                tr("Audio"),
                &["mp3", "wav", "m4a", "aac", "aif", "aiff", "flac"],
            );
        if let Some(paths) = dialog.pick_files() {
            self.import_paths(paths);
        }
    }

    pub fn import_folder_dialog(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title(tr("Ordner importieren"))
            .pick_folder()
        {
            self.import_paths(vec![dir]);
        }
    }

    pub fn import_subtitles_dialog(&mut self) {
        let dialog = rfd::FileDialog::new()
            .set_title(tr("Untertitel oder Lyrics importieren"))
            .add_filter(tr("Untertitel und Lyrics"), SUBTITLE_EXT);
        if let Some(paths) = dialog.pick_files() {
            self.import_paths(paths);
        }
    }

    /// Import everything in `paths` (files and folders). Media go to the current scope,
    /// subtitle files always become text clips in this project.
    pub fn import_paths(&mut self, paths: Vec<PathBuf>) {
        let files = expand(paths);
        let (mut added, mut texts, mut failed) = (0, 0, 0);
        for path in files {
            if is_subtitle(&path) {
                match self.import_subtitle_file(&path) {
                    Ok(n) => texts += n,
                    Err(e) => {
                        failed += 1;
                        self.say(e);
                    }
                }
                continue;
            }
            match self.import_media_file(&path) {
                Ok(()) => added += 1,
                Err(e) => {
                    failed += 1;
                    self.say(e);
                }
            }
        }
        if added + texts > 0 {
            let mut msg = String::new();
            if added > 0 {
                msg.push_str(&if self.scope == Scope::Global {
                    trf!("{n} Datei(en) in die Bibliothek importiert", n = added)
                } else {
                    trf!("{n} Datei(en) ins Projekt importiert", n = added)
                });
            }
            if texts > 0 {
                if !msg.is_empty() {
                    msg.push_str(" · ");
                }
                msg.push_str(&trf!("{n} Textzeilen", n = texts));
            }
            if failed > 0 {
                msg.push_str(&trf!(" · {n} nicht lesbar", n = failed));
            }
            self.say(msg);
        }
    }

    fn import_media_file(&mut self, path: &Path) -> Result<(), String> {
        let probed = match hk_media::kind_of(path) {
            Some(MediaKind::Audio) => hk_audio::probe(path)
                .map(|secs| hk_media::Probe {
                    kind: MediaKind::Audio,
                    duration: Tick::from_secs(secs),
                    size: (0, 0),
                    alpha: false,
                })
                .map_err(hk_media::MediaError::Decode),
            _ => hk_media::probe(path),
        }
        .map_err(|e| {
            format!(
                "{}: {e}",
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            )
        })?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Medium".into());
        if probed.kind == MediaKind::Audio {
            self.tab = PoolTab::Audio;
        }
        match self.scope {
            Scope::Project => {
                self.doc.edit(|p| {
                    let id = p.ensure_media(
                        name,
                        MediaSource::File(path.to_path_buf()),
                        probed.kind,
                        probed.duration,
                        probed.size,
                    );
                    p.set_media_alpha(id, probed.alpha);
                });
            }
            Scope::Global => {
                let id = self.lib.add_item(
                    name,
                    path.to_path_buf(),
                    probed.kind,
                    probed.duration,
                    probed.size,
                );
                self.lib.set_alpha(id, probed.alpha);
                self.save_library();
            }
        }
        Ok(())
    }

    /// SRT / LRC / TXT to text clips on a new video track. Returns the number of lines.
    pub fn import_subtitle_file(&mut self, path: &Path) -> Result<usize, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt")
            .to_ascii_lowercase();
        let cues = hk_core::parse_by_extension(&ext, &text, self.playhead);
        if cues.is_empty() {
            return Err(trf!(
                "In {name} wurde kein Text gefunden",
                name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ));
        }
        let style: TextSpec = if ext == "srt" {
            TextSpec::subtitle()
        } else {
            crate::presets::builtin()
                .into_iter()
                .find(|(n, _)| *n == "Lyrics")
                .map(|(_, s)| s)
                .unwrap_or_default()
        };
        let stem: String = path
            .file_stem()
            .map(|s| s.to_string_lossy().chars().take(16).collect())
            .unwrap_or_default();
        let prefix = match ext.as_str() {
            "srt" => "SRT",
            "lrc" => tr("Lyrics"),
            _ => tr("Text"),
        };
        let title = format!("{prefix} {stem}");
        let result = self.doc.edit(|p| {
            let track = p
                .add_track_of(TrackKind::Video)
                .map_err(|e| e.to_string())?;
            p.rename_track(track, &title);
            Ok::<_, String>(p.import_cues(track, &cues, &style, Tick::ZERO))
        })?;
        if let Some(first) = result.first() {
            self.selection = Selection::Clip(*first);
            self.tab = PoolTab::Text;
        }
        Ok(result.len())
    }

    /// Make a global library entry usable in the project (adds it once) and return its project id.
    pub fn use_lib_item(&mut self, lib_id: hk_core::Id) -> Option<hk_core::Id> {
        let item = self.lib.item(lib_id)?.clone();
        if !item.path.exists() {
            self.say(trf!(
                "{name} fehlt – die Datei wurde verschoben oder gelöscht",
                name = item.name
            ));
            return None;
        }
        Some(self.doc.edit(|p| {
            let id = p.ensure_media(
                item.name,
                MediaSource::File(item.path),
                item.kind,
                item.duration,
                item.size,
            );
            p.set_media_alpha(id, item.alpha);
            id
        }))
    }

    pub fn save_library(&mut self) {
        if self.is_dev_run() {
            return;
        }
        if let Some(dir) = crate::autosave::dir()
            && let Err(e) = self.lib.save(&dir.join("library.json"))
        {
            self.say(format!("Bibliothek konnte nicht gespeichert werden: {e}"));
        }
    }
}
