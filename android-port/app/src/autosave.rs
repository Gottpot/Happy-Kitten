//! Crash safety: while a project has unsaved changes it is copied to
//! `<Appordner>/autosave.kitten` every 30 s.
//! A normal quit removes the copy, so one that exists at start-up means the last
//! session ended badly and is offered back.

use hk_core::Document;
use std::path::PathBuf;

pub fn dir() -> Option<PathBuf> {
    crate::platform::data_dir()
}

pub fn path() -> Option<PathBuf> {
    dir().map(|d| d.join("autosave.kitten"))
}

pub fn write(doc: &Document) {
    let (Some(dir), Some(path)) = (dir(), path()) else {
        return;
    };
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = doc.write_copy(&path);
    }
}

pub fn remove() {
    if let Some(p) = path() {
        let _ = std::fs::remove_file(p);
    }
}

/// The unsaved project of a crashed session, if there is one.
pub fn recover() -> Option<Document> {
    let doc = Document::open(&path()?).ok()?;
    doc.project
        .tracks
        .iter()
        .any(|t| !t.clips.is_empty())
        .then_some(doc)
}

/// Writes panics to `crash.log` next to the autosave so they can be reported.
pub fn install_panic_log() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Some(dir) = dir() {
            let _ = std::fs::create_dir_all(&dir);
            let _ = std::fs::write(dir.join("crash.log"), format!("{info}\n"));
        }
        previous(info);
    }));
}
