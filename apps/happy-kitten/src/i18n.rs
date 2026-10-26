//! Languages of the user interface.
//!
//! The German text in the source is the key: `tr("Datei")` returns the text in the chosen
//! language, or the German original when there is none. Dynamic parts use placeholders:
//! `trf!("{n} Clips ausgewählt", n = 3)`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    De,
    En,
    Ja,
    Ko,
    Es,
}

impl Lang {
    pub const ALL: [Lang; 5] = [Lang::De, Lang::En, Lang::Ja, Lang::Ko, Lang::Es];

    /// Short code shown on the language button (country style: JP, KR).
    pub fn code(self) -> &'static str {
        match self {
            Lang::De => "DE",
            Lang::En => "EN",
            Lang::Ja => "JP",
            Lang::Ko => "KR",
            Lang::Es => "ES",
        }
    }

    /// Name of the language in that language.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::De => "Deutsch",
            Lang::En => "English",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
            Lang::Es => "Español",
        }
    }

    /// Identifier saved in the settings file.
    pub fn id(self) -> &'static str {
        match self {
            Lang::De => "de",
            Lang::En => "en",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::Es => "es",
        }
    }

    pub fn from_id(id: &str) -> Option<Lang> {
        Lang::ALL
            .into_iter()
            .find(|l| l.id() == id.trim().to_ascii_lowercase())
    }

    /// Index into the translation entries (German is the key itself).
    fn column(self) -> Option<usize> {
        match self {
            Lang::De => None,
            Lang::En => Some(0),
            Lang::Ja => Some(1),
            Lang::Ko => Some(2),
            Lang::Es => Some(3),
        }
    }

    fn to_u8(self) -> u8 {
        Lang::ALL.iter().position(|l| *l == self).unwrap_or(0) as u8
    }

    fn from_u8(v: u8) -> Lang {
        Lang::ALL.get(v as usize).copied().unwrap_or(Lang::De)
    }
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    CURRENT.store(l.to_u8(), Ordering::Relaxed);
}

pub fn lang() -> Lang {
    Lang::from_u8(CURRENT.load(Ordering::Relaxed))
}

type Entry = &'static [&'static str; 4];

fn table() -> &'static HashMap<&'static str, Entry> {
    static MAP: OnceLock<HashMap<&'static str, Entry>> = OnceLock::new();
    MAP.get_or_init(|| {
        crate::i18n_data::ENTRIES
            .iter()
            .map(|(k, v)| (*k, v))
            .collect()
    })
}

/// `key` in the current language; unknown keys come back unchanged.
pub fn tr(key: &str) -> &str {
    let Some(col) = lang().column() else {
        return key;
    };
    match table().get(key).and_then(|e| e.get(col)) {
        Some(t) if !t.is_empty() => t,
        _ => key,
    }
}

/// Like [`tr`] for texts that are built at run time (error messages). A table key that ends with
/// `:` also matches messages that start with it and carry a detail after it.
pub fn tr_dyn(text: &str) -> String {
    if lang() == Lang::De {
        return text.to_string();
    }
    let direct = tr(text);
    if direct != text {
        return direct.to_string();
    }
    if let Some((head, rest)) = text.split_once(':') {
        let key = format!("{head}:");
        let t = tr(&key);
        if t != key {
            let detail = rest.trim_start();
            let detail = if detail.contains(':') {
                detail.to_string()
            } else {
                tr(detail).to_string()
            };
            return format!("{t} {detail}");
        }
    }
    text.to_string()
}

/// Fill `{name}` placeholders of the translated `key`.
pub fn tr_fmt(key: &str, args: &[(&str, String)]) -> String {
    let mut out: Cow<str> = Cow::Borrowed(tr(key));
    for (name, value) in args {
        out = Cow::Owned(out.replace(&format!("{{{name}}}"), value));
    }
    out.into_owned()
}

/// `trf!("{n} Clips ausgewählt", n = count)`
#[macro_export]
macro_rules! trf {
    ($key:literal $(, $name:ident = $val:expr)* $(,)?) => {
        $crate::i18n::tr_fmt($key, &[$((stringify!($name), $val.to_string())),*])
    };
}

/// The language of macOS if we have it, English otherwise.
pub fn detect_system() -> Lang {
    let out = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLanguages"])
        .output()
        .ok();
    let text = out
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let first = text.split('"').nth(1).unwrap_or("").to_ascii_lowercase();
    Lang::ALL
        .into_iter()
        .find(|l| first.starts_with(l.id()))
        .unwrap_or(Lang::En)
}

fn settings_path() -> Option<std::path::PathBuf> {
    crate::autosave::dir().map(|d| d.join("settings.json"))
}

/// The language to start with: `HK_LANG` (for screenshots), else the saved one, else the system's.
pub fn initial() -> Lang {
    std::env::var("HK_LANG")
        .ok()
        .and_then(|v| Lang::from_id(&v))
        .unwrap_or_else(load_saved_or_system)
}

/// The saved language, or the system language on the first start.
pub fn load_saved_or_system() -> Lang {
    let saved = settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| {
            v.get("lang")
                .and_then(|l| l.as_str())
                .and_then(Lang::from_id)
        });
    saved.unwrap_or_else(detect_system)
}

pub fn save(l: Lang) {
    if let Some(p) = settings_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, format!("{{\"lang\":\"{}\"}}\n", l.id()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n_data::ENTRIES;

    fn placeholders(s: &str) -> Vec<&str> {
        let mut v: Vec<&str> = s
            .match_indices('{')
            .filter_map(|(i, _)| s[i..].find('}').map(|e| &s[i..=i + e]))
            .collect();
        v.sort_unstable();
        v
    }

    #[test]
    fn every_entry_is_filled_and_keeps_its_placeholders() {
        for (de, cols) in ENTRIES {
            for (i, t) in cols.iter().enumerate() {
                assert!(!t.trim().is_empty(), "empty translation {i} for {de:?}");
                assert_eq!(
                    placeholders(t),
                    placeholders(de),
                    "placeholders differ in {t:?} (key {de:?})"
                );
            }
        }
    }

    #[test]
    fn every_transition_and_effect_text_is_translated() {
        let known: std::collections::HashSet<&str> = ENTRIES.iter().map(|(k, _)| *k).collect();
        let mut keys: Vec<&str> = Vec::new();
        for k in hk_core::TransitionKind::ALL {
            keys.push(k.name());
            keys.push(k.blurb());
        }
        for fx in hk_core::AudioFx::ALL {
            keys.push(fx.name());
            keys.push(fx.blurb());
        }
        for k in hk_core::FxKind::ALL {
            keys.push(k.name());
            keys.push(k.blurb());
        }
        for g in hk_core::FxGroup::ALL {
            keys.push(g.name());
        }
        for key in keys {
            assert!(known.contains(key), "no translation for {key:?}");
        }
    }

    #[test]
    fn keys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for (de, _) in ENTRIES {
            assert!(seen.insert(*de), "duplicate key {de:?}");
        }
    }

    #[test]
    fn language_codes_and_ids_round_trip() {
        for l in Lang::ALL {
            assert_eq!(Lang::from_id(l.id()), Some(l));
        }
        assert_eq!(Lang::Ja.code(), "JP");
        assert_eq!(Lang::Ko.code(), "KR");
        assert_eq!(Lang::from_id(" EN "), Some(Lang::En));
        assert_eq!(Lang::from_id("fr"), None);
    }

    #[test]
    fn lookup_falls_back_to_german() {
        set_lang(Lang::Es);
        assert_eq!(tr("Speichern"), "Guardar");
        assert_eq!(tr("gibt es nicht"), "gibt es nicht");
        assert_eq!(
            tr_fmt("{n} Clips ausgewählt", &[("n", "3".to_string())]),
            "3 clips seleccionados"
        );
        assert_eq!(tr_dyn("Gespeichert ✓"), "Guardado ✓");
        assert_eq!(
            tr_dyn("Speichern fehlgeschlagen: Die Datei enthält keinen hörbaren Ton"),
            "No se pudo guardar: El archivo no contiene sonido audible"
        );
        set_lang(Lang::De);
        assert_eq!(tr("Speichern"), "Speichern");
    }
}
