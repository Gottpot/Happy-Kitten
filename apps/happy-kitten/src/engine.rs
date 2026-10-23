//! Preview engine: decodes frames on a worker thread, keeps recent ones on the
//! GPU and renders the picture for the playhead through the compositor.

use crate::layers::{self, ImageCache};
use eframe::egui_wgpu::RenderState;
use hk_core::{Id, MediaKind, MediaSource, Project, SourceRef, Tick, TransitionKind, VideoFrame};
use hk_gpu::{Compositor, GpuFrame, Output};
use hk_media::Rgba;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};

/// Longest side of decoded preview frames.
const PREVIEW_DIM: u32 = 1280;
const THUMB_DIM: u32 = 256;
/// Video frames are cached per 1/30 s.
const BUCKETS_PER_SEC: f64 = 30.0;
const CACHE_BUDGET: usize = 600 * 1024 * 1024;
const MAX_STACK: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct FrameKey {
    media: Id,
    bucket: i64,
}

enum Job {
    Frame {
        key: FrameKey,
        source: MediaSource,
        kind: MediaKind,
        secs: f64,
        epoch: u64,
    },
    Thumb {
        media: Id,
        source: MediaSource,
        kind: MediaKind,
        epoch: u64,
    },
}

struct Done {
    epoch: u64,
    key: Option<FrameKey>,
    thumb_for: Option<Id>,
    result: Result<Rgba, String>,
}

struct Cached {
    frame: GpuFrame,
    used: u64,
}

pub struct Engine {
    comp: Compositor,
    preview: Output,
    preview_tex: egui::TextureId,
    render_state: RenderState,
    cache: HashMap<FrameKey, Cached>,
    cache_bytes: usize,
    pending: HashSet<FrameKey>,
    thumb_pending: HashSet<Id>,
    thumbs: HashMap<Id, egui::TextureHandle>,
    text_cache: HashMap<(u64, u32, u32), (GpuFrame, u64)>,
    images: ImageCache,
    effect_cards: Vec<(TransitionKind, Output, egui::TextureId)>,
    card_a: GpuFrame,
    card_b: GpuFrame,
    jobs: Sender<Job>,
    done: Receiver<Done>,
    tick: u64,
    /// Bumped when the project is replaced; results of older requests are dropped.
    epoch: u64,
    pub errors: Vec<String>,
}

impl Engine {
    pub fn new(render_state: &RenderState, size: (u32, u32)) -> Self {
        let comp = Compositor::new(render_state.device.clone(), render_state.queue.clone());
        let preview = comp.create_output(size.0, size.1);
        let preview_tex = register(render_state, &preview);
        let (card_w, card_h) = (224, 126);
        let card_a = comp.upload(
            &hk_media::sample_frame(0, card_w, card_h).pixels,
            card_w,
            card_h,
        );
        let card_b = comp.upload(
            &hk_media::sample_frame(2, card_w, card_h).pixels,
            card_w,
            card_h,
        );
        let effect_cards = TransitionKind::ALL
            .iter()
            .map(|k| {
                let out = comp.create_output(card_w, card_h);
                comp.render_blend(&out, &card_a, &card_b, *k, 0.5, 0.5);
                let id = register(render_state, &out);
                (*k, out, id)
            })
            .collect();
        let (jobs, job_rx) = channel();
        let (done_tx, done) = channel();
        std::thread::Builder::new()
            .name("hk-decode".into())
            .spawn(move || worker(job_rx, done_tx))
            .ok();
        Engine {
            comp,
            preview,
            preview_tex,
            render_state: render_state.clone(),
            cache: HashMap::new(),
            cache_bytes: 0,
            pending: HashSet::new(),
            thumb_pending: HashSet::new(),
            thumbs: HashMap::new(),
            text_cache: HashMap::new(),
            images: ImageCache::default(),
            effect_cards,
            card_a,
            card_b,
            jobs,
            done,
            tick: 0,
            epoch: 0,
            errors: Vec::new(),
        }
    }

    /// Forget everything cached; call when another project is loaded.
    pub fn reset(&mut self) {
        self.epoch += 1;
        self.cache.clear();
        self.cache_bytes = 0;
        self.pending.clear();
        self.thumb_pending.clear();
        self.thumbs.clear();
        self.text_cache.clear();
        self.images.clear();
    }

    pub fn gpu(&self) -> (eframe::wgpu::Device, eframe::wgpu::Queue) {
        (self.comp.device().clone(), self.comp.queue().clone())
    }

    /// Make the preview as large as the project canvas (it changes with 16:9 / 9:16 / ...).
    pub fn ensure_size(&mut self, size: (u32, u32)) {
        if self.preview.size() == size {
            return;
        }
        let out = self.comp.create_output(size.0, size.1);
        let tex = register(&self.render_state, &out);
        self.render_state
            .renderer
            .write()
            .free_texture(&self.preview_tex);
        self.preview = out;
        self.preview_tex = tex;
        self.text_cache.clear();
    }

    pub fn preview_texture(&self) -> (egui::TextureId, (u32, u32)) {
        (self.preview_tex, self.preview.size())
    }

    pub fn thumb(&self, media: Id) -> Option<&egui::TextureHandle> {
        self.thumbs.get(&media)
    }

    pub fn effect_card(&self, kind: TransitionKind) -> Option<egui::TextureId> {
        self.effect_cards
            .iter()
            .find(|(k, _, _)| *k == kind)
            .map(|(_, _, id)| *id)
    }

    /// Re-render one effect card at `progress` (hover animation).
    pub fn animate_effect_card(&self, kind: TransitionKind, progress: f32) {
        if let Some((_, out, _)) = self.effect_cards.iter().find(|(k, _, _)| *k == kind) {
            // the cards are never colour graded; the next preview render sets its own grades
            self.comp.set_grades(Default::default(), Default::default());
            self.comp
                .render_blend(out, &self.card_a, &self.card_b, kind, progress, progress);
        }
    }

    /// Ask for a thumbnail of a media item once.
    pub fn want_thumb(&mut self, project: &Project, media: Id) {
        if self.thumbs.contains_key(&media) || !self.thumb_pending.insert(media) {
            return;
        }
        if let Some(m) = project.media(media)
            && m.kind != MediaKind::Audio
        {
            let _ = self.jobs.send(Job::Thumb {
                media,
                source: m.source.clone(),
                kind: m.kind,
                epoch: self.epoch,
            });
        }
    }

    /// Thumbnail for something that is not a project media entry (global library).
    pub fn want_thumb_item(&mut self, id: Id, source: MediaSource, kind: MediaKind) {
        if self.thumbs.contains_key(&id)
            || kind == MediaKind::Audio
            || !self.thumb_pending.insert(id)
        {
            return;
        }
        let _ = self.jobs.send(Job::Thumb {
            media: id,
            source,
            kind,
            epoch: self.epoch,
        });
    }

    /// Collect finished decodes. Returns true while anything is still in flight.
    pub fn pump(&mut self, ctx: &egui::Context) -> bool {
        self.tick += 1;
        while let Ok(done) = self.done.try_recv() {
            if done.epoch != self.epoch {
                continue;
            }
            match (done.result, done.key, done.thumb_for) {
                (Ok(img), Some(key), _) => {
                    self.pending.remove(&key);
                    let frame = self.comp.upload(&img.pixels, img.width, img.height);
                    self.cache_bytes += frame.bytes();
                    if let Some(old) = self.cache.insert(
                        key,
                        Cached {
                            frame,
                            used: self.tick,
                        },
                    ) {
                        self.cache_bytes = self.cache_bytes.saturating_sub(old.frame.bytes());
                    }
                    self.evict();
                    ctx.request_repaint();
                }
                (Ok(img), None, Some(media)) => {
                    let color = egui::ColorImage::from_rgba_unmultiplied(
                        [img.width as usize, img.height as usize],
                        &img.pixels,
                    );
                    let tex = ctx.load_texture(
                        format!("thumb-{media}"),
                        color,
                        egui::TextureOptions::LINEAR,
                    );
                    self.thumbs.insert(media, tex);
                    ctx.request_repaint();
                }
                (Err(e), key, thumb) => {
                    if let Some(k) = key {
                        self.pending.remove(&k);
                        // Remember the failure as a black frame so we do not retry forever.
                        let black = self.comp.upload(&[0, 0, 0, 255], 1, 1);
                        self.cache.insert(
                            k,
                            Cached {
                                frame: black,
                                used: self.tick,
                            },
                        );
                    }
                    let _ = thumb;
                    if !self.errors.contains(&e) && self.errors.len() < 20 {
                        self.errors.push(e);
                    }
                }
                _ => {}
            }
        }
        !self.pending.is_empty()
    }

    fn evict(&mut self) {
        while self.cache_bytes > CACHE_BUDGET {
            let Some((&oldest, _)) = self
                .cache
                .iter()
                .filter(|(_, c)| c.used + 2 < self.tick)
                .min_by_key(|(_, c)| c.used)
            else {
                break;
            };
            if let Some(c) = self.cache.remove(&oldest) {
                self.cache_bytes = self.cache_bytes.saturating_sub(c.frame.bytes());
            }
        }
    }

    fn key_for(
        project: &Project,
        src: SourceRef,
    ) -> Option<(FrameKey, MediaKind, MediaSource, f64)> {
        let m = project.media(src.media)?;
        let secs = src.time.secs();
        let bucket = if m.kind == MediaKind::Video {
            (secs * BUCKETS_PER_SEC).floor() as i64
        } else {
            0
        };
        Some((
            FrameKey {
                media: m.id,
                bucket,
            },
            m.kind,
            m.source.clone(),
            secs,
        ))
    }

    /// Make sure a frame is cached or on its way. Returns the key to draw
    /// right now: the exact frame, or the closest cached one of that media.
    fn request(&mut self, project: &Project, src: SourceRef) -> Option<FrameKey> {
        let (key, kind, source, secs) = Self::key_for(project, src)?;
        if !self.cache.contains_key(&key) && self.pending.insert(key) {
            let _ = self.jobs.send(Job::Frame {
                key,
                source,
                kind,
                secs,
                epoch: self.epoch,
            });
        }
        if let Some(c) = self.cache.get_mut(&key) {
            c.used = self.tick;
            return Some(key);
        }
        let best = self
            .cache
            .keys()
            .filter(|k| k.media == key.media)
            .min_by_key(|k| (k.bucket - key.bucket).abs())
            .copied();
        if let Some(b) = best
            && let Some(c) = self.cache.get_mut(&b)
        {
            c.used = self.tick;
        }
        best
    }

    /// Warm the cache for the frames just ahead of the playhead.
    pub fn prefetch(&mut self, project: &Project, t: Tick, frames: i64) {
        let fps = project.fps;
        for i in 1..=frames {
            let ahead = t + Tick::from_frames(i, fps);
            let src = match project.video_frame_at(ahead) {
                VideoFrame::Single(s) => Some(s),
                VideoFrame::Blend(b) => Some(b.to),
                VideoFrame::Black => None,
            };
            if let Some(src) = src
                && self.pending.len() < MAX_STACK
            {
                self.request(project, src);
            }
        }
    }

    /// Draw the picture at time `t` into the preview texture.
    pub fn render(&mut self, project: &Project, t: Tick) {
        self.comp.set_fit(project.fit);
        let scene = project.scene_at(t);
        self.render_base(project, t, scene.base);
        let (w, h) = self.preview.size();
        for layer in &scene.layers {
            let key = (layers::key(layer), w, h);
            if !self.text_cache.contains_key(&key) {
                if self.text_cache.len() >= 24
                    && let Some(oldest) = self
                        .text_cache
                        .iter()
                        .min_by_key(|(_, (_, used))| *used)
                        .map(|(k, _)| *k)
                {
                    self.text_cache.remove(&oldest);
                }
                if let Some(r) = layers::raster(layer, project, &mut self.images, w, h) {
                    self.text_cache
                        .insert(key, (self.comp.upload(&r.pixels, w, h), self.tick));
                }
            }
            if let Some((frame, used)) = self.text_cache.get_mut(&key) {
                *used = self.tick;
                let (fx, progress, anchor) = layers::animation(layer);
                self.comp
                    .overlay_text(&self.preview, frame, fx, progress, t.secs() as f32, anchor);
            }
        }
    }

    fn render_base(&mut self, project: &Project, t: Tick, base: VideoFrame) {
        match base {
            VideoFrame::Black => self.comp.render_black(&self.preview),
            VideoFrame::Single(src) => {
                self.comp.set_grades(src.grade, src.grade);
                match self.request(project, src).and_then(|k| self.cache.get(&k)) {
                    Some(c) => self.comp.render_single(&self.preview, &c.frame),
                    None => self.comp.render_black(&self.preview),
                }
            }
            VideoFrame::Blend(span) => {
                self.comp.set_grades(span.from.grade, span.to.grade);
                let ka = self.request(project, span.from);
                let kb = self.request(project, span.to);
                let a = ka.and_then(|k| self.cache.get(&k)).map(|c| &c.frame);
                let b = kb.and_then(|k| self.cache.get(&k)).map(|c| &c.frame);
                match (a, b) {
                    (Some(a), Some(b)) => self.comp.render_blend(
                        &self.preview,
                        a,
                        b,
                        span.kind,
                        span.progress,
                        t.secs() as f32,
                    ),
                    (Some(only), None) => {
                        self.comp.set_grades(span.from.grade, span.from.grade);
                        self.comp.render_single(&self.preview, only)
                    }
                    (None, Some(only)) => {
                        self.comp.set_grades(span.to.grade, span.to.grade);
                        self.comp.render_single(&self.preview, only)
                    }
                    (None, None) => self.comp.render_black(&self.preview),
                }
            }
        }
    }

    /// Current frame as RGBA for export / stills.
    pub fn read_preview(&self) -> Result<Vec<u8>, String> {
        self.comp.read_output(&self.preview)
    }
}

fn register(rs: &RenderState, out: &Output) -> egui::TextureId {
    rs.renderer.write().register_native_texture(
        &rs.device,
        out.view(),
        eframe::wgpu::FilterMode::Linear,
    )
}

fn worker(jobs: Receiver<Job>, done: Sender<Done>) {
    #[cfg(target_os = "macos")]
    let mut sources: HashMap<std::path::PathBuf, Option<hk_media::VideoSource>> = HashMap::new();
    while let Ok(first) = jobs.recv() {
        let mut stack = vec![first];
        loop {
            stack.extend(jobs.try_iter());
            if stack.len() > MAX_STACK {
                stack.drain(..stack.len() - MAX_STACK);
            }
            // Newest request first (while scrubbing the older ones are stale), but all pending
            // frames of the same media in ascending time so sequential decoding stays sequential.
            let Some(newest) = stack.pop() else { break };
            let media = job_media(&newest);
            let mut batch = vec![newest];
            let mut i = 0;
            while i < stack.len() {
                if job_media(&stack[i]) == media {
                    batch.push(stack.remove(i));
                } else {
                    i += 1;
                }
            }
            batch.sort_by(|a, b| job_secs(a).total_cmp(&job_secs(b)));
            for job in batch {
                let (epoch, key, thumb_for, source, kind, secs, dim) = match job {
                    Job::Frame {
                        key,
                        source,
                        kind,
                        secs,
                        epoch,
                    } => (epoch, Some(key), None, source, kind, secs, PREVIEW_DIM),
                    Job::Thumb {
                        media,
                        source,
                        kind,
                        epoch,
                    } => (epoch, None, Some(media), source, kind, 0.0, THUMB_DIM),
                };
                let result = decode(
                    &source,
                    kind,
                    secs,
                    dim,
                    #[cfg(target_os = "macos")]
                    &mut sources,
                );
                if done
                    .send(Done {
                        epoch,
                        key,
                        thumb_for,
                        result,
                    })
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

fn job_media(job: &Job) -> (Id, bool) {
    match job {
        Job::Frame { key, .. } => (key.media, false),
        Job::Thumb { media, .. } => (*media, true),
    }
}

fn job_secs(job: &Job) -> f64 {
    match job {
        Job::Frame { secs, .. } => *secs,
        Job::Thumb { .. } => 0.0,
    }
}

fn decode(
    source: &MediaSource,
    kind: MediaKind,
    secs: f64,
    dim: u32,
    #[cfg(target_os = "macos")] sources: &mut HashMap<
        std::path::PathBuf,
        Option<hk_media::VideoSource>,
    >,
) -> Result<Rgba, String> {
    match source {
        MediaSource::Sample(i) => {
            let (w, h) = if dim == THUMB_DIM {
                (THUMB_DIM, THUMB_DIM * 9 / 16)
            } else {
                (1280, 720)
            };
            Ok(hk_media::sample_frame(*i, w, h))
        }
        MediaSource::File(path) => match kind {
            MediaKind::Image => hk_media::load_image(path, dim).map_err(|e| e.to_string()),
            MediaKind::Video => {
                #[cfg(target_os = "macos")]
                {
                    let src = sources
                        .entry(path.clone())
                        .or_insert_with(|| hk_media::VideoSource::open(path).ok());
                    match src {
                        Some(s) => s.frame_at(secs, dim).map_err(|e| e.to_string()),
                        None => Err(format!(
                            "Video konnte nicht geöffnet werden: {}",
                            path.display()
                        )),
                    }
                }
                #[cfg(not(target_os = "macos"))]
                {
                    hk_media::video_frame(path, secs, dim).map_err(|e| e.to_string())
                }
            }
            MediaKind::Audio => Err("Audio hat kein Bild".into()),
        },
    }
}
