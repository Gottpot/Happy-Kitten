//! MP4 export through MediaCodec (H.264 / HEVC, AAC) and MediaMuxer. Frames arrive as RGBA8,
//! audio as interleaved stereo f32; the API mirrors the macOS writer so `export.rs` is shared.

use crate::MediaError;
use crate::ndk::{Codec as NdkCodec, FLAG_CONFIG, FLAG_EOS, Format, Muxer, Output};
use hk_core::Fps;
use std::path::Path;
use std::time::{Duration, Instant};

const COLOR_I420: i32 = 19;
const COLOR_NV12: i32 = 21;
/// Longest wait for a free codec buffer before the export is called stuck.
const STALL: Duration = Duration::from_secs(15);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Codec {
    H264,
    Hevc,
}

/// What to write: picture size, frame rate, video bit rate and codec, and optionally an AAC track.
#[derive(Clone, Copy, Debug)]
pub struct WriterConfig {
    pub width: u32,
    pub height: u32,
    pub fps: Fps,
    /// Video bit rate in bit/s.
    pub bitrate: u32,
    pub codec: Codec,
    /// `Some((sample rate in Hz, bit rate in bit/s))` adds an AAC track.
    pub audio: Option<(u32, u32)>,
}

fn err(msg: impl Into<String>) -> MediaError {
    MediaError::Decode(msg.into())
}

struct Held {
    video: bool,
    data: Vec<u8>,
    pts: i64,
    flags: u32,
}

pub struct VideoWriter {
    muxer: Option<Muxer>,
    venc: NdkCodec,
    aenc: Option<NdkCodec>,
    width: usize,
    height: usize,
    /// Planar (19) or semi-planar (21) input, whichever the encoder accepted.
    color: i32,
    fps: Fps,
    audio_rate: u32,
    audio_frames: u64,
    v_fmt: Option<Format>,
    a_fmt: Option<Format>,
    v_track: usize,
    a_track: usize,
    started: bool,
    held: Vec<Held>,
    v_in: Option<usize>,
    a_in: Option<usize>,
    v_input_done: bool,
    a_input_done: bool,
    v_out_done: bool,
    a_out_done: bool,
    last_video_pts: i64,
    error: Option<String>,
}

fn video_encoder(config: &WriterConfig, w: usize, h: usize) -> Result<(NdkCodec, i32), MediaError> {
    let mime = match config.codec {
        Codec::H264 => "video/avc",
        Codec::Hevc => "video/hevc",
    };
    let mut last = String::new();
    for color in [COLOR_NV12, COLOR_I420] {
        let mut f = Format::new();
        f.set_str("mime", mime);
        f.set_i32("width", w as i32);
        f.set_i32("height", h as i32);
        f.set_i32("bitrate", config.bitrate.min(i32::MAX as u32) as i32);
        f.set_i32(
            "frame-rate",
            (config.fps.num as f64 / config.fps.den.max(1) as f64).round().max(1.0) as i32,
        );
        f.set_i32("i-frame-interval", 1);
        f.set_i32("color-format", color);
        f.set_i32("color-standard", 1); // BT.709
        f.set_i32("color-range", 2); // limited
        let tried = NdkCodec::encoder(mime).and_then(|mut c| {
            c.configure(&f, true)?;
            c.start()?;
            Ok(c)
        });
        match tried {
            Ok(c) => return Ok((c, color)),
            Err(e) => last = e,
        }
    }
    Err(err(format!("Video-Encoder: {last}")))
}

fn audio_encoder(rate: u32, bitrate: u32) -> Result<NdkCodec, MediaError> {
    let mut f = Format::new();
    f.set_str("mime", "audio/mp4a-latm");
    f.set_i32("sample-rate", rate as i32);
    f.set_i32("channel-count", 2);
    f.set_i32("bitrate", bitrate.min(i32::MAX as u32) as i32);
    f.set_i32("aac-profile", 2); // AAC-LC
    f.set_i32("max-input-size", 1 << 16);
    let mut c = NdkCodec::encoder("audio/mp4a-latm").map_err(err)?;
    c.configure(&f, true).map_err(err)?;
    c.start().map_err(err)?;
    Ok(c)
}

impl VideoWriter {
    pub fn create(path: &Path, config: &WriterConfig) -> Result<VideoWriter, MediaError> {
        // Encoders want even sizes; the export renders at `size()`.
        let width = ((config.width.max(16) + 1) & !1) as usize;
        let height = ((config.height.max(16) + 1) & !1) as usize;
        let (venc, color) = video_encoder(config, width, height)?;
        let aenc = match config.audio {
            Some((rate, bitrate)) => Some(audio_encoder(rate, bitrate)?),
            None => None,
        };
        let muxer = Muxer::create(path).map_err(err)?;
        Ok(VideoWriter {
            muxer: Some(muxer),
            venc,
            aenc,
            width,
            height,
            color,
            fps: config.fps,
            audio_rate: config.audio.map_or(48_000, |a| a.0),
            audio_frames: 0,
            v_fmt: None,
            a_fmt: None,
            v_track: 0,
            a_track: 0,
            started: false,
            held: Vec::new(),
            v_in: None,
            a_in: None,
            v_input_done: false,
            a_input_done: false,
            v_out_done: false,
            a_out_done: false,
            last_video_pts: 0,
            error: None,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width as u32, self.height as u32)
    }

    pub fn check(&self) -> Result<(), MediaError> {
        match &self.error {
            Some(e) => Err(err(format!("Export fehlgeschlagen: {e}"))),
            None => Ok(()),
        }
    }

    fn fail(&mut self, e: impl Into<String>) {
        if self.error.is_none() {
            self.error = Some(e.into());
        }
    }

    fn try_start(&mut self) {
        if self.started || self.v_fmt.is_none() || (self.aenc.is_some() && self.a_fmt.is_none()) {
            return;
        }
        let Some(mux) = self.muxer.as_mut() else {
            return;
        };
        let mut go = || -> Result<(), String> {
            if let Some(f) = &self.v_fmt {
                self.v_track = mux.add_track(f)?;
            }
            if let Some(f) = &self.a_fmt {
                self.a_track = mux.add_track(f)?;
            }
            mux.start()?;
            for h in self.held.drain(..) {
                let track = if h.video { self.v_track } else { self.a_track };
                mux.write(track, &h.data, h.pts, h.flags)?;
            }
            Ok(())
        };
        match go() {
            Ok(()) => self.started = true,
            Err(e) => self.fail(e),
        }
    }

    /// Moves everything the encoders have finished into the file (or holds it until the file
    /// knows both tracks).
    fn drain(&mut self) {
        for video in [true, false] {
            loop {
                let codec = if video { Some(&mut self.venc) } else { self.aenc.as_mut() };
                let Some(codec) = codec else {
                    break;
                };
                let out = match codec.dequeue_output(0) {
                    Ok(o) => o,
                    Err(e) => {
                        self.fail(e);
                        return;
                    }
                };
                match out {
                    Output::TryAgain => break,
                    Output::FormatChanged => {
                        let f = codec.output_format();
                        if video {
                            self.v_fmt = f;
                        } else {
                            self.a_fmt = f;
                        }
                        self.try_start();
                    }
                    Output::Buffer(b) => {
                        let data = if b.size > 0 && b.flags & FLAG_CONFIG == 0 {
                            codec.output_bytes(&b)
                        } else {
                            Vec::new()
                        };
                        codec.release_output(b.index);
                        if b.flags & FLAG_EOS != 0 {
                            if video {
                                self.v_out_done = true;
                            } else {
                                self.a_out_done = true;
                            }
                        }
                        if data.is_empty() {
                            continue;
                        }
                        let flags = b.flags & !FLAG_EOS;
                        if self.started {
                            let track = if video { self.v_track } else { self.a_track };
                            if let Some(mux) = self.muxer.as_mut() {
                                if let Err(e) = mux.write(track, &data, b.pts, flags) {
                                    self.fail(e);
                                    return;
                                }
                            }
                        } else {
                            self.held.push(Held {
                                video,
                                data,
                                pts: b.pts,
                                flags,
                            });
                        }
                    }
                }
            }
        }
    }

    pub fn video_ready(&mut self) -> bool {
        self.drain();
        if self.v_in.is_none() {
            self.v_in = self.venc.dequeue_input(0);
        }
        self.v_in.is_some()
    }

    pub fn audio_ready(&mut self) -> bool {
        self.drain();
        if self.a_in.is_none() {
            self.a_in = self.aenc.as_mut().and_then(|a| a.dequeue_input(0));
        }
        self.a_in.is_some()
    }

    pub fn has_audio(&self) -> bool {
        self.aenc.is_some()
    }

    fn wait_video_input(&mut self) -> Result<usize, MediaError> {
        let start = Instant::now();
        loop {
            self.check()?;
            if let Some(i) = self.v_in.take().or_else(|| self.venc.dequeue_input(5_000)) {
                return Ok(i);
            }
            self.drain();
            if start.elapsed() > STALL {
                return Err(err("Der Encoder antwortet nicht mehr"));
            }
        }
    }

    fn wait_audio_input(&mut self) -> Result<usize, MediaError> {
        let start = Instant::now();
        loop {
            self.check()?;
            let taken = self.a_in.take();
            let got = taken.or_else(|| self.aenc.as_mut().and_then(|a| a.dequeue_input(5_000)));
            if let Some(i) = got {
                return Ok(i);
            }
            self.drain();
            if start.elapsed() > STALL {
                return Err(err("Der Audio-Encoder antwortet nicht mehr"));
            }
        }
    }

    /// No more sound will follow.
    pub fn finish_audio(&mut self) {
        if self.a_input_done || self.aenc.is_none() {
            return;
        }
        match self.wait_audio_input() {
            Ok(i) => {
                let pts = self.audio_pts();
                if let Some(a) = self.aenc.as_mut() {
                    if let Err(e) = a.queue_input(i, 0, pts, FLAG_EOS) {
                        self.fail(e);
                    }
                }
                self.a_input_done = true;
            }
            Err(e) => self.fail(e.to_string()),
        }
    }

    /// No more picture will follow.
    pub fn finish_video(&mut self) {
        if self.v_input_done {
            return;
        }
        match self.wait_video_input() {
            Ok(i) => {
                let pts = self.last_video_pts;
                if let Err(e) = self.venc.queue_input(i, 0, pts, FLAG_EOS) {
                    self.fail(e);
                }
                self.v_input_done = true;
            }
            Err(e) => self.fail(e.to_string()),
        }
    }

    fn audio_pts(&self) -> i64 {
        (self.audio_frames as i128 * 1_000_000 / self.audio_rate.max(1) as i128) as i64
    }

    /// Append frame number `index`. `rgba` must be `width * height * 4` bytes (the size from
    /// [`VideoWriter::size`]).
    pub fn push_video(&mut self, rgba: &[u8], index: i64) -> Result<(), MediaError> {
        let (w, h) = (self.width, self.height);
        if rgba.len() < w * h * 4 {
            return Err(err("Bild zu klein für den Export"));
        }
        let i = self.wait_video_input()?;
        let color = self.color;
        let buf = self.venc.input_buffer(i);
        let need = w * h * 3 / 2;
        if buf.len() < need {
            return Err(err("Der Encoder-Puffer ist zu klein für das Bild"));
        }
        rgba_to_yuv(rgba, w, h, color == COLOR_I420, buf);
        let pts = index.max(0) as i128 * 1_000_000 * self.fps.den.max(1) as i128
            / self.fps.num.max(1) as i128;
        self.last_video_pts = pts as i64;
        self.venc
            .queue_input(i, need, pts as i64, 0)
            .map_err(err)?;
        self.drain();
        self.check()
    }

    /// Append interleaved stereo samples.
    pub fn push_audio(&mut self, stereo: &[f32]) -> Result<(), MediaError> {
        if self.aenc.is_none() {
            return Ok(());
        }
        let mut pcm: Vec<u8> = Vec::with_capacity(stereo.len() * 2);
        for s in stereo {
            let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
            pcm.extend_from_slice(&v.to_le_bytes());
        }
        let mut offset = 0;
        while offset < pcm.len() {
            let i = self.wait_audio_input()?;
            let pts = self.audio_pts();
            let Some(a) = self.aenc.as_mut() else {
                return Ok(());
            };
            let buf = a.input_buffer(i);
            // whole stereo frames only (4 bytes)
            let n = (pcm.len() - offset).min(buf.len() & !3);
            if n == 0 {
                return Err(err("Der Audio-Puffer ist zu klein"));
            }
            buf[..n].copy_from_slice(&pcm[offset..offset + n]);
            a.queue_input(i, n, pts, 0).map_err(err)?;
            offset += n;
            self.audio_frames += (n / 4) as u64;
            self.drain();
        }
        self.check()
    }

    /// Finishes the file.
    pub fn finish(mut self) -> Result<(), MediaError> {
        self.finish_video();
        self.finish_audio();
        let start = Instant::now();
        while !(self.v_out_done && (self.aenc.is_none() || self.a_out_done)) {
            self.check()?;
            self.drain();
            if start.elapsed() > STALL {
                return Err(err("Der Encoder wird nicht fertig"));
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        self.check()?;
        if !self.started {
            return Err(err("Es wurde kein Bild geschrieben"));
        }
        match self.muxer.as_mut() {
            Some(m) => m.stop().map_err(err),
            None => Ok(()),
        }
    }

    /// Throws the half-written file away.
    pub fn cancel(self, path: &Path) {
        drop(self);
        let _ = std::fs::remove_file(path);
    }
}

/// RGBA → planar or semi-planar 4:2:0, BT.709 limited range. Chroma is the average of each
/// 2×2 block.
fn rgba_to_yuv(rgba: &[u8], w: usize, h: usize, planar: bool, out: &mut [u8]) {
    let (y_plane, chroma) = out.split_at_mut(w * h);
    for (row, y_row) in y_plane.chunks_exact_mut(w).enumerate().take(h) {
        let src = rgba.get(row * w * 4..(row + 1) * w * 4).unwrap_or(&[]);
        for (x, y) in y_row.iter_mut().enumerate() {
            let p = src.get(x * 4..x * 4 + 3).unwrap_or(&[0, 0, 0]);
            let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
            *y = (16 + ((47 * r + 157 * g + 16 * b + 128) >> 8)).clamp(16, 235) as u8;
        }
    }
    let (cw, ch) = (w / 2, h / 2);
    for cy in 0..ch {
        for cx in 0..cw {
            let (mut r, mut g, mut b) = (0i32, 0i32, 0i32);
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let i = ((cy * 2 + dy) * w + cx * 2 + dx) * 4;
                if let Some(p) = rgba.get(i..i + 3) {
                    r += p[0] as i32;
                    g += p[1] as i32;
                    b += p[2] as i32;
                }
            }
            let (r, g, b) = (r / 4, g / 4, b / 4);
            let u = (128 + ((-26 * r - 87 * g + 112 * b + 128) >> 8)).clamp(16, 240) as u8;
            let v = (128 + ((112 * r - 102 * g - 10 * b + 128) >> 8)).clamp(16, 240) as u8;
            if planar {
                let i = cy * cw + cx;
                if let Some(d) = chroma.get_mut(i) {
                    *d = u;
                }
                if let Some(d) = chroma.get_mut(cw * ch + i) {
                    *d = v;
                }
            } else {
                let i = (cy * cw + cx) * 2;
                if let Some(d) = chroma.get_mut(i) {
                    *d = u;
                }
                if let Some(d) = chroma.get_mut(i + 1) {
                    *d = v;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hk_core::Fps;

    #[test]
    fn white_and_black_map_to_the_limits() {
        let (w, h) = (4, 4);
        for (value, y_expect) in [(255u8, 235u8), (0, 16)] {
            let rgba = vec![value; w * h * 4];
            let mut out = vec![0u8; w * h * 3 / 2];
            rgba_to_yuv(&rgba, w, h, false, &mut out);
            assert!(out[..w * h].iter().all(|y| (*y as i32 - y_expect as i32).abs() <= 1));
            assert!(out[w * h..].iter().all(|c| (*c as i32 - 128).abs() <= 1));
        }
    }

    #[test]
    fn planar_and_semi_planar_hold_the_same_chroma() {
        let (w, h) = (4, 2);
        let rgba: Vec<u8> = (0..w * h).flat_map(|_| [200u8, 40, 20, 255]).collect();
        let (mut a, mut b) = (vec![0u8; w * h * 3 / 2], vec![0u8; w * h * 3 / 2]);
        rgba_to_yuv(&rgba, w, h, true, &mut a);
        rgba_to_yuv(&rgba, w, h, false, &mut b);
        assert_eq!(a[w * h], b[w * h]); // U of the first block
        assert_eq!(a[w * h + 2], b[w * h + 1]); // V of the first block
    }

    /// Runs on a device only: `HK_TEST_OUT=/data/local/tmp/out.mp4 hk_media-... --ignored`.
    #[test]
    #[ignore]
    fn writes_a_playable_file() {
        let out = std::path::PathBuf::from(std::env::var("HK_TEST_OUT").expect("HK_TEST_OUT"));
        let (w, h, fps, frames) = (640u32, 360u32, Fps::FPS_30, 90i64);
        let cfg = WriterConfig {
            width: w,
            height: h,
            fps,
            bitrate: 4_000_000,
            codec: Codec::H264,
            audio: Some((48_000, 128_000)),
        };
        let mut writer = VideoWriter::create(&out, &cfg).expect("create");
        let (w, h) = writer.size();
        let per_frame = 48_000 / 30;
        let (mut v, mut a) = (0i64, 0i64);
        while v < frames || a < frames {
            if v < frames && writer.video_ready() {
                let mut px = vec![255u8; (w * h * 4) as usize];
                for (i, p) in px.chunks_exact_mut(4).enumerate() {
                    let x = (i as u32 % w) as f32 / w as f32;
                    p[0] = (x * 255.0) as u8;
                    p[1] = (v as f32 / frames as f32 * 255.0) as u8;
                    p[2] = 80;
                }
                writer.push_video(&px, v).expect("video");
                v += 1;
                if v >= frames {
                    writer.finish_video();
                }
            }
            if a < frames && writer.audio_ready() {
                let mut buf = vec![0f32; per_frame * 2];
                for (i, s) in buf.chunks_exact_mut(2).enumerate() {
                    let t = (a as usize * per_frame + i) as f32 / 48_000.0;
                    s[0] = (t * 440.0 * std::f32::consts::TAU).sin() * 0.3;
                    s[1] = s[0];
                }
                writer.push_audio(&buf).expect("audio");
                a += 1;
                if a >= frames {
                    writer.finish_audio();
                }
            }
        }
        writer.finish().expect("finish");
        eprintln!("written: {} bytes", std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0));
    }
}
