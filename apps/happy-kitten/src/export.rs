//! Offline export to MP4: renders every frame through the same GPU compositor
//! as the preview, encodes with the Mac's hardware encoder and mixes the sound.

use crate::layers::{self, ImageCache};
use hk_audio::{ClipPlan, Pcm, mix_block};
use hk_core::{
    AudioLevels, Id, Layer, MediaKind, MediaSource, Project, SourceRef, Tick, VideoFrame,
};
use hk_gpu::{Compositor, GpuFrame};
use hk_media::{Codec, VideoSource, VideoWriter};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ExportSettings {
    pub width: u32,
    pub height: u32,
    pub bitrate: u32,
    pub codec: Codec,
    pub path: PathBuf,
    /// Frames per second of the finished film (the timeline is sampled at this rate).
    pub fps: hk_core::Fps,
    /// Sample rate of the AAC track and its bit rate in bit/s.
    pub audio_rate: u32,
    pub audio_bitrate: u32,
}

pub struct ExportJob {
    pub settings: ExportSettings,
    pub total: u32,
    pub progress: Arc<AtomicU32>,
    pub cancel: Arc<AtomicBool>,
    pub started: Instant,
    result: Receiver<Result<(), String>>,
}

impl ExportJob {
    pub fn fraction(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / self.total.max(1) as f32
    }

    /// `Some` once the worker is done.
    pub fn poll(&self) -> Option<Result<(), String>> {
        self.result.try_recv().ok()
    }
}

pub fn frame_count(project: &Project, fps: hk_core::Fps) -> u32 {
    let tpf = fps.ticks_per_frame().max(1);
    ((project.duration().0 + tpf - 1) / tpf).clamp(0, u32::MAX as i64) as u32
}

pub fn start(
    project: Project,
    plan: Vec<ClipPlan>,
    pcm: HashMap<Id, Arc<Pcm>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    settings: ExportSettings,
) -> ExportJob {
    let total = frame_count(&project, settings.fps);
    let progress = Arc::new(AtomicU32::new(0));
    let cancel = Arc::new(AtomicBool::new(false));
    let (tx, rx) = channel();
    let (p2, c2, s2) = (progress.clone(), cancel.clone(), settings.clone());
    std::thread::Builder::new()
        .name("hk-export".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run(&project, &plan, &pcm, device, queue, &s2, total, &p2, &c2)
            }))
            .unwrap_or_else(|_| Err("Beim Export ist etwas schiefgelaufen".into()));
            let _ = tx.send(result);
        })
        .ok();
    ExportJob {
        settings,
        total,
        progress,
        cancel,
        started: Instant::now(),
        result: rx,
    }
}

/// Frames the exporter needs, decoded at export size.
struct Source {
    max_dim: u32,
    stills: HashMap<Id, Rc<GpuFrame>>,
    readers: HashMap<PathBuf, Option<VideoSource>>,
    last_video: HashMap<Id, (i64, Rc<GpuFrame>)>,
    texts: HashMap<u64, Rc<GpuFrame>>,
    images: ImageCache,
}

impl Source {
    fn get(
        &mut self,
        comp: &Compositor,
        project: &Project,
        src: SourceRef,
    ) -> Option<Rc<GpuFrame>> {
        let m = project.media(src.media)?;
        match (&m.source, m.kind) {
            (MediaSource::Sample(i), _) => {
                self.stills.entry(m.id).or_insert_with(|| {
                    let img = hk_media::sample_frame(*i, 1280, 720);
                    Rc::new(comp.upload(&img.pixels, img.width, img.height))
                });
                self.stills.get(&m.id).cloned()
            }
            (MediaSource::File(path), MediaKind::Image) => {
                if !self.stills.contains_key(&m.id) {
                    let img = hk_media::load_image(path, self.max_dim).ok()?;
                    self.stills.insert(
                        m.id,
                        Rc::new(comp.upload(&img.pixels, img.width, img.height)),
                    );
                }
                self.stills.get(&m.id).cloned()
            }
            (MediaSource::File(path), MediaKind::Video) => {
                let bucket = (src.time.secs() * 240.0).round() as i64;
                if let Some((b, f)) = self.last_video.get(&m.id)
                    && *b == bucket
                {
                    return Some(f.clone());
                }
                let reader = self
                    .readers
                    .entry(path.clone())
                    .or_insert_with(|| VideoSource::open(path).ok())
                    .as_mut()?;
                let img = reader.frame_at(src.time.secs(), self.max_dim).ok()?;
                let frame = Rc::new(comp.upload(&img.pixels, img.width, img.height));
                self.last_video.insert(m.id, (bucket, frame.clone()));
                Some(frame)
            }
            _ => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    project: &Project,
    plan: &[ClipPlan],
    pcm: &HashMap<Id, Arc<Pcm>>,
    device: eframe::wgpu::Device,
    queue: eframe::wgpu::Queue,
    s: &ExportSettings,
    total: u32,
    progress: &AtomicU32,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let comp = Compositor::new(device, queue);
    comp.set_fit(project.fit);
    let out = comp.create_output(s.width, s.height);
    let has_audio = !plan.is_empty();
    let mut writer = VideoWriter::create(
        &s.path,
        &hk_media::WriterConfig {
            width: s.width,
            height: s.height,
            fps: s.fps,
            bitrate: s.bitrate,
            codec: s.codec,
            audio: has_audio.then_some((s.audio_rate, s.audio_bitrate)),
        },
    )
    .map_err(|e| e.to_string())?;
    let (w, h) = writer.size();
    // The writer rounds odd sizes down to even; render at exactly that size.
    let out = if (w, h) == out.size() {
        out
    } else {
        comp.create_output(w, h)
    };
    let mut source = Source {
        max_dim: w.max(h),
        stills: HashMap::new(),
        readers: HashMap::new(),
        last_video: HashMap::new(),
        texts: HashMap::new(),
        images: ImageCache::default(),
    };
    let fps = s.fps;
    let audio_rate = s.audio_rate;
    let audio_end = |i: i64| {
        (i as f64 * audio_rate as f64 * fps.den as f64 / fps.num.max(1) as f64).round() as u64
    };
    let frame_secs = |i: i64| i as f64 * fps.den as f64 / fps.num.max(1) as f64;
    let total = total as i64;
    let (mut next_video, mut next_audio) = (0i64, if has_audio { 0i64 } else { total });
    let mut pending_video: Option<Vec<u8>> = None;
    let mut pending_audio: Option<Vec<f32>> = None;
    let mut stalled = 0u32;
    // AVAssetWriter wants audio and picture interleaved by time: always feed whichever input
    // is behind, and the other one only while it is at most a second ahead.
    while next_video < total || next_audio < total {
        if cancel.load(Ordering::Relaxed) {
            writer.cancel(&s.path);
            return Err("Abgebrochen".into());
        }
        writer.check().map_err(|e| e.to_string())?;
        let video_first = next_video < total
            && (next_audio >= total || frame_secs(next_video) <= frame_secs(next_audio));
        let mut progressed = false;
        for pass in 0..2 {
            let try_video = (pass == 0) == video_first;
            if try_video && next_video < total {
                if pass == 1 && frame_secs(next_video) > frame_secs(next_audio.min(total)) + 1.0 {
                    continue;
                }
                if pending_video.is_none() {
                    pending_video = Some(render_frame(
                        &comp,
                        &out,
                        project,
                        &mut source,
                        next_video,
                        fps,
                        (plan, pcm),
                    )?);
                }
                if writer.video_ready() {
                    if let Some(px) = pending_video.take() {
                        writer
                            .push_video(&px, next_video)
                            .map_err(|e| e.to_string())?;
                    }
                    next_video += 1;
                    progress.store(next_video as u32, Ordering::Relaxed);
                    progressed = true;
                    if next_video >= total {
                        writer.finish_video();
                    }
                }
            } else if !try_video && next_audio < total {
                if pass == 1 && frame_secs(next_audio) > frame_secs(next_video.min(total)) + 1.0 {
                    continue;
                }
                if pending_audio.is_none() {
                    let (a0, a1) = (audio_end(next_audio), audio_end(next_audio + 1));
                    let mut buf = vec![0.0; ((a1 - a0) * 2) as usize];
                    mix_block(plan, pcm, audio_rate, a0, &mut buf);
                    pending_audio = Some(buf);
                }
                if writer.audio_ready() {
                    if let Some(buf) = pending_audio.take() {
                        writer.push_audio(&buf).map_err(|e| e.to_string())?;
                    }
                    next_audio += 1;
                    progressed = true;
                    if next_audio >= total {
                        writer.finish_audio();
                    }
                }
            }
            if progressed {
                break;
            }
        }
        if progressed {
            stalled = 0;
        } else {
            stalled += 1;
            if stalled > 15_000 {
                return Err("Der Encoder antwortet nicht mehr".into());
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    writer.finish().map_err(|e| e.to_string())
}

pub fn reveal_in_finder(path: &Path) {
    let _ = std::process::Command::new("open")
        .arg("-R")
        .arg(path)
        .spawn();
}

/// Render frame `i` of the project and read it back as RGBA.
fn render_frame(
    comp: &Compositor,
    out: &hk_gpu::Output,
    project: &Project,
    source: &mut Source,
    i: i64,
    fps: hk_core::Fps,
    audio: (&[ClipPlan], &HashMap<Id, Arc<Pcm>>),
) -> Result<Vec<u8>, String> {
    let t = Tick::from_frames(i, fps);
    let scene = project.scene_at(t);
    match scene.base {
        VideoFrame::Black => comp.render_black(out),
        VideoFrame::Single(src) => match source.get(comp, project, src) {
            Some(f) => {
                comp.set_grades(src.grade, src.grade);
                comp.render_single(out, &f)
            }
            None => comp.render_black(out),
        },
        VideoFrame::Blend(span) => match (
            source.get(comp, project, span.from),
            source.get(comp, project, span.to),
        ) {
            (Some(a), Some(b)) => {
                comp.set_grades(span.from.grade, span.to.grade);
                comp.render_blend(out, &a, &b, span.kind, span.progress, t.secs() as f32)
            }
            (Some(only), None) => {
                comp.set_grades(span.from.grade, span.from.grade);
                comp.render_single(out, &only)
            }
            (None, Some(only)) => {
                comp.set_grades(span.to.grade, span.to.grade);
                comp.render_single(out, &only)
            }
            (None, None) => comp.render_black(out),
        },
    }
    let (w, h) = out.size();
    let mut levels: Option<AudioLevels> = None;
    for layer in &scene.layers {
        if let Layer::Effect(e) = layer {
            let l = if e.spec.needs_audio() {
                *levels.get_or_insert_with(|| hk_audio::analyze(audio.0, audio.1, t.secs()))
            } else {
                AudioLevels::SILENT
            };
            comp.overlay_fx(out, &e.spec, e.time, e.alpha, e.seed, &l);
            continue;
        }
        let key = layers::key(layer);
        if !source.texts.contains_key(&key) {
            if source.texts.len() >= 16 {
                source.texts.clear();
            }
            if let Some(r) = layers::raster(layer, project, &mut source.images, w, h) {
                source
                    .texts
                    .insert(key, Rc::new(comp.upload(&r.pixels, w, h)));
            }
        }
        if let Some(frame) = source.texts.get(&key) {
            let (fx, progress, anchor) = layers::animation(layer);
            comp.overlay_text(out, frame, fx, progress, t.secs() as f32, anchor);
        }
    }
    comp.read_output(out)
}
