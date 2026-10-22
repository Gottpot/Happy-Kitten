use crate::overlay::OverlaySpec;
use crate::subtitles::Cue;
use crate::text::{TextFx, TextSpec};
use crate::time::{Fps, Tick};
use crate::transition::{Transition, TransitionKind};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Id = u64;

/// Shortest clip the editor will produce (about three frames at 30 fps).
const MIN_CLIP: Tick = Tick(70_560_000);

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum EditError {
    #[error("Clip nicht gefunden")]
    NoSuchClip,
    #[error("Medium nicht gefunden")]
    NoSuchMedia,
    #[error("Spur nicht gefunden")]
    NoSuchTrack,
    #[error("Das Medium passt nicht auf diese Spur")]
    WrongTrackKind,
    #[error("Hier ist kein Platz")]
    NoRoom,
    #[error("Der Schnitt liegt außerhalb des Clips")]
    OutsideClip,
    #[error("Eine Blende braucht zwei Bild-Clips direkt hintereinander")]
    NoNeighbour,
    #[error("Das Medium wird noch in einem Clip benutzt")]
    MediaInUse,
    #[error("Es gibt schon genug Spuren dieser Art")]
    TooManyTracks,
    #[error("Die Spur ist nicht leer oder die letzte ihrer Art")]
    TrackNotRemovable,
}

/// Most tracks of one kind (video or audio).
pub const MAX_TRACKS_PER_KIND: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Video,
    Image,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaSource {
    File(PathBuf),
    /// Built-in demo scene (see `hk-media`), always available offline.
    Sample(u8),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Id,
    pub name: String,
    pub source: MediaSource,
    pub kind: MediaKind,
    pub duration: Tick,
    pub size: (u32, u32),
    /// The picture has see-through parts (a PNG logo): it is laid over the video as a graphic.
    #[serde(default)]
    pub alpha: bool,
}

impl MediaItem {
    /// Stills have no natural end; the timeline may stretch them freely.
    pub fn is_unbounded(&self) -> bool {
        self.kind == MediaKind::Image || matches!(self.source, MediaSource::Sample(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub id: Id,
    pub media: Id,
    pub start: Tick,
    pub duration: Tick,
    pub source_in: Tick,
    /// Index into the UI's pastel clip palette.
    pub color: u8,
    pub transition_in: Option<Transition>,
    /// Set for text clips (they have no media; `media` is 0).
    #[serde(default)]
    pub text: Option<TextSpec>,
    /// Set for graphic clips: an image laid over the picture instead of replacing it.
    #[serde(default)]
    pub overlay: Option<OverlaySpec>,
}

impl Clip {
    pub fn is_text(&self) -> bool {
        self.text.is_some()
    }

    pub fn is_overlay(&self) -> bool {
        self.overlay.is_some()
    }

    /// Text and graphics are drawn on top of the picture and are never the picture itself.
    pub fn is_layer(&self) -> bool {
        self.text.is_some() || self.overlay.is_some()
    }

    pub fn end(&self) -> Tick {
        self.start + self.duration
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: Id,
    pub name: String,
    pub kind: TrackKind,
    pub clips: Vec<Clip>,
    pub muted: bool,
}

/// How a picture that does not match the canvas shape is placed on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FitMode {
    /// Whole picture visible, empty bars where it does not fill.
    #[default]
    Contain,
    /// Picture fills the canvas, the overhang is cropped.
    Cover,
    /// Whole picture, with a blurred enlarged copy behind it instead of bars.
    Blur,
}

impl FitMode {
    pub const ALL: [FitMode; 3] = [FitMode::Contain, FitMode::Cover, FitMode::Blur];

    pub fn shader_id(self) -> u32 {
        match self {
            FitMode::Contain => 0,
            FitMode::Cover => 1,
            FitMode::Blur => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FitMode::Contain => "Einpassen",
            FitMode::Cover => "Füllen",
            FitMode::Blur => "Unscharf füllen",
        }
    }
}

/// Canvas shapes offered in the UI: (label, width : height).
pub const CANVAS_SHAPES: [(&str, u32, u32); 4] = [
    ("16:9", 16, 9),
    ("9:16", 9, 16),
    ("1:1", 1, 1),
    ("4:5", 4, 5),
];

/// Preview size for a canvas shape: long side 1280, both sides even.
pub fn preview_size(w: u32, h: u32) -> (u32, u32) {
    let (fw, fh) = (w.max(1) as f64, h.max(1) as f64);
    let k = 1280.0 / fw.max(fh);
    let even = |v: f64| ((v * k).round() as u32).max(2) & !1;
    (even(fw), even(fh))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub fps: Fps,
    pub size: (u32, u32),
    #[serde(default)]
    pub fit: FitMode,
    pub media: Vec<MediaItem>,
    pub tracks: Vec<Track>,
    next_id: Id,
    /// While a drag is running, blends whose neighbour is momentarily away are kept.
    #[serde(skip)]
    hold: bool,
}

/// Where to read one frame from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceRef {
    pub media: Id,
    pub time: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionSpan {
    pub kind: TransitionKind,
    pub from: SourceRef,
    pub to: SourceRef,
    /// 0.0 = only `from`, 1.0 = only `to`.
    pub progress: f32,
}

/// What the preview has to show at one point in time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VideoFrame {
    Black,
    Single(SourceRef),
    Blend(TransitionSpan),
}

/// A text clip as it must be drawn right now, with its fade-in/out state.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLayer {
    pub clip: Id,
    pub spec: TextSpec,
    pub fx: TextFx,
    /// 0 = invisible, 1 = fully shown; the effect decides what in between looks like.
    pub progress: f32,
}

/// A graphic clip as it must be drawn right now.
#[derive(Clone, Debug, PartialEq)]
pub struct ImageLayer {
    pub clip: Id,
    pub media: Id,
    pub spec: OverlaySpec,
    pub fx: TextFx,
    pub progress: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Layer {
    Text(TextLayer),
    Image(ImageLayer),
}

/// Everything to draw at one point in time: the picture, then text and graphics on top of it
/// (lower tracks first).
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub base: VideoFrame,
    pub layers: Vec<Layer>,
}

impl Scene {
    pub fn texts(&self) -> impl Iterator<Item = &TextLayer> {
        self.layers.iter().filter_map(|l| {
            if let Layer::Text(t) = l {
                Some(t)
            } else {
                None
            }
        })
    }

    pub fn images(&self) -> impl Iterator<Item = &ImageLayer> {
        self.layers.iter().filter_map(|l| {
            if let Layer::Image(i) = l {
                Some(i)
            } else {
                None
            }
        })
    }
}

impl Default for Project {
    fn default() -> Self {
        let mut p = Project {
            name: "Mein Kätzchen-Film".into(),
            fps: Fps::FPS_30,
            size: (1280, 720),
            fit: FitMode::Contain,
            media: Vec::new(),
            tracks: Vec::new(),
            next_id: 1,
            hold: false,
        };
        p.add_track("Video 1", TrackKind::Video);
        p.add_track("Audio 1", TrackKind::Audio);
        p
    }
}

impl Project {
    fn alloc_id(&mut self) -> Id {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    pub fn add_track(&mut self, name: &str, kind: TrackKind) -> Id {
        let id = self.alloc_id();
        self.tracks.push(Track {
            id,
            name: name.into(),
            kind,
            clips: Vec::new(),
            muted: false,
        });
        id
    }

    /// Add the next numbered track ("Video 3", "Audio 2") on top of the others of its kind.
    pub fn add_track_of(&mut self, kind: TrackKind) -> Result<Id, EditError> {
        let count = self.tracks.iter().filter(|t| t.kind == kind).count();
        if count >= MAX_TRACKS_PER_KIND {
            return Err(EditError::TooManyTracks);
        }
        let base = if kind == TrackKind::Video {
            "Video"
        } else {
            "Audio"
        };
        let n = (1..)
            .find(|n| !self.tracks.iter().any(|t| t.name == format!("{base} {n}")))
            .unwrap_or(count + 1);
        Ok(self.add_track(&format!("{base} {n}"), kind))
    }

    pub fn rename_track(&mut self, id: Id, name: &str) {
        if let Some(t) = self.tracks.iter_mut().find(|t| t.id == id) {
            t.name = name.chars().take(24).collect();
        }
    }

    /// Only empty tracks go, and one of each kind always stays.
    pub fn remove_track(&mut self, id: Id) -> Result<(), EditError> {
        let idx = self.track_index(id)?;
        let kind = self.tracks[idx].kind;
        if !self.tracks[idx].clips.is_empty()
            || self.tracks.iter().filter(|t| t.kind == kind).count() <= 1
        {
            return Err(EditError::TrackNotRemovable);
        }
        self.tracks.remove(idx);
        Ok(())
    }

    /// Reuse an existing media entry with the same source, else add one.
    pub fn ensure_media(
        &mut self,
        name: String,
        source: MediaSource,
        kind: MediaKind,
        duration: Tick,
        size: (u32, u32),
    ) -> Id {
        match self.media.iter().find(|m| m.source == source) {
            Some(m) => m.id,
            None => self.add_media(name, source, kind, duration, size),
        }
    }

    pub fn add_media(
        &mut self,
        name: String,
        source: MediaSource,
        kind: MediaKind,
        duration: Tick,
        size: (u32, u32),
    ) -> Id {
        let id = self.alloc_id();
        self.media.push(MediaItem {
            id,
            name,
            source,
            kind,
            duration,
            size,
            alpha: false,
        });
        id
    }

    /// Media entries that at least one clip uses.
    pub fn used_media(&self) -> std::collections::HashSet<Id> {
        self.tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| !c.is_text())
            .map(|c| c.media)
            .collect()
    }

    /// Set the canvas shape (the pictures of all clips stay as they are).
    pub fn set_canvas(&mut self, size: (u32, u32)) {
        self.size = (size.0.max(2), size.1.max(2));
    }

    /// Move several clips together. Every clip goes to `origin + delta`, with `delta` clamped
    /// so that nothing crosses time zero or runs into a clip that is not part of the move.
    /// Stateless: call it again with the same `origins` and a new `delta` while dragging.
    pub fn shift_clips(&mut self, origins: &[(Id, Tick)], delta: Tick) -> Tick {
        let moving: std::collections::HashSet<Id> = origins.iter().map(|(id, _)| *id).collect();
        let (mut lo, mut hi) = (Tick(i64::MIN / 4), Tick(i64::MAX / 4));
        for (id, origin) in origins {
            let Some((track, clip)) = self.clip(*id) else {
                continue;
            };
            let end = *origin + clip.duration;
            lo = lo.max(Tick::ZERO - *origin);
            for other in track.clips.iter().filter(|o| !moving.contains(&o.id)) {
                if other.end() <= *origin {
                    lo = lo.max(other.end() - *origin);
                } else if other.start >= end {
                    hi = hi.min(other.start - end);
                } else {
                    // already overlapping a fixed clip: do not move at all
                    lo = lo.max(Tick::ZERO);
                    hi = hi.min(Tick::ZERO);
                }
            }
        }
        let applied = if lo <= hi {
            delta.clamp(lo, hi)
        } else {
            Tick::ZERO
        };
        for (id, origin) in origins {
            if let Ok((ti, ci)) = self.locate(*id) {
                self.tracks[ti].clips[ci].start = *origin + applied;
            }
        }
        for t in &mut self.tracks {
            t.clips.sort_by_key(|c| c.start);
        }
        self.prune_transitions();
        applied
    }

    /// Put `transition` on every cut between two selected neighbouring clips. With one clip
    /// selected, its own left cut (or its right cut when it has no left neighbour) is used.
    /// Returns how many blends were set.
    pub fn apply_transition_to_cuts(&mut self, selected: &[Id], transition: Transition) -> usize {
        let set: std::collections::HashSet<Id> = selected.iter().copied().collect();
        let mut targets: Vec<Id> = Vec::new();
        for id in selected {
            if let Some(left) = self.left_neighbour(*id)
                && set.contains(&left.id)
            {
                targets.push(*id);
            }
        }
        if targets.is_empty() && selected.len() == 1 {
            let id = selected[0];
            if self.left_neighbour(id).is_some() {
                targets.push(id);
            } else if let Some((t, c)) = self.clip(id)
                && let Some(next) = t.clips.iter().find(|o| o.start == c.end() && o.id != id)
            {
                targets.push(next.id);
            }
        }
        targets
            .into_iter()
            .filter(|id| self.set_transition(*id, Some(transition)).is_ok())
            .count()
    }

    /// Take a media entry out of the project pool; refused while a clip still uses it.
    pub fn remove_media(&mut self, id: Id) -> Result<(), EditError> {
        if self
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .any(|c| c.media == id && !c.is_text())
        {
            return Err(EditError::MediaInUse);
        }
        self.media.retain(|m| m.id != id);
        Ok(())
    }

    pub fn media(&self, id: Id) -> Option<&MediaItem> {
        self.media.iter().find(|m| m.id == id)
    }

    pub fn track(&self, id: Id) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    pub fn clip(&self, id: Id) -> Option<(&Track, &Clip)> {
        self.tracks
            .iter()
            .find_map(|t| t.clips.iter().find(|c| c.id == id).map(|c| (t, c)))
    }

    fn locate(&self, clip: Id) -> Result<(usize, usize), EditError> {
        self.tracks
            .iter()
            .enumerate()
            .find_map(|(ti, t)| t.clips.iter().position(|c| c.id == clip).map(|ci| (ti, ci)))
            .ok_or(EditError::NoSuchClip)
    }

    fn track_index(&self, id: Id) -> Result<usize, EditError> {
        self.tracks
            .iter()
            .position(|t| t.id == id)
            .ok_or(EditError::NoSuchTrack)
    }

    pub fn duration(&self) -> Tick {
        self.tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .map(Clip::end)
            .fold(Tick::ZERO, Tick::max)
    }

    pub fn min_clip_duration(&self) -> Tick {
        MIN_CLIP
    }

    // ----- placement ---------------------------------------------------

    /// Closest start to `desired` where a clip of `duration` fits on the track
    /// without overlapping any clip except `ignore`.
    pub fn free_start(
        &self,
        track: Id,
        ignore: Option<Id>,
        desired: Tick,
        duration: Tick,
    ) -> Option<Tick> {
        let track = self.track(track)?;
        let mut others: Vec<(Tick, Tick)> = track
            .clips
            .iter()
            .filter(|c| Some(c.id) != ignore)
            .map(|c| (c.start, c.end()))
            .collect();
        others.sort();
        let desired = desired.max(Tick::ZERO);
        // Gaps between neighbours; the last gap is open-ended.
        let mut best: Option<Tick> = None;
        let mut consider = |gap_start: Tick, gap_end: Option<Tick>| {
            let latest = match gap_end {
                Some(end) if end - gap_start < duration => return,
                Some(end) => end - duration,
                None => Tick(i64::MAX / 4),
            };
            let pos = desired.clamp(gap_start, latest);
            if best.is_none_or(|b| (pos - desired).0.abs() < (b - desired).0.abs()) {
                best = Some(pos);
            }
        };
        let mut cursor = Tick::ZERO;
        for (s, e) in &others {
            consider(cursor, Some(*s));
            cursor = cursor.max(*e);
        }
        consider(cursor, None);
        best
    }

    pub fn add_clip(&mut self, track: Id, media: Id, at: Tick) -> Result<Id, EditError> {
        let item = self.media(media).ok_or(EditError::NoSuchMedia)?;
        let kind = self.track(track).ok_or(EditError::NoSuchTrack)?.kind;
        let fits = matches!(
            (item.kind, kind),
            (MediaKind::Audio, TrackKind::Audio)
                | (MediaKind::Video | MediaKind::Image, TrackKind::Video)
        );
        if !fits {
            return Err(EditError::WrongTrackKind);
        }
        let duration = if item.is_unbounded() {
            Tick::from_secs(5.0)
        } else {
            item.duration.max(MIN_CLIP)
        };
        let start = self
            .free_start(track, None, at, duration)
            .ok_or(EditError::NoRoom)?;
        let color = (media % 6) as u8;
        let overlay = (item.kind == MediaKind::Image && item.alpha).then(OverlaySpec::default);
        let id = self.alloc_id();
        let ti = self.track_index(track)?;
        let clips = &mut self.tracks[ti].clips;
        clips.push(Clip {
            id,
            media,
            start,
            duration,
            source_in: Tick::ZERO,
            color,
            transition_in: None,
            text: None,
            overlay,
        });
        clips.sort_by_key(|c| c.start);
        self.prune_transitions();
        Ok(id)
    }

    /// Flag a picture as having transparency (graphics are laid over the video by default).
    pub fn set_media_alpha(&mut self, media: Id, alpha: bool) {
        if let Some(m) = self.media.iter_mut().find(|m| m.id == media) {
            m.alpha = alpha;
        }
    }

    /// Turn an image clip into a graphic over the picture (`Some`) or back into a full picture.
    pub fn set_overlay(&mut self, clip: Id, overlay: Option<OverlaySpec>) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        let c = &self.tracks[ti].clips[ci];
        let is_image = !c.is_text()
            && self
                .media(c.media)
                .is_some_and(|m| m.kind == MediaKind::Image);
        if overlay.is_some() && !is_image {
            return Err(EditError::WrongTrackKind);
        }
        self.tracks[ti].clips[ci].overlay = overlay;
        self.prune_transitions();
        Ok(())
    }

    /// Put an image on a video track as a graphic over the picture, at the nearest free spot.
    pub fn add_overlay_clip(&mut self, track: Id, media: Id, at: Tick) -> Result<Id, EditError> {
        let id = self.add_clip(track, media, at)?;
        self.set_overlay(id, Some(OverlaySpec::default()))?;
        Ok(id)
    }

    /// Give every listed text clip the look of `style` (their words stay). With `with_position`
    /// false each clip also keeps its own place on the picture.
    pub fn apply_style_to_texts(
        &mut self,
        ids: &[Id],
        style: &TextSpec,
        with_position: bool,
    ) -> usize {
        let mut n = 0;
        for t in self.tracks.iter_mut() {
            for c in t.clips.iter_mut().filter(|c| ids.contains(&c.id)) {
                if let Some(spec) = &mut c.text {
                    let pos = spec.pos;
                    spec.copy_style_from(style);
                    if !with_position {
                        spec.pos = pos;
                    }
                    n += 1;
                }
            }
        }
        n
    }

    pub fn overlay_of_mut(&mut self, clip: Id) -> Option<&mut OverlaySpec> {
        self.tracks
            .iter_mut()
            .flat_map(|t| t.clips.iter_mut())
            .find(|c| c.id == clip)
            .and_then(|c| c.overlay.as_mut())
    }

    /// Put a text clip on a video track at the nearest free spot.
    pub fn add_text_clip(
        &mut self,
        track: Id,
        at: Tick,
        duration: Tick,
        spec: TextSpec,
    ) -> Result<Id, EditError> {
        let ti = self.track_index(track)?;
        if self.tracks[ti].kind != TrackKind::Video {
            return Err(EditError::WrongTrackKind);
        }
        let duration = duration.max(MIN_CLIP);
        let start = self
            .free_start(track, None, at, duration)
            .ok_or(EditError::NoRoom)?;
        let id = self.alloc_id();
        let clips = &mut self.tracks[ti].clips;
        clips.push(Clip {
            id,
            media: 0,
            start,
            duration,
            source_in: Tick::ZERO,
            color: 4,
            transition_in: None,
            text: Some(spec),
            overlay: None,
        });
        clips.sort_by_key(|c| c.start);
        Ok(id)
    }

    /// One text clip per cue on `track`. Cues that would overlap the previous one are cut short or
    /// skipped so the track stays valid. Returns the new clip ids.
    pub fn import_cues(
        &mut self,
        track: Id,
        cues: &[Cue],
        style: &TextSpec,
        offset: Tick,
    ) -> Vec<Id> {
        let mut cues: Vec<&Cue> = cues.iter().collect();
        cues.sort_by_key(|c| c.start);
        let mut ids = Vec::new();
        for (i, cue) in cues.iter().enumerate() {
            let start = cue.start + offset;
            let mut end = cue.end + offset;
            if let Some(next) = cues.get(i + 1) {
                end = end.min(next.start + offset);
            }
            if end - start < MIN_CLIP {
                continue;
            }
            let mut spec = style.clone();
            spec.text = cue.text.clone();
            let free = self.free_start(track, None, start, end - start);
            if free != Some(start) {
                continue;
            }
            if let Ok(id) = self.add_text_clip(track, start, end - start, spec) {
                ids.push(id);
            }
        }
        ids
    }

    /// Copy `style` onto every text clip of the track (words stay).
    pub fn restyle_track_texts(&mut self, track: Id, style: &TextSpec) {
        if let Some(t) = self.tracks.iter_mut().find(|t| t.id == track) {
            for c in &mut t.clips {
                if let Some(spec) = &mut c.text {
                    spec.copy_style_from(style);
                }
            }
        }
    }

    pub fn text_of_mut(&mut self, clip: Id) -> Option<&mut TextSpec> {
        self.tracks
            .iter_mut()
            .flat_map(|t| t.clips.iter_mut())
            .find(|c| c.id == clip)
            .and_then(|c| c.text.as_mut())
    }

    /// Move a clip, possibly to another track of the same kind. The result is
    /// the nearest free position, so clips can never overlap.
    pub fn move_clip(
        &mut self,
        clip: Id,
        to_track: Id,
        desired_start: Tick,
    ) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        let dest = self.track_index(to_track)?;
        if self.tracks[ti].kind != self.tracks[dest].kind {
            return Err(EditError::WrongTrackKind);
        }
        let duration = self.tracks[ti].clips[ci].duration;
        let start = self
            .free_start(to_track, Some(clip), desired_start, duration)
            .ok_or(EditError::NoRoom)?;
        let mut c = self.tracks[ti].clips.remove(ci);
        c.start = start;
        self.tracks[dest].clips.push(c);
        self.tracks[dest].clips.sort_by_key(|c| c.start);
        self.prune_transitions();
        Ok(())
    }

    pub fn delete_clip(&mut self, clip: Id, ripple: bool) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        let removed = self.tracks[ti].clips.remove(ci);
        if ripple {
            for c in self.tracks[ti]
                .clips
                .iter_mut()
                .filter(|c| c.start >= removed.end())
            {
                c.start -= removed.duration;
            }
        }
        self.prune_transitions();
        Ok(())
    }

    // ----- split & trim ------------------------------------------------

    /// Cut a clip in two at timeline position `at`. Returns the id of the right half.
    pub fn split_clip(&mut self, clip: Id, at: Tick) -> Result<Id, EditError> {
        let (ti, ci) = self.locate(clip)?;
        let c = self.tracks[ti].clips[ci].clone();
        if at < c.start + MIN_CLIP || at > c.end() - MIN_CLIP {
            return Err(EditError::OutsideClip);
        }
        let left_len = at - c.start;
        let id = self.alloc_id();
        let right = Clip {
            id,
            media: c.media,
            start: at,
            duration: c.duration - left_len,
            source_in: c.source_in + left_len,
            color: c.color,
            transition_in: None,
            text: c.text.clone().map(|mut t| {
                t.fx_in = TextFx::None;
                t
            }),
            overlay: c.overlay.clone().map(|mut o| {
                o.fx_in = TextFx::None;
                o
            }),
        };
        if let Some(t) = &mut self.tracks[ti].clips[ci].text {
            t.fx_out = TextFx::None;
        }
        if let Some(o) = &mut self.tracks[ti].clips[ci].overlay {
            o.fx_out = TextFx::None;
        }
        self.tracks[ti].clips[ci].duration = left_len;
        self.tracks[ti].clips.insert(ci + 1, right);
        Ok(id)
    }

    /// Split every clip under the playhead (all unlocked tracks).
    pub fn split_at(&mut self, at: Tick) -> usize {
        let targets: Vec<Id> = self
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| at > c.start && at < c.end())
            .map(|c| c.id)
            .collect();
        targets
            .into_iter()
            .filter(|id| self.split_clip(*id, at).is_ok())
            .count()
    }

    /// Drag the left edge of a clip to `new_start`; the end stays put.
    pub fn trim_start(&mut self, clip: Id, new_start: Tick) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        let c = &self.tracks[ti].clips[ci];
        let prev_end = ci
            .checked_sub(1)
            .and_then(|p| self.tracks[ti].clips.get(p))
            .map_or(Tick::ZERO, Clip::end);
        // can't reveal media before its first frame
        let lo = prev_end.max(c.start - c.source_in);
        let hi = c.end() - MIN_CLIP;
        let new_start = new_start.clamp(lo, hi);
        let delta = new_start - c.start;
        let c = &mut self.tracks[ti].clips[ci];
        c.start = new_start;
        c.duration -= delta;
        c.source_in += delta;
        self.prune_transitions();
        Ok(())
    }

    /// Drag the right edge of a clip to `new_end`; the start stays put.
    pub fn trim_end(&mut self, clip: Id, new_end: Tick) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        let c = &self.tracks[ti].clips[ci];
        let next_start = self.tracks[ti].clips.get(ci + 1).map(|n| n.start);
        let unbounded = c.is_layer() || self.media(c.media).is_some_and(MediaItem::is_unbounded);
        let mut hi = next_start.unwrap_or(Tick(i64::MAX / 4));
        if !unbounded {
            let media_len = self.media(c.media).map_or(Tick::ZERO, |m| m.duration);
            hi = hi.min(c.start + (media_len - c.source_in));
        }
        let new_end = new_end.clamp(c.start + MIN_CLIP, hi);
        self.tracks[ti].clips[ci].duration = new_end - self.tracks[ti].clips[ci].start;
        self.prune_transitions();
        Ok(())
    }

    // ----- transitions -------------------------------------------------

    /// Clip that ends exactly where `clip` begins on the same track.
    pub fn left_neighbour(&self, clip: Id) -> Option<&Clip> {
        let (t, c) = self.clip(clip)?;
        if c.is_layer() {
            return None;
        }
        t.clips
            .iter()
            .find(|o| o.end() == c.start && o.id != c.id && !o.is_layer())
    }

    pub fn max_transition(&self, clip: Id) -> Tick {
        match (self.clip(clip), self.left_neighbour(clip)) {
            (Some((_, c)), Some(l)) => {
                Tick(c.duration.0.min(l.duration.0) * 9 / 10).max(Tick::ZERO)
            }
            _ => Tick::ZERO,
        }
    }

    pub fn set_transition(
        &mut self,
        clip: Id,
        transition: Option<Transition>,
    ) -> Result<(), EditError> {
        let (ti, ci) = self.locate(clip)?;
        if let Some(mut t) = transition {
            if self.left_neighbour(clip).is_none() {
                return Err(EditError::NoNeighbour);
            }
            t.duration = t.duration.min(self.max_transition(clip)).max(MIN_CLIP);
            self.tracks[ti].clips[ci].transition_in = Some(t);
        } else {
            self.tracks[ti].clips[ci].transition_in = None;
        }
        Ok(())
    }

    /// Cut points a transition can be dropped on: (right clip, cut time).
    pub fn cuts(&self, track: Id) -> Vec<(Id, Tick)> {
        let Some(t) = self.track(track) else {
            return Vec::new();
        };
        t.clips
            .iter()
            .filter(|c| self.left_neighbour(c.id).is_some())
            .map(|c| (c.id, c.start))
            .collect()
    }

    /// During a drag keep blends even when their neighbour is briefly away; turning it off again
    /// removes the ones that really lost their neighbour.
    pub fn hold_transitions(&mut self, on: bool) {
        self.hold = on;
        if !on {
            self.prune_transitions();
        }
    }

    /// Remove or clamp transitions whose neighbour moved away or got too short.
    fn prune_transitions(&mut self) {
        let ids: Vec<Id> = self
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .filter(|c| c.transition_in.is_some())
            .map(|c| c.id)
            .collect();
        for id in ids {
            let max = self.max_transition(id);
            let has_left = self.left_neighbour(id).is_some();
            if let Ok((ti, ci)) = self.locate(id) {
                let c = &mut self.tracks[ti].clips[ci];
                match (&mut c.transition_in, has_left && max >= MIN_CLIP) {
                    (Some(_), false) if !self.hold => c.transition_in = None,
                    (Some(t), true) => t.duration = t.duration.min(max),
                    _ => {}
                }
            }
        }
    }

    // ----- evaluation --------------------------------------------------

    /// The picture and all text on top of it at time `t`. Text of every unmuted video track is
    /// drawn, lower tracks first.
    pub fn scene_at(&self, t: Tick) -> Scene {
        let mut layers = Vec::new();
        for track in self
            .tracks
            .iter()
            .filter(|tr| tr.kind == TrackKind::Video && !tr.muted)
        {
            for c in &track.clips {
                if t < c.start || t >= c.end() {
                    continue;
                }
                let ramp = |dur: Tick, dist: Tick| {
                    if dur.0 > 0 {
                        (dist.0 as f64 / dur.0 as f64).clamp(0.0, 1.0) as f32
                    } else {
                        1.0
                    }
                };
                // which fade (in at the start, out at the end) is running and how far it is
                let fade = |fx_in: TextFx, d_in: Tick, fx_out: TextFx, d_out: Tick| {
                    let in_p = if fx_in == TextFx::None {
                        1.0
                    } else {
                        ramp(d_in, t - c.start)
                    };
                    let out_p = if fx_out == TextFx::None {
                        1.0
                    } else {
                        ramp(d_out, c.end() - t)
                    };
                    if in_p < 1.0 {
                        (fx_in, in_p)
                    } else if out_p < 1.0 {
                        (fx_out, out_p)
                    } else {
                        (TextFx::None, 1.0)
                    }
                };
                if let Some(spec) = &c.text {
                    let (fx, progress) =
                        fade(spec.fx_in, spec.fx_in_dur, spec.fx_out, spec.fx_out_dur);
                    layers.push(Layer::Text(TextLayer {
                        clip: c.id,
                        spec: spec.clone(),
                        fx,
                        progress,
                    }));
                } else if let Some(spec) = &c.overlay {
                    let (fx, progress) =
                        fade(spec.fx_in, spec.fx_in_dur, spec.fx_out, spec.fx_out_dur);
                    layers.push(Layer::Image(ImageLayer {
                        clip: c.id,
                        media: c.media,
                        spec: spec.clone(),
                        fx,
                        progress,
                    }));
                }
            }
        }
        Scene {
            base: self.video_frame_at(t),
            layers,
        }
    }

    /// What the preview must show at time `t`: the top-most video track wins.
    pub fn video_frame_at(&self, t: Tick) -> VideoFrame {
        for track in self
            .tracks
            .iter()
            .rev()
            .filter(|tr| tr.kind == TrackKind::Video && !tr.muted)
        {
            if let Some(f) = Self::frame_on_track(track, t) {
                return f;
            }
        }
        VideoFrame::Black
    }

    fn source_time(c: &Clip, t: Tick) -> Tick {
        (c.source_in + (t - c.start)).max(Tick::ZERO)
    }

    fn frame_on_track(track: &Track, t: Tick) -> Option<VideoFrame> {
        for (i, c) in track.clips.iter().enumerate() {
            if c.is_layer() {
                continue;
            }
            let Some(tr) = c.transition_in else { continue };
            let Some(left) = i
                .checked_sub(1)
                .and_then(|p| track.clips.get(p))
                .filter(|l| l.end() == c.start)
            else {
                continue;
            };
            let from_t = c.start - Tick(tr.duration.0 / 2);
            let to_t = from_t + tr.duration;
            if t >= from_t && t < to_t && tr.duration.0 > 0 {
                let progress = ((t - from_t).0 as f64 / tr.duration.0 as f64) as f32;
                return Some(VideoFrame::Blend(TransitionSpan {
                    kind: tr.kind,
                    from: SourceRef {
                        media: left.media,
                        time: Self::source_time(left, t),
                    },
                    to: SourceRef {
                        media: c.media,
                        time: Self::source_time(c, t),
                    },
                    progress: progress.clamp(0.0, 1.0),
                }));
            }
        }
        track
            .clips
            .iter()
            .find(|c| !c.is_layer() && t >= c.start && t < c.end())
            .map(|c| {
                VideoFrame::Single(SourceRef {
                    media: c.media,
                    time: Self::source_time(c, t),
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(x: f64) -> Tick {
        Tick::from_secs(x)
    }

    fn sample() -> (Project, Id, Id) {
        let mut p = Project::default();
        let m = p.add_media(
            "a".into(),
            MediaSource::Sample(0),
            MediaKind::Image,
            s(5.0),
            (1280, 720),
        );
        let v = p.tracks[0].id;
        (p, v, m)
    }

    #[test]
    fn clips_never_overlap() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(1.0)).unwrap();
        let (_, ca) = p.clip(a).unwrap();
        let (_, cb) = p.clip(b).unwrap();
        assert_eq!(cb.start, ca.end());
    }

    #[test]
    fn split_keeps_total_length_and_source_offsets() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let r = p.split_clip(a, s(2.0)).unwrap();
        let (_, l) = p.clip(a).unwrap();
        let (_, r) = p.clip(r).unwrap();
        assert_eq!(l.duration + r.duration, s(5.0));
        assert_eq!(r.source_in, s(2.0));
        assert_eq!(r.start, l.end());
    }

    #[test]
    fn split_near_edge_is_refused() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        assert_eq!(p.split_clip(a, Tick(1)), Err(EditError::OutsideClip));
    }

    #[test]
    fn trim_start_respects_neighbour_and_source() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.trim_start(b, s(1.0)).unwrap();
        assert_eq!(
            p.clip(b).unwrap().1.start,
            s(5.0),
            "cannot grow into the left neighbour"
        );
        p.trim_start(b, s(7.0)).unwrap();
        let (_, cb) = p.clip(b).unwrap();
        assert_eq!(cb.start, s(7.0));
        assert_eq!(cb.source_in, s(2.0));
        assert_eq!(cb.end(), s(10.0));
        let _ = a;
    }

    #[test]
    fn video_trim_end_cannot_exceed_media() {
        let mut p = Project::default();
        let m = p.add_media(
            "v".into(),
            MediaSource::File("x.mp4".into()),
            MediaKind::Video,
            s(4.0),
            (1920, 1080),
        );
        let v = p.tracks[0].id;
        let c = p.add_clip(v, m, s(0.0)).unwrap();
        p.trim_end(c, s(60.0)).unwrap();
        assert_eq!(p.clip(c).unwrap().1.duration, s(4.0));
    }

    #[test]
    fn move_snaps_into_nearest_gap() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.move_clip(a, v, s(6.0)).unwrap();
        let (_, ca) = p.clip(a).unwrap();
        let (_, cb) = p.clip(b).unwrap();
        assert!(ca.start >= cb.end() || ca.end() <= cb.start);
    }

    #[test]
    fn transition_needs_neighbour_and_blends_around_cut() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        assert_eq!(
            p.set_transition(a, Some(Transition::new(TransitionKind::ClawWipe))),
            Err(EditError::NoNeighbour)
        );
        p.set_transition(
            b,
            Some(Transition {
                kind: TransitionKind::ClawWipe,
                duration: s(1.0),
            }),
        )
        .unwrap();
        assert!(matches!(p.video_frame_at(s(4.0)), VideoFrame::Single(_)));
        match p.video_frame_at(s(5.0)) {
            VideoFrame::Blend(span) => assert!((span.progress - 0.5).abs() < 1e-3),
            other => panic!("{other:?}"),
        }
        assert!(matches!(p.video_frame_at(s(5.6)), VideoFrame::Single(_)));
    }

    #[test]
    fn moving_a_clip_away_removes_its_transition() {
        let (mut p, v, m) = sample();
        p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.set_transition(b, Some(Transition::new(TransitionKind::PawSpin)))
            .unwrap();
        p.move_clip(b, v, s(20.0)).unwrap();
        assert!(p.clip(b).unwrap().1.transition_in.is_none());
    }

    #[test]
    fn nudging_a_clip_that_snaps_back_keeps_its_transition() {
        let (mut p, v, m) = sample();
        p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.set_transition(b, Some(Transition::new(TransitionKind::PawSpin)))
            .unwrap();
        p.move_clip(b, v, s(4.0)).unwrap(); // overlaps the left clip, so it lands where it was
        assert_eq!(p.clip(b).unwrap().1.start, s(5.0));
        assert!(p.clip(b).unwrap().1.transition_in.is_some());
    }

    #[test]
    fn ripple_delete_closes_gap() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.delete_clip(a, true).unwrap();
        assert_eq!(p.clip(b).unwrap().1.start, Tick::ZERO);
    }

    #[test]
    fn audio_does_not_go_on_video_track() {
        let mut p = Project::default();
        let m = p.add_media(
            "song".into(),
            MediaSource::File("a.mp3".into()),
            MediaKind::Audio,
            s(10.0),
            (0, 0),
        );
        let v = p.tracks[0].id;
        assert_eq!(p.add_clip(v, m, Tick::ZERO), Err(EditError::WrongTrackKind));
    }

    #[test]
    fn text_clips_do_not_block_picture_or_transitions() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        let v2 = p.add_track_of(TrackKind::Video).unwrap();
        let t = p
            .add_text_clip(v2, s(1.0), s(2.0), TextSpec::default())
            .unwrap();
        match p.video_frame_at(s(1.5)) {
            VideoFrame::Single(src) => assert_eq!(
                src.media, m,
                "the picture comes from the media clip, not the text"
            ),
            other => panic!("{other:?}"),
        }
        assert_eq!(p.scene_at(s(1.5)).texts().count(), 1);
        // a text clip alone gives no picture
        let (mut q, qv, _) = sample();
        q.add_text_clip(qv, s(0.0), s(2.0), TextSpec::default())
            .unwrap();
        assert_eq!(q.video_frame_at(s(1.0)), VideoFrame::Black);
        assert!(p.scene_at(s(3.5)).texts().next().is_none());
        // text on the same track as media is fine, but cannot take a blend
        assert_eq!(
            p.set_transition(t, Some(Transition::new(TransitionKind::Push))),
            Err(EditError::NoNeighbour)
        );
        p.set_transition(b, Some(Transition::new(TransitionKind::Push)))
            .unwrap();
        let _ = a;
    }

    #[test]
    fn text_fades_in_and_out() {
        let (mut p, v, _) = sample();
        let spec = TextSpec {
            fx_in: TextFx::Pixelate,
            fx_in_dur: s(1.0),
            fx_out: TextFx::Swirl,
            fx_out_dur: s(1.0),
            ..Default::default()
        };
        p.add_text_clip(v, s(0.0), s(5.0), spec).unwrap();
        let sc = p.scene_at(s(0.25));
        let early = sc.texts().next().unwrap();
        assert_eq!(early.fx, TextFx::Pixelate);
        assert!((early.progress - 0.25).abs() < 1e-3);
        let sc = p.scene_at(s(2.5));
        let mid = sc.texts().next().unwrap();
        assert_eq!((mid.fx, mid.progress), (TextFx::None, 1.0));
        let sc = p.scene_at(s(4.5));
        let late = sc.texts().next().unwrap();
        assert_eq!(late.fx, TextFx::Swirl);
        assert!((late.progress - 0.5).abs() < 1e-3);
    }

    #[test]
    fn text_clips_trim_freely_and_split_keeps_each_effect_on_its_side() {
        let (mut p, v, _) = sample();
        let id = p
            .add_text_clip(v, s(0.0), s(2.0), TextSpec::default())
            .unwrap();
        p.trim_end(id, s(30.0)).unwrap();
        assert_eq!(p.clip(id).unwrap().1.duration, s(30.0));
        let right = p.split_clip(id, s(10.0)).unwrap();
        assert_eq!(
            p.clip(id).unwrap().1.text.as_ref().unwrap().fx_out,
            TextFx::None
        );
        assert_eq!(
            p.clip(right).unwrap().1.text.as_ref().unwrap().fx_in,
            TextFx::None
        );
        assert_eq!(
            p.clip(right).unwrap().1.text.as_ref().unwrap().fx_out,
            TextFx::Fade
        );
    }

    #[test]
    fn tracks_can_be_added_removed_and_are_capped() {
        let mut p = Project::default();
        let v2 = p.add_track_of(TrackKind::Video).unwrap();
        assert_eq!(p.track(v2).unwrap().name, "Video 2");
        let a2 = p.add_track_of(TrackKind::Audio).unwrap();
        assert_eq!(p.track(a2).unwrap().name, "Audio 2");
        p.remove_track(v2).unwrap();
        let first = p.tracks[0].id;
        assert_eq!(
            p.remove_track(first),
            Err(EditError::TrackNotRemovable),
            "last video track stays"
        );
        for _ in 0..20 {
            let _ = p.add_track_of(TrackKind::Video);
        }
        assert_eq!(
            p.tracks
                .iter()
                .filter(|t| t.kind == TrackKind::Video)
                .count(),
            MAX_TRACKS_PER_KIND
        );
        assert_eq!(
            p.add_track_of(TrackKind::Video),
            Err(EditError::TooManyTracks)
        );
    }

    #[test]
    fn non_empty_track_cannot_be_removed() {
        let (mut p, _, m) = sample();
        let v2 = p.add_track_of(TrackKind::Video).unwrap();
        p.add_clip(v2, m, s(0.0)).unwrap();
        assert_eq!(p.remove_track(v2), Err(EditError::TrackNotRemovable));
    }

    #[test]
    fn cues_become_text_clips_without_overlap() {
        let mut p = Project::default();
        let v = p.tracks[0].id;
        let cues = vec![
            Cue {
                start: s(1.0),
                end: s(4.0),
                text: "eins".into(),
            },
            Cue {
                start: s(3.0),
                end: s(5.0),
                text: "zwei".into(),
            },
            Cue {
                start: s(5.0),
                end: s(5.01),
                text: "zu kurz".into(),
            },
        ];
        let ids = p.import_cues(v, &cues, &TextSpec::subtitle(), Tick::ZERO);
        assert_eq!(ids.len(), 2);
        let first = p.clip(ids[0]).unwrap().1;
        assert_eq!(first.end(), s(3.0), "cut short where the next cue starts");
        assert_eq!(first.text.as_ref().unwrap().text, "eins");
    }

    #[test]
    fn restyle_changes_every_text_but_not_the_words() {
        let mut p = Project::default();
        let v = p.tracks[0].id;
        let a = p
            .add_text_clip(
                v,
                s(0.0),
                s(1.0),
                TextSpec {
                    text: "a".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let b = p
            .add_text_clip(
                v,
                s(2.0),
                s(1.0),
                TextSpec {
                    text: "b".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let style = TextSpec {
            color: [255, 0, 0, 255],
            ..Default::default()
        };
        p.restyle_track_texts(v, &style);
        for (id, word) in [(a, "a"), (b, "b")] {
            let t = p.clip(id).unwrap().1.text.clone().unwrap();
            assert_eq!(t.text, word);
            assert_eq!(t.color, [255, 0, 0, 255]);
        }
    }

    #[test]
    fn old_projects_without_text_field_still_load() {
        let json = r#"{"id":1,"media":2,"start":0,"duration":5,"source_in":0,"color":1,"transition_in":null}"#;
        let c: Clip = serde_json::from_str(json).unwrap();
        assert!(c.text.is_none());
    }

    #[test]
    fn ensure_media_reuses_same_source() {
        let mut p = Project::default();
        let a = p.ensure_media(
            "x".into(),
            MediaSource::File("/a.mp4".into()),
            MediaKind::Video,
            s(1.0),
            (1, 1),
        );
        let b = p.ensure_media(
            "y".into(),
            MediaSource::File("/a.mp4".into()),
            MediaKind::Video,
            s(1.0),
            (1, 1),
        );
        assert_eq!(a, b);
        assert_eq!(p.media.len(), 1);
    }

    #[test]
    fn media_in_use_cannot_be_removed() {
        let (mut p, v, m) = sample();
        let c = p.add_clip(v, m, s(0.0)).unwrap();
        assert_eq!(p.remove_media(m), Err(EditError::MediaInUse));
        p.delete_clip(c, false).unwrap();
        p.remove_media(m).unwrap();
        assert!(p.media(m).is_none());
    }

    #[test]
    fn group_shift_moves_together_and_stops_at_fixed_clips() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        let fixed = p.add_clip(v, m, s(12.0)).unwrap();
        let origins = vec![(a, s(0.0)), (b, s(5.0))];
        let applied = p.shift_clips(&origins, s(10.0));
        assert_eq!(
            applied,
            s(2.0),
            "b may only travel until it touches the fixed clip"
        );
        assert_eq!(p.clip(a).unwrap().1.start, s(2.0));
        assert_eq!(p.clip(b).unwrap().1.start, s(7.0));
        // dragging left cannot cross time zero
        let origins = vec![(a, s(2.0)), (b, s(7.0))];
        assert_eq!(p.shift_clips(&origins, s(-9.0)), s(-2.0));
        assert_eq!(p.clip(fixed).unwrap().1.start, s(12.0));
    }

    #[test]
    fn blend_on_all_cuts_of_a_selection() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        let c = p.add_clip(v, m, s(10.0)).unwrap();
        let d = p.add_clip(v, m, s(20.0)).unwrap(); // not touching c
        let t = Transition::new(TransitionKind::Iris);
        assert_eq!(p.apply_transition_to_cuts(&[a, b, c, d], t), 2);
        assert!(p.clip(b).unwrap().1.transition_in.is_some());
        assert!(p.clip(c).unwrap().1.transition_in.is_some());
        assert!(p.clip(d).unwrap().1.transition_in.is_none());
        assert!(p.clip(a).unwrap().1.transition_in.is_none());
        // a single clip: its own cut, or the cut after it when it is the first
        assert_eq!(
            p.apply_transition_to_cuts(&[a], Transition::new(TransitionKind::Zoom)),
            1
        );
        assert_eq!(
            p.clip(b).unwrap().1.transition_in.unwrap().kind,
            TransitionKind::Zoom
        );
    }

    #[test]
    fn used_media_and_canvas() {
        let (mut p, v, m) = sample();
        assert!(p.used_media().is_empty());
        p.add_clip(v, m, s(0.0)).unwrap();
        assert!(p.used_media().contains(&m));
        p.add_text_clip(v, s(30.0), s(1.0), TextSpec::default())
            .unwrap();
        assert_eq!(p.used_media().len(), 1, "text clips use no media");
        p.set_canvas((720, 1280));
        assert_eq!(p.size, (720, 1280));
        assert_eq!(crate::preview_size(9, 16), (720, 1280));
        assert_eq!(crate::preview_size(16, 9), (1280, 720));
    }

    #[test]
    fn old_projects_default_to_contain() {
        let json = serde_json::to_string(&Project::default())
            .unwrap()
            .replace("\"fit\":\"Contain\",", "");
        let p: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(p.fit, FitMode::Contain);
    }

    #[test]
    fn blocked_group_move_keeps_the_blends_inside_the_group() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        let c = p.add_clip(v, m, s(10.0)).unwrap();
        p.set_transition(b, Some(Transition::new(TransitionKind::ClawWipe)))
            .unwrap();
        p.set_transition(c, Some(Transition::new(TransitionKind::PawSpin)))
            .unwrap();
        assert_eq!(
            p.shift_clips(&[(a, s(0.0)), (b, s(5.0))], s(1.0)),
            Tick::ZERO
        );
        assert!(
            p.clip(b).unwrap().1.transition_in.is_some(),
            "blend between the moved clips stays"
        );
        assert!(
            p.clip(c).unwrap().1.transition_in.is_some(),
            "the fixed clip keeps its blend"
        );
    }

    #[test]
    fn a_blend_survives_a_trim_that_comes_back() {
        let (mut p, v, m) = sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        p.set_transition(b, Some(Transition::new(TransitionKind::Push)))
            .unwrap();
        p.hold_transitions(true);
        p.trim_end(a, s(4.0)).unwrap(); // gap opens while dragging
        p.trim_end(a, s(5.0)).unwrap(); // and closes again
        p.hold_transitions(false);
        assert!(p.clip(b).unwrap().1.transition_in.is_some());
        // but when the drag ends with a gap, the blend goes
        p.hold_transitions(true);
        p.trim_end(a, s(4.0)).unwrap();
        p.hold_transitions(false);
        assert!(p.clip(b).unwrap().1.transition_in.is_none());
    }

    fn png_sample() -> (Project, Id, Id) {
        let mut p = Project::default();
        let m = p.add_media(
            "logo".into(),
            MediaSource::Sample(0),
            MediaKind::Image,
            s(5.0),
            (100, 100),
        );
        p.set_media_alpha(m, true);
        let v = p.tracks[0].id;
        (p, v, m)
    }

    #[test]
    fn transparent_images_become_graphics_over_the_picture() {
        let (mut p, v, m) = png_sample();
        let plain = p.add_media(
            "bg".into(),
            MediaSource::Sample(1),
            MediaKind::Image,
            s(5.0),
            (100, 100),
        );
        let bg = p.add_clip(v, plain, s(0.0)).unwrap();
        let v2 = p.add_track_of(TrackKind::Video).unwrap();
        let logo = p.add_clip(v2, m, s(1.0)).unwrap();
        assert!(
            p.clip(logo).unwrap().1.is_overlay(),
            "alpha media defaults to an overlay"
        );
        assert!(!p.clip(bg).unwrap().1.is_overlay());
        // the picture is still the background clip, the graphic is a layer
        match p.video_frame_at(s(2.0)) {
            VideoFrame::Single(src) => assert_eq!(src.media, plain),
            other => panic!("{other:?}"),
        }
        let sc = p.scene_at(s(2.0));
        assert_eq!(sc.images().count(), 1);
        assert_eq!(sc.images().next().unwrap().media, m);
        // a graphic alone gives no picture
        let (mut q, qv, qm) = png_sample();
        q.add_clip(qv, qm, s(0.0)).unwrap();
        assert_eq!(q.video_frame_at(s(1.0)), VideoFrame::Black);
    }

    #[test]
    fn graphics_can_be_switched_and_never_take_blends() {
        let (mut p, v, m) = png_sample();
        let a = p.add_clip(v, m, s(0.0)).unwrap();
        let b = p.add_clip(v, m, s(5.0)).unwrap();
        assert_eq!(
            p.set_transition(b, Some(Transition::new(TransitionKind::Push))),
            Err(EditError::NoNeighbour)
        );
        p.set_overlay(a, None).unwrap();
        p.set_overlay(b, None).unwrap();
        p.set_transition(b, Some(Transition::new(TransitionKind::Push)))
            .unwrap();
        let t = p
            .add_text_clip(v, s(20.0), s(1.0), TextSpec::default())
            .unwrap();
        assert!(
            p.set_overlay(t, Some(OverlaySpec::default())).is_err(),
            "text is not an image"
        );
    }

    #[test]
    fn graphic_fades_like_text_and_split_keeps_each_fade_on_its_side() {
        let (mut p, v, m) = png_sample();
        let id = p.add_overlay_clip(v, m, s(0.0)).unwrap();
        {
            let o = p.overlay_of_mut(id).unwrap();
            o.fx_in = TextFx::Swirl;
            o.fx_in_dur = s(1.0);
        }
        let sc = p.scene_at(s(0.5));
        let layer = sc.images().next().unwrap();
        assert_eq!(layer.fx, TextFx::Swirl);
        assert!((layer.progress - 0.5).abs() < 1e-3);
        let right = p.split_clip(id, s(2.5)).unwrap();
        assert_eq!(
            p.clip(id).unwrap().1.overlay.as_ref().unwrap().fx_out,
            TextFx::None
        );
        assert_eq!(
            p.clip(right).unwrap().1.overlay.as_ref().unwrap().fx_in,
            TextFx::None
        );
        p.trim_end(right, s(60.0)).unwrap();
        assert_eq!(p.clip(right).unwrap().1.duration, s(57.5));
    }

    #[test]
    fn style_goes_onto_many_texts_at_once() {
        let mut p = Project::default();
        let v = p.tracks[0].id;
        let a = p
            .add_text_clip(
                v,
                s(0.0),
                s(1.0),
                TextSpec {
                    text: "eins".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let b = p
            .add_text_clip(
                v,
                s(2.0),
                s(1.0),
                TextSpec {
                    text: "zwei".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let c = p
            .add_text_clip(
                v,
                s(4.0),
                s(1.0),
                TextSpec {
                    text: "drei".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        let style = TextSpec {
            color: [9, 9, 9, 255],
            size: 0.2,
            font: "comic".into(),
            ..Default::default()
        };
        assert_eq!(p.apply_style_to_texts(&[a, b], &style, true), 2);
        assert_eq!(p.clip(a).unwrap().1.text.as_ref().unwrap().font, "comic");
        assert_eq!(p.clip(b).unwrap().1.text.as_ref().unwrap().text, "zwei");
        assert_eq!(
            p.clip(c).unwrap().1.text.as_ref().unwrap().font,
            "rounded",
            "unselected stays"
        );
        // without the position, every text stays where it was
        let moved = TextSpec {
            pos: [0.1, 0.1],
            color: [1, 1, 1, 255],
            ..Default::default()
        };
        p.apply_style_to_texts(&[a], &moved, false);
        let t = p.clip(a).unwrap().1.text.clone().unwrap();
        assert_eq!((t.color, t.pos), ([1, 1, 1, 255], [0.5, 0.5]));
    }

    #[test]
    fn old_clips_and_media_without_overlay_fields_load() {
        let json =
            r#"{"id":1,"name":"x","source":{"Sample":0},"kind":"Image","duration":1,"size":[1,1]}"#;
        let m: MediaItem = serde_json::from_str(json).unwrap();
        assert!(!m.alpha);
    }
}
