//! Media access for Happy-Kitten on Android: built-in demo scenes, still images and
//! video through the NDK's MediaCodec/MediaExtractor/MediaMuxer. All of it works offline.

mod overlay;
mod samples;
mod stills;

mod decode;
mod encode;
mod ndk;

pub use overlay::{compose_overlay, image_has_alpha};
pub use samples::{SAMPLE_COUNT, sample_frame, sample_name};
pub use stills::load_image;

use hk_core::{MediaKind, Tick};
use std::path::Path;

/// Tightly packed RGBA8 picture.
#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Dateityp wird nicht unterstützt: {0}")]
    Unsupported(String),
    #[error("{0}")]
    Decode(String),
}

#[derive(Clone, Debug)]
pub struct Probe {
    pub kind: MediaKind,
    pub duration: Tick,
    pub size: (u32, u32),
    /// The picture has see-through parts (PNG logos, buttons).
    pub alpha: bool,
}

const IMAGE_EXT: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff"];
const VIDEO_EXT: &[&str] = &["mp4", "mov", "m4v", "avi", "mkv", "webm"];
const AUDIO_EXT: &[&str] = &["mp3", "wav", "m4a", "aac", "aif", "aiff", "flac"];

pub fn kind_of(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if IMAGE_EXT.contains(&ext.as_str()) {
        Some(MediaKind::Image)
    } else if VIDEO_EXT.contains(&ext.as_str()) {
        Some(MediaKind::Video)
    } else if AUDIO_EXT.contains(&ext.as_str()) {
        Some(MediaKind::Audio)
    } else {
        None
    }
}

pub fn probe(path: &Path) -> Result<Probe, MediaError> {
    let kind = kind_of(path).ok_or_else(|| MediaError::Unsupported(path.display().to_string()))?;
    match kind {
        MediaKind::Image => {
            let size =
                image::image_dimensions(path).map_err(|e| MediaError::Decode(e.to_string()))?;
            Ok(Probe {
                kind,
                duration: Tick::from_secs(5.0),
                size,
                alpha: image_has_alpha(path),
            })
        }
        MediaKind::Video => probe_av(path, kind),
        MediaKind::Audio => Err(MediaError::Unsupported(
            "Audio wird von hk-audio gelesen".into(),
        )),
    }
}

fn probe_av(path: &Path, kind: MediaKind) -> Result<Probe, MediaError> {
    decode::probe(path, kind)
}

pub use decode::{VideoReader, VideoSource};
pub use encode::{Codec, VideoWriter, WriterConfig};

/// One video frame at `secs` into the file, scaled to fit `max_dim`.
pub fn video_frame(path: &Path, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
    decode::frame_at(path, secs, max_dim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_by_extension() {
        assert_eq!(kind_of(Path::new("a.PNG")), Some(MediaKind::Image));
        assert_eq!(kind_of(Path::new("a.mov")), Some(MediaKind::Video));
        assert_eq!(kind_of(Path::new("a.wav")), Some(MediaKind::Audio));
        assert_eq!(kind_of(Path::new("a.txt")), None);
        assert_eq!(kind_of(Path::new("noext")), None);
    }

    #[test]
    fn samples_have_right_size_and_are_opaque() {
        for i in 0..SAMPLE_COUNT {
            let f = sample_frame(i, 64, 36);
            assert_eq!(f.pixels.len(), 64 * 36 * 4);
            assert!(f.pixels.chunks(4).all(|p| p[3] == 255));
        }
    }
}
