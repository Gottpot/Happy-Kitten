//! Video probing and frame extraction through AVFoundation.
//! This is the only module in the workspace that needs `unsafe`: it talks to
//! Apple frameworks through the `objc2` bindings.

use crate::{MediaError, Probe, Rgba};
use hk_core::{MediaKind, Tick};
use objc2::rc::Retained;
use objc2_av_foundation::{AVAssetImageGenerator, AVMediaTypeVideo, AVURLAsset};
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo,
};
use objc2_core_media::{CMTime, kCMTimeZero};
use objc2_foundation::{NSString, NSURL};
use std::ffi::c_void;
use std::path::Path;
use std::ptr::NonNull;

unsafe extern "C-unwind" {
    fn CGBitmapContextCreate(
        data: *mut c_void,
        width: usize,
        height: usize,
        bits_per_component: usize,
        bytes_per_row: usize,
        space: Option<&CGColorSpace>,
        bitmap_info: u32,
    ) -> Option<NonNull<CGContext>>;
}

fn decode_err(msg: impl Into<String>) -> MediaError {
    MediaError::Decode(msg.into())
}

fn open_asset(path: &Path) -> Result<Retained<AVURLAsset>, MediaError> {
    let path_str = path
        .to_str()
        .ok_or_else(|| decode_err("Pfad ist kein gültiger Text"))?;
    let ns_path = NSString::from_str(path_str);
    let url = NSURL::fileURLWithPath(&ns_path);
    // SAFETY: `url` is a valid file URL and `None` options are allowed.
    Ok(unsafe { AVURLAsset::URLAssetWithURL_options(&url, None) })
}

/// A video file kept open so scrubbing does not reopen it for every frame.
pub struct Reader {
    generator: Retained<AVAssetImageGenerator>,
    duration: Tick,
}

impl Reader {
    pub fn open(path: &Path) -> Result<Reader, MediaError> {
        if !path.exists() {
            return Err(decode_err(format!(
                "Datei nicht gefunden: {}",
                path.display()
            )));
        }
        let asset = open_asset(path)?;
        // SAFETY: plain property reads / configuration on live AVFoundation objects.
        unsafe {
            let secs = asset.duration().seconds();
            let video = AVMediaTypeVideo.ok_or_else(|| decode_err("AVFoundation fehlt"))?;
            #[allow(deprecated)]
            let has_video = asset.tracksWithMediaType(video).count() > 0;
            if !has_video {
                return Err(decode_err("Keine Videospur gefunden"));
            }
            let generator = AVAssetImageGenerator::assetImageGeneratorWithAsset(&asset);
            generator.setAppliesPreferredTrackTransform(true);
            generator.setRequestedTimeToleranceBefore(kCMTimeZero);
            generator.setRequestedTimeToleranceAfter(kCMTimeZero);
            Ok(Reader {
                generator,
                duration: Tick::from_secs(if secs.is_finite() { secs.max(0.0) } else { 0.0 }),
            })
        }
    }

    pub fn duration(&self) -> Tick {
        self.duration
    }

    /// Frame at `secs`, scaled so neither side exceeds `max_dim`.
    pub fn frame_at(&self, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
        let secs = if secs.is_finite() { secs.max(0.0) } else { 0.0 };
        // Asking for exactly the end fails on some files; stay one tick inside.
        let secs = if self.duration.secs() > 0.0 {
            secs.min((self.duration.secs() - 0.001).max(0.0))
        } else {
            secs
        };
        // SAFETY: the generator is alive; `actual_time` may be null.
        let image = unsafe {
            let m = max_dim.max(16) as f64;
            self.generator.setMaximumSize(CGSize::new(m, m));
            let t: CMTime = CMTime::with_seconds(secs, 600);
            #[allow(deprecated)]
            self.generator
                .copyCGImageAtTime_actualTime_error(t, std::ptr::null_mut())
                .map_err(|e| decode_err(e.localizedDescription().to_string()))?
        };
        image_to_rgba(&image)
    }
}

fn image_to_rgba(image: &CGImage) -> Result<Rgba, MediaError> {
    let width = CGImage::width(Some(image));
    let height = CGImage::height(Some(image));
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(decode_err("Ungültige Bildgröße"));
    }
    let mut pixels = vec![0u8; width * height * 4];
    let space = CGColorSpace::new_device_rgb().ok_or_else(|| decode_err("Kein Farbraum"))?;
    let info = CGImageAlphaInfo::NoneSkipLast.0 | CGImageByteOrderInfo::Order32Big.0;
    // SAFETY: `pixels` outlives the context (dropped before this function returns)
    // and is exactly `height * width * 4` bytes, matching bytes_per_row below.
    let ctx = unsafe {
        let raw = CGBitmapContextCreate(
            pixels.as_mut_ptr().cast(),
            width,
            height,
            8,
            width * 4,
            Some(&space),
            info,
        )
        .ok_or_else(|| decode_err("Bitmap-Kontext fehlgeschlagen"))?;
        CFRetained::from_raw(raw)
    };
    CGContext::draw_image(
        Some(&ctx),
        CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(width as f64, height as f64),
        ),
        Some(image),
    );
    drop(ctx);
    for px in pixels.as_chunks_mut::<4>().0 {
        px[3] = 255;
    }
    Ok(Rgba {
        width: width as u32,
        height: height as u32,
        pixels,
    })
}

pub fn probe(path: &Path, kind: MediaKind) -> Result<Probe, MediaError> {
    match kind {
        MediaKind::Video => {
            let reader = Reader::open(path)?;
            let first = reader.frame_at(0.0, 4096)?;
            Ok(Probe {
                kind,
                duration: reader.duration(),
                size: (first.width, first.height),
                alpha: false,
            })
        }
        _ => Err(decode_err("Kein Video")),
    }
}

pub fn frame_at(path: &Path, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
    Reader::open(path)?.frame_at(secs, max_dim)
}
