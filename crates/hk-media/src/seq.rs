//! Sequential hardware decoding through AVAssetReader: far faster than asking
//! for single frames when the playhead just moves forward.

#![allow(deprecated)]

use crate::{MediaError, Rgba};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_av_foundation::{
    AVAssetReader, AVAssetReaderOutput, AVAssetReaderTrackOutput, AVMediaTypeVideo, AVURLAsset,
};
use objc2_core_media::{
    CMSampleBufferGetImageBuffer, CMTime, CMTimeRange, kCMTimePositiveInfinity,
};
use objc2_core_video::{
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
    CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::path::Path;

fn err(msg: impl Into<String>) -> MediaError {
    MediaError::Decode(msg.into())
}

/// Plays one video track forward from a start time.
pub struct SeqReader {
    reader: Retained<AVAssetReader>,
    output: Retained<AVAssetReaderTrackOutput>,
    /// Clockwise quarter turns to apply so the picture is upright.
    turns: u8,
}

impl SeqReader {
    pub fn open_at(path: &Path, start_secs: f64, max_dim: u32) -> Result<SeqReader, MediaError> {
        let path_str = path
            .to_str()
            .ok_or_else(|| err("Pfad ist kein gültiger Text"))?;
        let url = NSURL::fileURLWithPath(&NSString::from_str(path_str));
        // SAFETY: all objects are created and used here; out-pointers are valid; the sample data
        // is read while the buffer is locked and copied before it is released.
        unsafe {
            let asset = AVURLAsset::URLAssetWithURL_options(&url, None);
            let video = AVMediaTypeVideo.ok_or_else(|| err("AVFoundation fehlt"))?;
            let track = asset
                .tracksWithMediaType(video)
                .firstObject()
                .ok_or_else(|| err("Keine Videospur gefunden"))?;
            let natural = track.naturalSize();
            let t = track.preferredTransform();
            let turns = match (
                t.a.round() as i32,
                t.b.round() as i32,
                t.c.round() as i32,
                t.d.round() as i32,
            ) {
                (0, 1, -1, 0) => 1,
                (-1, 0, 0, -1) => 2,
                (0, -1, 1, 0) => 3,
                _ => 0,
            };
            // Let the decoder scale: much less data to move than full 4K frames.
            let longest = natural.width.max(natural.height).max(1.0);
            let k = (max_dim as f64 / longest).min(1.0);
            let even = |v: f64| ((v * k).round() as usize).max(2) & !1;
            let (w, h) = (even(natural.width), even(natural.height));
            let num = |v: usize| -> Retained<AnyObject> {
                Retained::cast_unchecked(NSNumber::numberWithUnsignedInteger(v))
            };
            let keys = [
                NSString::from_str("PixelFormatType"),
                NSString::from_str("Width"),
                NSString::from_str("Height"),
            ];
            let key_refs: Vec<&NSString> = keys.iter().map(|k| &**k).collect();
            let values = vec![num(0x42475241), num(w), num(h)];
            let settings: Retained<NSDictionary<NSString, AnyObject>> =
                NSDictionary::from_retained_objects(&key_refs, &values);
            let output = AVAssetReaderTrackOutput::assetReaderTrackOutputWithTrack_outputSettings(
                &track,
                Some(&settings),
            );
            output.setAlwaysCopiesSampleData(false);
            let reader = AVAssetReader::assetReaderWithAsset_error(&asset)
                .map_err(|e| err(e.localizedDescription().to_string()))?;
            let range = CMTimeRange {
                start: CMTime::with_seconds(start_secs.max(0.0), 600),
                duration: kCMTimePositiveInfinity,
            };
            reader.setTimeRange(range);
            let as_output: &AVAssetReaderOutput = &output;
            if !reader.canAddOutput(as_output) {
                return Err(err("Video-Ausgabe nicht möglich"));
            }
            reader.addOutput(as_output);
            if !reader.startReading() {
                let why = reader
                    .error()
                    .map(|e| e.localizedDescription().to_string())
                    .unwrap_or_default();
                return Err(err(format!("Lesen konnte nicht starten: {why}")));
            }
            Ok(SeqReader {
                reader,
                output,
                turns,
            })
        }
    }

    /// Next frame in presentation order with its time in seconds, or `None` at the end.
    pub fn next_frame(&mut self) -> Result<Option<(f64, Rgba)>, MediaError> {
        // SAFETY: see `open_at`; the pixel buffer is locked while we read it.
        unsafe {
            let Some(sample) = self.output.copyNextSampleBuffer() else {
                return Ok(None);
            };
            let pts = objc2_core_media::CMSampleBufferGetPresentationTimeStamp(&sample).seconds();
            let Some(raw) = CMSampleBufferGetImageBuffer(&sample) else {
                return Ok(None);
            };
            let buffer: &objc2_core_video::CVPixelBuffer = &raw;
            CVPixelBufferLockBaseAddress(buffer, CVPixelBufferLockFlags::ReadOnly);
            let (w, h) = (
                CVPixelBufferGetWidth(buffer),
                CVPixelBufferGetHeight(buffer),
            );
            let stride = CVPixelBufferGetBytesPerRow(buffer);
            let base = CVPixelBufferGetBaseAddress(buffer) as *const u8;
            let result = if base.is_null() || w == 0 || h == 0 || stride < w * 4 {
                Err(err("Videobild nicht lesbar"))
            } else {
                let src = std::slice::from_raw_parts(base, stride * h);
                Ok(convert(src, w, h, stride, self.turns))
            };
            CVPixelBufferUnlockBaseAddress(buffer, CVPixelBufferLockFlags::ReadOnly);
            result.map(|img| Some((if pts.is_finite() { pts } else { 0.0 }, img)))
        }
    }
}

impl Drop for SeqReader {
    fn drop(&mut self) {
        // SAFETY: valid reader; cancelling stops the background read-ahead.
        unsafe { self.reader.cancelReading() };
    }
}

/// BGRA (with row padding) to upright RGBA.
fn convert(src: &[u8], w: usize, h: usize, stride: usize, turns: u8) -> Rgba {
    let (ow, oh) = if turns % 2 == 1 { (h, w) } else { (w, h) };
    let mut pixels = vec![255u8; ow * oh * 4];
    for y in 0..h {
        let row = &src[y * stride..y * stride + w * 4];
        for (x, px) in row.as_chunks::<4>().0.iter().enumerate() {
            let (dx, dy) = match turns % 4 {
                0 => (x, y),
                1 => (h - 1 - y, x),
                2 => (w - 1 - x, h - 1 - y),
                _ => (y, w - 1 - x),
            };
            let o = (dy * ow + dx) * 4;
            pixels[o] = px[2];
            pixels[o + 1] = px[1];
            pixels[o + 2] = px[0];
        }
    }
    Rgba {
        width: ow as u32,
        height: oh as u32,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::convert;

    #[test]
    fn convert_swaps_channels_and_rotates() {
        // 2x1 image: pixel0 = blue, pixel1 = red (BGRA order), stride padded to 12
        let src = [255, 0, 0, 255, 0, 0, 255, 255, 9, 9, 9, 9];
        let a = convert(&src, 2, 1, 12, 0);
        assert_eq!((a.width, a.height), (2, 1));
        assert_eq!(&a.pixels[..4], &[0, 0, 255, 255]);
        assert_eq!(&a.pixels[4..8], &[255, 0, 0, 255]);
        let b = convert(&src, 2, 1, 12, 1);
        assert_eq!((b.width, b.height), (1, 2));
        // clockwise: the left pixel ends up on top
        assert_eq!(&b.pixels[..4], &[0, 0, 255, 255]);
        let c = convert(&src, 2, 1, 12, 2);
        assert_eq!(&c.pixels[..4], &[255, 0, 0, 255]);
    }
}
