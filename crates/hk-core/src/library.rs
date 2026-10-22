//! The global library: media and text styles that every project can use.
//! Media are referenced by path (never copied), so moving a file shows up as missing.

use crate::project::{Id, MediaKind};
use crate::text::TextSpec;
use crate::time::Tick;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Library ids live above this bit so they never collide with project ids.
pub const LIB_ID_BIT: Id = 1 << 62;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LibItem {
    pub id: Id,
    pub name: String,
    pub path: PathBuf,
    pub kind: MediaKind,
    pub duration: Tick,
    pub size: (u32, u32),
    #[serde(default)]
    pub alpha: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextPreset {
    pub name: String,
    pub spec: TextSpec,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GlobalLibrary {
    pub items: Vec<LibItem>,
    pub presets: Vec<TextPreset>,
    next: u64,
}

impl GlobalLibrary {
    /// Missing or unreadable files give an empty library; the app must always start.
    pub fn load(path: &Path) -> GlobalLibrary {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, path).map_err(|e| e.to_string())
    }

    /// Add a file once; adding the same path again returns the existing entry.
    pub fn add_item(
        &mut self,
        name: String,
        path: PathBuf,
        kind: MediaKind,
        duration: Tick,
        size: (u32, u32),
    ) -> Id {
        if let Some(existing) = self.items.iter().find(|i| i.path == path) {
            return existing.id;
        }
        self.next += 1;
        let id = LIB_ID_BIT | self.next;
        self.items.push(LibItem {
            id,
            name,
            path,
            kind,
            duration,
            size,
            alpha: false,
        });
        id
    }

    pub fn set_alpha(&mut self, id: Id, alpha: bool) {
        if let Some(i) = self.items.iter_mut().find(|i| i.id == id) {
            i.alpha = alpha;
        }
    }

    pub fn item(&self, id: Id) -> Option<&LibItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn remove_item(&mut self, id: Id) {
        self.items.retain(|i| i.id != id);
    }

    pub fn add_preset(&mut self, name: String, spec: TextSpec) {
        self.presets.push(TextPreset { name, spec });
    }

    pub fn remove_preset(&mut self, index: usize) {
        if index < self.presets.len() {
            self.presets.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_is_one_entry_and_ids_do_not_clash_with_projects() {
        let mut lib = GlobalLibrary::default();
        let a = lib.add_item(
            "a".into(),
            "/x/a.mp4".into(),
            MediaKind::Video,
            Tick::ZERO,
            (1, 1),
        );
        let b = lib.add_item(
            "again".into(),
            "/x/a.mp4".into(),
            MediaKind::Video,
            Tick::ZERO,
            (1, 1),
        );
        assert_eq!(a, b);
        assert!(a & LIB_ID_BIT != 0);
        assert_eq!(lib.items.len(), 1);
    }

    #[test]
    fn roundtrip_and_bad_file() {
        let dir = std::env::temp_dir().join(format!("hk-lib-{}", std::process::id()));
        let path = dir.join("library.json");
        let mut lib = GlobalLibrary::default();
        lib.add_preset("Titel".into(), TextSpec::default());
        lib.add_item(
            "song".into(),
            "/x/s.mp3".into(),
            MediaKind::Audio,
            Tick::from_secs(3.0),
            (0, 0),
        );
        lib.save(&path).unwrap();
        assert_eq!(GlobalLibrary::load(&path), lib);
        std::fs::write(&path, "{kaputt").unwrap();
        assert_eq!(GlobalLibrary::load(&path), GlobalLibrary::default());
        std::fs::remove_dir_all(dir).ok();
    }
}
