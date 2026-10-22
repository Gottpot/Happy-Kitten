//! One video file as a random-access frame source that is fast when the time
//! mostly moves forward (playback, export) and still exact when it jumps.

use crate::seq::SeqReader;
use crate::video::Reader;
use crate::{MediaError, Rgba};
use hk_core::Tick;
use std::path::{Path, PathBuf};

/// Within this distance ahead of the last frame we keep decoding sequentially.
const FORWARD_WINDOW: f64 = 1.5;
const EPS: f64 = 1e-3;

pub struct VideoSource {
    path: PathBuf,
    still: Reader,
    seq: Option<SeqReader>,
    seq_dim: u32,
    cur: Option<(f64, Rgba)>,
    peek: Option<(f64, Rgba)>,
}

impl VideoSource {
    pub fn open(path: &Path) -> Result<VideoSource, MediaError> {
        Ok(VideoSource {
            path: path.to_path_buf(),
            still: Reader::open(path)?,
            seq: None,
            seq_dim: 0,
            cur: None,
            peek: None,
        })
    }

    pub fn duration(&self) -> Tick {
        self.still.duration()
    }

    /// The frame shown at `secs`, scaled to fit `max_dim`.
    pub fn frame_at(&mut self, secs: f64, max_dim: u32) -> Result<Rgba, MediaError> {
        let secs = if secs.is_finite() { secs.max(0.0) } else { 0.0 };
        let end = self.duration().secs();
        let secs = if end > 0.0 {
            secs.min((end - 0.001).max(0.0))
        } else {
            secs
        };

        let sequential = self.seq.is_some()
            && self.seq_dim == max_dim
            && self
                .cur
                .as_ref()
                .is_some_and(|(pts, _)| secs >= *pts - EPS && secs - *pts < FORWARD_WINDOW);
        if !sequential {
            self.cur = None;
            self.peek = None;
            self.seq = SeqReader::open_at(&self.path, (secs - 0.1).max(0.0), max_dim).ok();
            self.seq_dim = max_dim;
        }
        if self.seq.is_some() {
            match self.advance_to(secs) {
                Ok(Some(img)) => return Ok(img),
                Ok(None) | Err(_) => {
                    // fall through to the exact single-frame path
                    self.seq = None;
                    self.cur = None;
                    self.peek = None;
                }
            }
        }
        self.still.frame_at(secs, max_dim)
    }

    fn advance_to(&mut self, secs: f64) -> Result<Option<Rgba>, MediaError> {
        let Some(seq) = self.seq.as_mut() else {
            return Ok(None);
        };
        loop {
            let next = match self.peek.take() {
                Some(f) => Some(f),
                None => seq.next_frame()?,
            };
            match next {
                Some((pts, img)) if pts <= secs + EPS || self.cur.is_none() => {
                    self.cur = Some((pts, img))
                }
                Some(ahead) => {
                    self.peek = Some(ahead);
                    break;
                }
                None => break,
            }
        }
        Ok(self.cur.as_ref().map(|(_, img)| img.clone()))
    }
}
