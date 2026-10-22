use crate::project::Project;
use std::path::{Path, PathBuf};

const HISTORY_LIMIT: usize = 200;

/// A project plus undo/redo and the file it was loaded from.
#[derive(Default)]
pub struct Document {
    pub project: Project,
    pub path: Option<PathBuf>,
    undo: Vec<Project>,
    redo: Vec<Project>,
    dirty: bool,
    merge: Option<(u64, std::time::Instant)>,
}

impl Document {
    pub fn new(project: Project) -> Self {
        Document {
            project,
            ..Default::default()
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Run one undoable edit. The snapshot is dropped again if nothing changed.
    pub fn edit<R>(&mut self, f: impl FnOnce(&mut Project) -> R) -> R {
        self.begin_gesture();
        let out = f(&mut self.project);
        self.end_gesture();
        out
    }

    /// An edit that is folded into the previous undo step when it has the same `key` and follows
    /// within a second (typing, dragging a slider): one Undo takes back the whole burst.
    pub fn edit_merge<R>(&mut self, key: u64, f: impl FnOnce(&mut Project) -> R) -> R {
        let merge = self
            .merge
            .is_some_and(|(k, at)| k == key && at.elapsed() < std::time::Duration::from_secs(1));
        if merge {
            let out = f(&mut self.project);
            self.dirty = true;
            self.redo.clear();
            self.merge = Some((key, std::time::Instant::now()));
            out
        } else {
            let out = self.edit(f);
            self.merge = Some((key, std::time::Instant::now()));
            out
        }
    }

    /// Start a multi-step edit (a drag). Pair with [`Document::end_gesture`].
    pub fn begin_gesture(&mut self) {
        self.undo.push(self.project.clone());
        self.project.hold_transitions(true);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
    }

    pub fn end_gesture(&mut self) {
        self.project.hold_transitions(false);
        if self.undo.last().is_some_and(|last| *last == self.project) {
            self.undo.pop();
        } else {
            self.redo.clear();
            self.dirty = true;
        }
    }

    pub fn undo(&mut self) -> bool {
        let Some(prev) = self.undo.pop() else {
            return false;
        };
        self.redo.push(std::mem::replace(&mut self.project, prev));
        self.dirty = true;
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(next) = self.redo.pop() else {
            return false;
        };
        self.undo.push(std::mem::replace(&mut self.project, next));
        self.dirty = true;
        true
    }

    /// Write the project to `path` without making it the document's file (autosave).
    pub fn write_copy(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.project).map_err(|e| e.to_string())?;
        // Write beside the target first so a crash never leaves a half-written project.
        let tmp = path.with_extension("kitten.tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save(&mut self, path: &Path) -> Result<(), String> {
        self.write_copy(path)?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn open(path: &Path) -> Result<Document, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let project: Project = serde_json::from_str(&text)
            .map_err(|e| format!("Kein gültiges Happy-Kitten-Projekt: {e}"))?;
        Ok(Document {
            project,
            path: Some(path.to_path_buf()),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MediaKind, MediaSource, Tick};

    #[test]
    fn undo_redo_roundtrip() {
        let mut d = Document::default();
        d.edit(|p| {
            p.add_media(
                "m".into(),
                MediaSource::Sample(0),
                MediaKind::Image,
                Tick::from_secs(5.0),
                (1, 1),
            );
        });
        assert_eq!(d.project.media.len(), 1);
        assert!(d.undo());
        assert!(d.project.media.is_empty());
        assert!(d.redo());
        assert_eq!(d.project.media.len(), 1);
    }

    #[test]
    fn merged_edits_undo_in_one_step() {
        let mut d = Document::default();
        for i in 0..5 {
            d.edit_merge(7, |p| p.name = format!("n{i}"));
        }
        assert_eq!(d.project.name, "n4");
        assert!(d.undo());
        assert_eq!(d.project.name, "Mein Kätzchen-Film");
        assert!(!d.can_undo());
        d.edit_merge(1, |p| p.name = "a".into());
        d.edit_merge(2, |p| p.name = "b".into());
        assert!(d.undo());
        assert_eq!(d.project.name, "a");
    }

    #[test]
    fn noop_edit_leaves_no_history() {
        let mut d = Document::default();
        d.edit(|_| {});
        assert!(!d.can_undo());
        assert!(!d.is_dirty());
    }

    #[test]
    fn save_and_open_roundtrip() {
        let dir = std::env::temp_dir().join(format!("hk-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.kitten");
        let mut d = Document::default();
        d.edit(|p| {
            p.add_media(
                "m".into(),
                MediaSource::Sample(1),
                MediaKind::Image,
                Tick::from_secs(5.0),
                (1, 1),
            );
        });
        d.save(&path).unwrap();
        let loaded = Document::open(&path).unwrap();
        assert_eq!(loaded.project, d.project);
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn corrupt_project_is_an_error_not_a_panic() {
        let dir = std::env::temp_dir().join(format!("hk-test-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.kitten");
        std::fs::write(&path, "{ nope").unwrap();
        assert!(Document::open(&path).is_err());
        std::fs::remove_dir_all(dir).ok();
    }
}
