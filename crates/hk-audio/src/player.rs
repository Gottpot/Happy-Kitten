use crate::decode::Pcm;
use crate::mix::{ClipPlan, mix_block};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hk_core::Id;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Default)]
struct Plan {
    clips: Vec<ClipPlan>,
    pcm: HashMap<Id, Arc<Pcm>>,
}

struct Shared {
    plan: Mutex<Plan>,
    playing: AtomicBool,
    /// Timeline position in output frames; the audio clock the picture follows.
    pos: AtomicU64,
}

/// Plays the mixed timeline audio. The stream stays open and outputs silence
/// while paused, so starting playback has no device start-up delay.
pub struct Player {
    shared: Arc<Shared>,
    rate: u32,
    _stream: cpal::Stream,
}

impl Player {
    /// `None` when the machine has no usable audio output.
    pub fn new() -> Option<Player> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let supported = device.default_output_config().ok()?;
        let channels = supported.channels().max(1) as usize;
        let rate = supported.sample_rate();
        if supported.sample_format() != cpal::SampleFormat::F32 {
            log::warn!("audio output is not f32; playing without sound");
            return None;
        }
        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: rate,
            buffer_size: cpal::BufferSize::Default,
        };
        let shared = Arc::new(Shared {
            plan: Mutex::new(Plan::default()),
            playing: AtomicBool::new(false),
            pos: AtomicU64::new(0),
        });
        let cb = shared.clone();
        let mut scratch: Vec<f32> = Vec::new();
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _| {
                    data.fill(0.0);
                    if !cb.playing.load(Ordering::Relaxed) {
                        return;
                    }
                    let frames = data.len() / channels;
                    let pos = cb.pos.load(Ordering::Relaxed);
                    scratch.resize(frames * 2, 0.0);
                    // Never block the audio thread: if the UI holds the plan, this block is silent.
                    if let Ok(plan) = cb.plan.try_lock() {
                        mix_block(&plan.clips, &plan.pcm, rate, pos, &mut scratch);
                        for (i, frame) in data.chunks_exact_mut(channels).enumerate() {
                            let (l, r) = (scratch[i * 2], scratch[i * 2 + 1]);
                            match frame {
                                [only] => *only = (l + r) * 0.5,
                                [a, b, ..] => {
                                    *a = l;
                                    *b = r;
                                }
                                [] => {}
                            }
                        }
                    }
                    cb.pos.fetch_add(frames as u64, Ordering::Relaxed);
                },
                |e| log::warn!("audio stream error: {e}"),
                None,
            )
            .ok()?;
        stream.play().ok()?;
        Some(Player {
            shared,
            rate,
            _stream: stream,
        })
    }

    pub fn set_plan(&self, clips: Vec<ClipPlan>, pcm: HashMap<Id, Arc<Pcm>>) {
        let mut plan = self
            .shared
            .plan
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        plan.clips = clips;
        plan.pcm = pcm;
    }

    pub fn play_from(&self, secs: f64) {
        self.seek(secs);
        self.shared.playing.store(true, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.shared.playing.store(false, Ordering::Relaxed);
    }

    pub fn seek(&self, secs: f64) {
        let frames = (secs.max(0.0) * self.rate as f64) as u64;
        self.shared.pos.store(frames, Ordering::Relaxed);
    }

    pub fn position_secs(&self) -> f64 {
        self.shared.pos.load(Ordering::Relaxed) as f64 / self.rate as f64
    }
}
