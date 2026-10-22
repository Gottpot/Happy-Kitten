use crate::decode::{Pcm, decode_file};
use hk_core::Id;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

/// Decodes media audio in the background and keeps the results.
pub struct Library {
    pcm: HashMap<Id, Arc<Pcm>>,
    /// Media with no usable audio (silent videos, broken files).
    silent: HashSet<Id>,
    requested: HashSet<Id>,
    jobs: Sender<(Id, PathBuf)>,
    done: Receiver<(Id, Result<Pcm, String>)>,
    pub errors: Vec<String>,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    pub fn new() -> Self {
        let (jobs, rx) = channel::<(Id, PathBuf)>();
        let (tx, done) = channel();
        std::thread::Builder::new()
            .name("hk-audio-decode".into())
            .spawn(move || {
                while let Ok((id, path)) = rx.recv() {
                    if tx.send((id, decode_file(&path))).is_err() {
                        return;
                    }
                }
            })
            .ok();
        Library {
            pcm: HashMap::new(),
            silent: HashSet::new(),
            requested: HashSet::new(),
            jobs,
            done,
            errors: Vec::new(),
        }
    }

    /// Start decoding `media` once. `report` says whether failure is worth telling the user
    /// (audio files yes, videos that simply have no sound no).
    pub fn want(&mut self, media: Id, path: PathBuf) {
        if self.requested.insert(media) {
            let _ = self.jobs.send((media, path));
        }
    }

    /// Collect finished decodes; true when something new arrived.
    pub fn pump(&mut self, loud_failures: &HashSet<Id>) -> bool {
        let mut changed = false;
        while let Ok((id, result)) = self.done.try_recv() {
            match result {
                Ok(p) => {
                    self.pcm.insert(id, Arc::new(p));
                    changed = true;
                }
                Err(e) => {
                    self.silent.insert(id);
                    if loud_failures.contains(&id) {
                        self.errors.push(e);
                    }
                }
            }
        }
        changed
    }

    pub fn get(&self, media: Id) -> Option<&Arc<Pcm>> {
        self.pcm.get(&media)
    }

    pub fn snapshot(&self) -> HashMap<Id, Arc<Pcm>> {
        self.pcm.clone()
    }

    pub fn is_loading(&self, media: Id) -> bool {
        self.requested.contains(&media)
            && !self.pcm.contains_key(&media)
            && !self.silent.contains(&media)
    }

    /// Forget everything (new project).
    pub fn clear(&mut self) {
        self.requested.clear();
    }
}
