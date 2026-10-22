//! Glue between the project and `hk-audio`: decodes media sound, keeps the
//! mixer plan in step with the timeline and drives playback.

use hk_audio::{ClipPlan, Library, Pcm, Player};
use hk_core::{Id, MediaKind, MediaSource, Project, Tick, TrackKind};
use std::collections::HashSet;
use std::sync::Arc;

pub struct AudioEngine {
    player: Option<Player>,
    pub library: Library,
    last_plan: Vec<ClipPlan>,
    loaded: usize,
    pub errors: Vec<String>,
}

impl AudioEngine {
    pub fn new() -> Self {
        AudioEngine {
            player: Player::new(),
            library: Library::new(),
            last_plan: Vec::new(),
            loaded: 0,
            errors: Vec::new(),
        }
    }

    /// New or opened project: media ids start over, so drop everything decoded.
    pub fn reset(&mut self) {
        self.library = Library::new();
        self.last_plan.clear();
        self.loaded = 0;
        if let Some(p) = &self.player {
            p.set_plan(Vec::new(), Default::default());
        }
    }

    pub fn pcm(&self, media: Id) -> Option<&Arc<Pcm>> {
        self.library.get(media)
    }

    /// Call every frame. Returns true when new sound finished decoding.
    pub fn sync(&mut self, project: &Project) -> bool {
        let mut loud = HashSet::new();
        for m in &project.media {
            if let (MediaSource::File(path), true) = (
                &m.source,
                matches!(m.kind, MediaKind::Audio | MediaKind::Video),
            ) {
                self.library.want(m.id, path.clone());
                if m.kind == MediaKind::Audio {
                    loud.insert(m.id);
                }
            }
        }
        let arrived = self.library.pump(&loud);
        self.errors.append(&mut self.library.errors);

        let plan = self.plan(project);
        let loaded = self.library.snapshot().len();
        if plan != self.last_plan || loaded != self.loaded {
            if let Some(p) = &self.player {
                p.set_plan(plan.clone(), self.library.snapshot());
            }
            self.last_plan = plan;
            self.loaded = loaded;
        }
        arrived
    }

    /// Sound clips of the timeline that have finished decoding.
    pub fn plan(&self, project: &Project) -> Vec<ClipPlan> {
        let mut plan = Vec::new();
        for track in project.tracks.iter().filter(|t| !t.muted) {
            for c in &track.clips {
                let wants_sound = matches!(
                    (track.kind, project.media(c.media).map(|m| m.kind)),
                    (TrackKind::Audio, Some(MediaKind::Audio))
                        | (TrackKind::Video, Some(MediaKind::Video))
                );
                if wants_sound && self.library.get(c.media).is_some() {
                    plan.push(ClipPlan {
                        media: c.media,
                        start: c.start.secs(),
                        duration: c.duration.secs(),
                        source_in: c.source_in.secs(),
                        gain: 1.0,
                    });
                }
            }
        }
        plan
    }

    /// True while any sound that the timeline uses is still being decoded.
    pub fn busy(&self, project: &Project) -> bool {
        project
            .tracks
            .iter()
            .flat_map(|t| t.clips.iter())
            .any(|c| self.library.is_loading(c.media))
    }

    pub fn snapshot(&self) -> std::collections::HashMap<Id, Arc<Pcm>> {
        self.library.snapshot()
    }

    pub fn play(&self, at: Tick) {
        if let Some(p) = &self.player {
            p.play_from(at.secs());
        }
    }

    pub fn pause(&self) {
        if let Some(p) = &self.player {
            p.pause();
        }
    }

    pub fn seek(&self, at: Tick) {
        if let Some(p) = &self.player {
            p.seek(at.secs());
        }
    }

    /// The audio clock, when there is an output device.
    pub fn position(&self) -> Option<Tick> {
        self.player
            .as_ref()
            .map(|p| Tick::from_secs(p.position_secs()))
    }
}
