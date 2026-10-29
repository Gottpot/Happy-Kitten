//! Android glue: where the app may write, the system language and the screen class.
//! Everything that the Mac version asks the operating system lives here.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
static LANGUAGE: OnceLock<String> = OnceLock::new();

pub fn init(app: &android_activity::AndroidApp) {
    if let Some(dir) = app.internal_data_path() {
        let _ = DATA_DIR.set(dir);
    }
    if let Some(lang) = app.config().language() {
        let _ = LANGUAGE.set(lang);
    }
}

/// Private app folder (`/data/data/<paket>/files`): projects, autosave, settings, copied media.
pub fn data_dir() -> Option<PathBuf> {
    DATA_DIR.get().cloned()
}

/// Where projects are kept (`.kitten` files).
pub fn projects_dir() -> Option<PathBuf> {
    let dir = data_dir()?.join("projekte");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Where exports and stills are rendered before they are copied into the gallery.
pub fn export_dir() -> Option<PathBuf> {
    let dir = data_dir()?.join("export");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

/// Two-letter language of the device ("de", "ja", ...), empty when unknown.
pub fn system_language() -> String {
    LANGUAGE.get().cloned().unwrap_or_default().to_ascii_lowercase()
}

/// Phone or tablet layout. Below this width (in points) the phone layout is used.
pub const TABLET_MIN_WIDTH: f32 = 720.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    Phone,
    Tablet,
}

impl Form {
    /// Decided from the shorter screen side, so a tablet turned upright stays a tablet and a
    /// phone turned sideways stays a phone.
    pub fn of(screen: egui::Vec2) -> Form {
        if screen.x.min(screen.y) >= TABLET_MIN_WIDTH {
            Form::Tablet
        } else {
            Form::Phone
        }
    }
}

/// Space taken by the status bar, camera cut-out and navigation bar, in points
/// (`top`, `bottom`, `left`, `right`). Asked from the window about twice a second.
pub fn safe_insets(pixels_per_point: f32) -> [f32; 4] {
    static CACHE: Mutex<Option<(Instant, [i32; 4])>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(PoisonError::into_inner);
    let stale = cache.is_none_or(|(at, _)| at.elapsed() > Duration::from_millis(500));
    if stale {
        let px = crate::files::insets_px().unwrap_or([0; 4]);
        *cache = Some((Instant::now(), px));
    }
    let px = cache.map_or([0; 4], |(_, px)| px);
    let ppp = pixels_per_point.max(0.5);
    px.map(|v| v.max(0) as f32 / ppp)
}
