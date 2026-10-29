//! Video decoding through MediaExtractor + MediaCodec (ByteBuffer output, converted to RGBA on
//! the CPU). `VideoSource` is fast when time mostly moves forward (playback, export) and still
//! exact when it jumps.

use crate::ndk::{Codec, Extractor, FLAG_CONFIG, FLAG_EOS, Format, Output};
use crate::{MediaError, Probe, Rgba};
use hk_core::{MediaKind, Tick};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Within this distance ahead of the last frame we keep decoding sequentially.
const FORWARD_WINDOW_US: i64 = 1_500_000;
const EPS_US: i64 = 1_000;
/// A codec that produces nothing for this long is considered stuck.
const STALL: Duration = Duration::from_secs(8);

const COLOR_I420: i32 = 19;
const COLOR_NV12: i32 = 21;

fn err(msg: impl Into<String>) -> MediaError {
    MediaError::Decode(msg.into())
}

struct TrackInfo {
    index: usize,
    format: Format,
    mime: String,
    width: u32,
    height: u32,
    rotation: i32,
    duration_us: i64,
}

fn find_video(ex: &Extractor) -> Result<TrackInfo, MediaError> {
    for index in 0..ex.track_count() {
        let Some(format) = ex.track_format(index) else {
            continue;
        };
        let Some(mime) = format.string("mime").filter(|m| m.starts_with("video/")) else {
            continue;
        };
        let width = format.i32("width").unwrap_or(0).max(0) as u32;
        let height = format.i32("height").unwrap_or(0).max(0) as u32;
        if width == 0 || height == 0 {
            continue;
        }
        let rotation = format.i32("rotation-degrees").unwrap_or(0).rem_euclid(360);
        let duration_us = format.i64("durationUs").unwrap_or(0).max(0);
        return Ok(TrackInfo {
            index,
            format,
            mime,
            width,
            height,
            rotation,
            duration_us,
        });
    }
    Err(err("Die Datei enthält kein Video"))
}

pub fn probe(path: &Path, kind: MediaKind) -> Result<Probe, MediaError> {
    let ex = Extractor::open(path).map_err(err)?;
    let t = find_video(&ex)?;
    let (w, h) = if t.rotation % 180 == 90 {
        (t.height, t.width)
    } else {
        (t.width, t.height)
    };
    Ok(Probe {
        kind,
        duration: Tick::from_secs(t.duration_us as f64 / 1e6),
        size: (w, h),
        alpha: false,
    })
}

/// Where the planes of a decoded frame are inside the codec's output buffer.
#[derive(Clone, Copy, Debug)]
struct Layout {
    format: i32,
    width: usize,
    height: usize,
    stride: usize,
    slice_height: usize,
    left: usize,
    top: usize,
}

impl Layout {
    fn from_format(f: &Format, fallback: (u32, u32)) -> Layout {
        let get = |k: &str| f.i32(k).filter(|v| *v >= 0).map(|v| v as usize);
        let full_w = get("width").unwrap_or(fallback.0 as usize);
        let full_h = get("height").unwrap_or(fallback.1 as usize);
        let (left, top) = (get("crop-left").unwrap_or(0), get("crop-top").unwrap_or(0));
        let width = get("crop-right")
            .map(|r| (r + 1).saturating_sub(left))
            .filter(|w| *w > 0)
            .unwrap_or(full_w);
        let height = get("crop-bottom")
            .map(|b| (b + 1).saturating_sub(top))
            .filter(|h| *h > 0)
            .unwrap_or(full_h);
        let stride = get("stride").filter(|s| *s >= full_w).unwrap_or(full_w);
        let slice_height = get("slice-height")
            .filter(|s| *s >= full_h)
            .unwrap_or(full_h);
        Layout {
            // Anything but planar is treated as semi-planar NV12 (the flexible format of
            // current decoders is laid out that way).
            format: f.i32("color-format").unwrap_or(COLOR_NV12),
            width,
            height,
            stride,
            slice_height,
            left,
            top,
        }
    }
}

pub struct Frame {
    pts_us: i64,
    layout: Layout,
    data: Vec<u8>,
}

struct Decoder {
    ex: Extractor,
    codec: Codec,
    info_size: (u32, u32),
    rotation: i32,
    duration_us: i64,
    layout: Layout,
    input_done: bool,
    output_done: bool,
    queued: u32,
    produced: u32,
    scratch: Vec<u8>,
}

impl Decoder {
    fn open(path: &Path) -> Result<Decoder, MediaError> {
        let mut ex = Extractor::open(path).map_err(err)?;
        let t = find_video(&ex)?;
        ex.select(t.index).map_err(err)?;
        let mut codec = Codec::decoder(&t.mime).map_err(err)?;
        // The decoder picks its own output layout; `Layout::from_format` reads it back.
        codec.configure(&t.format, false).map_err(err)?;
        codec.start().map_err(err)?;
        let size = (t.width, t.height);
        Ok(Decoder {
            ex,
            codec,
            info_size: size,
            rotation: t.rotation,
            duration_us: t.duration_us,
            layout: Layout {
                format: COLOR_NV12,
                width: size.0 as usize,
                height: size.1 as usize,
                stride: size.0 as usize,
                slice_height: size.1 as usize,
                left: 0,
                top: 0,
            },
            input_done: false,
            output_done: false,
            queued: 0,
            produced: 0,
            scratch: vec![0; 1 << 20],
        })
    }

    fn seek(&mut self, us: i64) -> Result<(), MediaError> {
        self.ex.seek(us).map_err(err)?;
        // A codec that has not been fed yet has nothing to flush (and dislikes it).
        if self.queued > 0 {
            self.codec.flush().map_err(err)?;
        }
        self.input_done = false;
        self.output_done = false;
        self.queued = 0;
        Ok(())
    }

    /// The next frame in presentation order, `None` at the end of the file.
    fn next_frame(&mut self) -> Result<Option<Frame>, MediaError> {
        let mut last_progress = Instant::now();
        loop {
            if self.output_done {
                return Ok(None);
            }
            let mut progressed = false;
            match self.codec.dequeue_output(0).map_err(err)? {
                Output::Buffer(out) => {
                    progressed = true;
                    let data = if out.size > 0 && out.flags & FLAG_CONFIG == 0 {
                        self.codec.output_bytes(&out)
                    } else {
                        Vec::new()
                    };
                    self.codec.release_output(out.index);
                    if out.flags & FLAG_EOS != 0 {
                        self.output_done = true;
                    }
                    if !data.is_empty() {
                        self.produced += 1;
                        return Ok(Some(Frame {
                            pts_us: out.pts,
                            layout: self.layout,
                            data,
                        }));
                    }
                }
                Output::FormatChanged => {
                    progressed = true;
                    if let Some(f) = self.codec.output_format() {
                        self.layout = Layout::from_format(&f, self.info_size);
                    }
                }
                Output::TryAgain => {}
            }
            if !self.input_done {
                if let Some(i) = self.codec.dequeue_input(0) {
                    progressed = true;
                    let cap = self.codec.input_buffer(i).len();
                    if self.scratch.len() < cap {
                        self.scratch.resize(cap, 0);
                    }
                    let read = self.ex.read(&mut self.scratch[..cap]);
                    match read {
                        Some((n, pts)) => {
                            self.codec.input_buffer(i)[..n].copy_from_slice(&self.scratch[..n]);
                            self.codec.queue_input(i, n, pts, 0).map_err(err)?;
                            self.queued += 1;
                            self.ex.advance();
                        }
                        None => {
                            self.codec.queue_input(i, 0, 0, FLAG_EOS).map_err(err)?;
                            self.input_done = true;
                        }
                    }
                }
            }
            if progressed {
                last_progress = Instant::now();
            } else {
                if last_progress.elapsed() > STALL {
                    return Err(err(format!(
                        "Der Decoder antwortet nicht mehr ({} Pakete hinein, {} Bilder heraus, Format {})",
                        self.queued, self.produced, self.layout.format
                    )));
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
}

/// One video file as a random-access frame source.
pub struct VideoSource {
    dec: Decoder,
    cur: Option<Frame>,
    peek: Option<Frame>,
    cur_dim: u32,
}

impl VideoSource {
    pub fn open(path: &Path) -> Result<VideoSource, MediaError> {
        Ok(VideoSource {
            dec: Decoder::open(path)?,
            cur: None,
            peek: None,
            cur_dim: 0,
        })
    }

    pub fn duration(&self) -> Tick {
        Tick::from_secs(self.dec.duration_us as f64 / 1e6)
    }

    /// The frame shown at `secs`, scaled to fit `max_dim`.
    pub fn frame_at(&mut self, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
        let end = self.dec.duration_us;
        let mut target = if secs.is_finite() {
            (secs.max(0.0) * 1e6) as i64
        } else {
            0
        };
        if end > 0 {
            target = target.min((end - 1_000).max(0));
        }
        let sequential = self
            .cur
            .as_ref()
            .is_some_and(|c| target >= c.pts_us - EPS_US && target - c.pts_us < FORWARD_WINDOW_US);
        if !sequential {
            self.cur = None;
            self.peek = None;
            self.dec.seek(target)?;
        }
        loop {
            let next = match self.peek.take() {
                Some(p) => Some(p),
                None => self.dec.next_frame()?,
            };
            match next {
                Some(p) if p.pts_us <= target + EPS_US || self.cur.is_none() => self.cur = Some(p),
                Some(p) => {
                    self.peek = Some(p);
                    break;
                }
                None => break,
            }
        }
        self.cur_dim = max_dim;
        let frame = self
            .cur
            .as_ref()
            .ok_or_else(|| err("Kein Bild an dieser Stelle"))?;
        Ok(to_rgba(frame, self.dec.rotation, max_dim))
    }
}

/// Single frames of a file (thumbnails, stills): opens a decoder per call.
pub struct VideoReader {
    path: PathBuf,
    duration: Tick,
}

impl VideoReader {
    pub fn open(path: &Path) -> Result<VideoReader, MediaError> {
        let p = probe(path, MediaKind::Video)?;
        Ok(VideoReader {
            path: path.to_path_buf(),
            duration: p.duration,
        })
    }

    pub fn duration(&self) -> Tick {
        self.duration
    }

    pub fn frame_at(&self, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
        VideoSource::open(&self.path)?.frame_at(secs, max_dim)
    }
}

pub fn frame_at(path: &Path, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
    VideoSource::open(path)?.frame_at(secs, max_dim)
}

// ---------------------------------------------------------------- YUV -> RGBA

fn clamp8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

/// Converts, rotates (clockwise by `rotation`) and scales a frame to fit `max_dim`.
fn to_rgba(f: &Frame, rotation: i32, max_dim: u32) -> Rgba {
    let l = &f.layout;
    let (w, h) = (l.width.max(1), l.height.max(1));
    let (rw, rh) = if rotation % 180 == 90 { (h, w) } else { (w, h) };
    let scale = (max_dim.max(2) as f32 / rw.max(rh) as f32).min(1.0);
    let ow = ((rw as f32 * scale).round() as usize).max(2) & !1;
    let oh = ((rh as f32 * scale).round() as usize).max(2) & !1;
    let hd = l.width >= 1280 || l.height >= 720;
    // BT.709 for HD, BT.601 below (limited range both)
    let (kr, kgu, kgv, kb) = if hd { (459, 55, 136, 541) } else { (409, 100, 208, 516) };
    let planar = l.format == COLOR_I420;
    let y_size = l.stride * l.slice_height;
    let chroma_stride = if planar { l.stride / 2 } else { l.stride };
    let v_plane = y_size + chroma_stride * (l.slice_height / 2);
    let at = |i: usize| f.data.get(i).copied().unwrap_or(128) as i32;

    let mut pixels = vec![255u8; ow * oh * 4];
    for oy in 0..oh {
        let ry = oy * rh / oh;
        for ox in 0..ow {
            let rx = ox * rw / ow;
            let (sx, sy) = match rotation {
                90 => (ry, h - 1 - rx.min(h - 1)),
                180 => (w - 1 - rx.min(w - 1), h - 1 - ry.min(h - 1)),
                270 => (w - 1 - ry.min(w - 1), rx),
                _ => (rx, ry),
            };
            let (sx, sy) = (sx.min(w - 1) + l.left, sy.min(h - 1) + l.top);
            let y = at(sy * l.stride + sx) - 16;
            let (u, v) = if planar {
                let c = (sy / 2) * chroma_stride + sx / 2;
                (at(y_size + c), at(v_plane + c))
            } else {
                let c = y_size + (sy / 2) * chroma_stride + (sx / 2) * 2;
                (at(c), at(c + 1))
            };
            let (d, e) = (u - 128, v - 128);
            let y = 298 * y.max(0);
            let o = (oy * ow + ox) * 4;
            if let Some(px) = pixels.get_mut(o..o + 3) {
                px[0] = clamp8((y + kr * e) >> 8);
                px[1] = clamp8((y - kgu * d - kgv * e) >> 8);
                px[2] = clamp8((y + kb * d) >> 8);
            }
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
    use super::*;

    fn gray_frame(w: usize, h: usize, y: u8) -> Frame {
        let mut data = vec![y; w * h];
        data.extend(std::iter::repeat_n(128u8, w * h / 2));
        Frame {
            pts_us: 0,
            layout: Layout {
                format: COLOR_NV12,
                width: w,
                height: h,
                stride: w,
                slice_height: h,
                left: 0,
                top: 0,
            },
            data,
        }
    }

    #[test]
    fn gray_stays_gray_and_scales_down() {
        let img = to_rgba(&gray_frame(64, 32, 126), 0, 32);
        assert_eq!((img.width, img.height), (32, 16));
        assert!(img.pixels.chunks(4).all(|p| p[0] == p[1] && p[1] == p[2] && p[3] == 255));
        assert!((img.pixels[0] as i32 - 128).abs() <= 2);
    }

    #[test]
    fn rotation_swaps_the_sides() {
        let img = to_rgba(&gray_frame(64, 32, 100), 90, 64);
        assert_eq!((img.width, img.height), (32, 64));
    }

    #[test]
    fn a_short_buffer_does_not_panic() {
        let mut f = gray_frame(64, 32, 100);
        f.data.truncate(10);
        let img = to_rgba(&f, 270, 64);
        assert_eq!(img.pixels.len(), (img.width * img.height * 4) as usize);
    }

    /// Runs on a device only: `HK_TEST_VIDEO=/sdcard/Download/testvideo.mp4 hk_media-... --ignored`.
    #[test]
    #[ignore]
    fn decodes_a_real_file() {
        let path = std::env::var("HK_TEST_VIDEO").expect("HK_TEST_VIDEO");
        let p = probe(Path::new(&path), MediaKind::Video).expect("probe");
        eprintln!("probe: {:?} {:?} {:?}", p.size, p.duration.secs(), p.alpha);
        let mut src = VideoSource::open(Path::new(&path)).expect("open");
        for t in [0.0, 0.5, 1.0, 1.03, 3.0, 0.2, 5.9] {
            let started = Instant::now();
            let img = src.frame_at(t, 640).expect("frame");
            let mid = ((img.height / 2 * img.width + img.width / 2) * 4) as usize;
            if t == 0.5 {
                if let Some(out) = std::env::var_os("HK_TEST_PNG") {
                    let _ = image::RgbaImage::from_raw(img.width, img.height, img.pixels.clone())
                        .map(|i| i.save(out));
                }
            }
            eprintln!(
                "t={t}: {}x{} px={:?} in {:?}",
                img.width,
                img.height,
                &img.pixels[mid..mid + 3],
                started.elapsed()
            );
        }
    }
}
