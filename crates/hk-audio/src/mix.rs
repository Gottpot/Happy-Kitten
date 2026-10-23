use crate::decode::Pcm;
use hk_core::{AudioFx, AudioSpec, Id};
use std::collections::HashMap;
use std::sync::Arc;

/// One clip of sound on the timeline, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipPlan {
    pub media: Id,
    pub start: f64,
    pub duration: f64,
    pub source_in: f64,
    pub audio: AudioSpec,
}

impl ClipPlan {
    /// A clip with neutral settings.
    pub fn plain(media: Id, start: f64, duration: f64, source_in: f64) -> Self {
        ClipPlan {
            media,
            start,
            duration,
            source_in,
            audio: AudioSpec::default(),
        }
    }

    /// Seconds after the clip's end during which an echo or reverb still rings.
    pub fn tail(&self) -> f64 {
        let a = self.audio.sanitized();
        if a.fx_amount <= 0.0 {
            return 0.0;
        }
        a.fx.tail_secs(a.fx_amount)
    }
}

/// Short ramp at clip edges so cuts never click.
const FADE: f64 = 0.004;
const ECHO_DELAY: f64 = 0.31;
const ECHO_TAPS: usize = 4;
const HALL_TAPS: usize = 40;

fn smooth(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// The clip's own sound `rel` seconds after its start with fades applied, before any effect.
/// Silent outside the clip.
fn dry(p: &Pcm, clip: &ClipPlan, a: &AudioSpec, rel: f64) -> (f32, f32) {
    if rel < 0.0 || rel >= clip.duration {
        return (0.0, 0.0);
    }
    let mut env = (rel / FADE)
        .min((clip.duration - rel) / FADE)
        .clamp(0.0, 1.0);
    let fade_in = a.fade_in.secs();
    if fade_in > 0.0 {
        env *= smooth(rel / fade_in);
    }
    let fade_out = a.fade_out.secs();
    if fade_out > 0.0 {
        env *= smooth((clip.duration - rel) / fade_out);
    }
    let (l, r) = p.at((clip.source_in + rel) * p.rate as f64);
    (l * env as f32, r * env as f32)
}

/// Delays and gains of the reverb taps for one ear. Fixed numbers, so every block (and the
/// export) renders the same reverb.
type Taps = Vec<(f64, f32)>;

fn hall_taps(length: f64, seed: u32) -> Taps {
    let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(12345);
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state as f64 / u32::MAX as f64
    };
    (0..HALL_TAPS)
        .map(|i| {
            // early reflections are dense, later ones spread out
            let x = (i as f64 + next()) / HALL_TAPS as f64;
            let delay = 0.011 + (length - 0.011) * x * x;
            let decay = (-4.0 * delay / length).exp();
            let sign = if next() < 0.5 { -1.0 } else { 1.0 };
            (delay, (sign * decay * (0.5 + 0.5 * next()) * 0.32) as f32)
        })
        .collect()
}

/// Everything that depends on the clip's effect, prepared once per block.
struct Voice {
    clip_spec: AudioSpec,
    hall: Option<(Taps, Taps)>,
    out_rate: f64,
}

impl Voice {
    fn new(clip: &ClipPlan, out_rate: f64) -> Voice {
        let spec = clip.audio.sanitized();
        let hall = (spec.fx == AudioFx::Hall).then(|| {
            let len = 0.4 + 1.6 * spec.fx_amount as f64;
            (hall_taps(len, 1), hall_taps(len, 2))
        });
        Voice {
            clip_spec: spec,
            hall,
            out_rate,
        }
    }

    /// One stereo sample of the clip, with volume and effect, `rel` seconds after its start.
    fn sample(&self, p: &Pcm, clip: &ClipPlan, rel: f64) -> (f32, f32) {
        let a = &self.clip_spec;
        let amount = a.fx_amount;
        let (l, r) = dry(p, clip, a, rel);
        let (l, r) = match a.fx {
            AudioFx::None => (l, r),
            AudioFx::Echo => {
                let (mut wl, mut wr) = (0.0, 0.0);
                let mut g = 0.55;
                for k in 1..=ECHO_TAPS {
                    let (el, er) = dry(p, clip, a, rel - ECHO_DELAY * k as f64);
                    wl += el * g;
                    wr += er * g;
                    g *= 0.55;
                }
                // a little ping-pong between the ears
                (
                    l + (wl * 0.8 + wr * 0.2) * amount * 1.5,
                    r + (wr * 0.8 + wl * 0.2) * amount * 1.5,
                )
            }
            AudioFx::Hall => {
                let (tl, tr) = self
                    .hall
                    .as_ref()
                    .map_or((&[][..], &[][..]), |(a, b)| (&a[..], &b[..]));
                let mono = |t: f64| {
                    let (x, y) = dry(p, clip, a, t);
                    (x + y) * 0.5
                };
                let wl: f32 = tl.iter().map(|(d, g)| mono(rel - d) * g).sum();
                let wr: f32 = tr.iter().map(|(d, g)| mono(rel - d) * g).sum();
                (l + wl * amount * 1.6, r + wr * amount * 1.6)
            }
            AudioFx::Muffled => {
                let n = 2 + (amount * 10.0).round() as i32;
                let (mut sl, mut sr) = (0.0, 0.0);
                for j in -n..=n {
                    let (x, y) = dry(p, clip, a, rel + j as f64 / self.out_rate);
                    sl += x;
                    sr += y;
                }
                let k = 1.0 / (2 * n + 1) as f32;
                (sl * k, sr * k)
            }
            AudioFx::Telephone => {
                let narrow = |t0: f64, stride: f64, n: i32| {
                    let (mut sl, mut sr) = (0.0, 0.0);
                    for j in -n..=n {
                        let (x, y) = dry(p, clip, a, t0 + j as f64 * stride / self.out_rate);
                        sl += x;
                        sr += y;
                    }
                    let k = 1.0 / (2 * n + 1) as f32;
                    (sl * k, sr * k)
                };
                let (fl, fr) = narrow(rel, 1.0, 2);
                let (sl, sr) = narrow(rel, 3.0, 8);
                let mono = ((fl - sl) + (fr - sr)) * 0.5 * 1.7;
                (l + (mono - l) * amount, r + (mono - r) * amount)
            }
        };
        (l * a.volume, r * a.volume)
    }
}

/// Mix `out.len() / 2` stereo frames starting at output frame `start_frame`.
/// `out` is overwritten.
pub fn mix_block(
    plan: &[ClipPlan],
    pcm: &HashMap<Id, Arc<Pcm>>,
    out_rate: u32,
    start_frame: u64,
    out: &mut [f32],
) {
    out.fill(0.0);
    let rate = out_rate.max(1) as f64;
    let frames = out.len() / 2;
    let block_start = start_frame as f64 / rate;
    let block_end = (start_frame + frames as u64) as f64 / rate;
    for clip in plan {
        let end = clip.start + clip.duration + clip.tail();
        if clip.start >= block_end || end <= block_start {
            continue;
        }
        let Some(p) = pcm.get(&clip.media) else {
            continue;
        };
        let voice = Voice::new(clip, rate);
        // first/last output frame of this block that fall inside the clip (and its tail)
        let first = ((clip.start * rate).ceil() as i64 - start_frame as i64).clamp(0, frames as i64)
            as usize;
        let last =
            ((end * rate).ceil() as i64 - start_frame as i64).clamp(0, frames as i64) as usize;
        for i in first..last {
            let t = (start_frame + i as u64) as f64 / rate;
            let (l, r) = voice.sample(p, clip, t - clip.start);
            out[i * 2] += l;
            out[i * 2 + 1] += r;
        }
    }
    for s in out.iter_mut() {
        *s = s.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hk_core::AudioFx;

    fn constant(rate: u32, secs: f64, v: f32) -> Arc<Pcm> {
        let frames = (rate as f64 * secs) as usize;
        Arc::new(Pcm {
            rate,
            samples: vec![v; frames * 2],
            peaks: vec![],
        })
    }

    #[test]
    fn clip_sounds_only_inside_its_span() {
        let mut map = HashMap::new();
        map.insert(1, constant(1000, 2.0, 0.5));
        let plan = vec![ClipPlan::plain(1, 1.0, 0.5, 0.0)];
        let mut out = vec![0.0; 2 * 3000];
        mix_block(&plan, &map, 1000, 0, &mut out);
        assert_eq!(out[2 * 500], 0.0, "before the clip");
        assert!((out[2 * 1250] - 0.5).abs() < 1e-4, "inside the clip");
        assert_eq!(out[2 * 1700], 0.0, "after the clip");
    }

    #[test]
    fn block_boundaries_do_not_matter() {
        let mut map = HashMap::new();
        map.insert(1, constant(1000, 2.0, 0.25));
        let plan = vec![ClipPlan::plain(1, 0.3, 1.0, 0.0)];
        let mut whole = vec![0.0; 2 * 1500];
        mix_block(&plan, &map, 1000, 0, &mut whole);
        let mut parts = vec![0.0; 2 * 1500];
        for (n, chunk) in parts.chunks_mut(2 * 100).enumerate() {
            mix_block(&plan, &map, 1000, (n * 100) as u64, chunk);
        }
        assert_eq!(whole, parts);
    }

    #[test]
    fn source_offset_and_missing_media_are_handled() {
        let mut map = HashMap::new();
        let mut samples = vec![0.0; 2 * 1000];
        samples.extend(vec![0.8; 2 * 1000]);
        map.insert(
            1,
            Arc::new(Pcm {
                rate: 1000,
                samples,
                peaks: vec![],
            }),
        );
        let plan = vec![
            ClipPlan::plain(1, 0.0, 0.5, 1.0),
            ClipPlan::plain(99, 0.0, 0.5, 0.0),
        ];
        let mut out = vec![0.0; 2 * 400];
        mix_block(&plan, &map, 1000, 0, &mut out);
        assert!((out[2 * 200] - 0.8).abs() < 1e-4);
    }

    #[test]
    fn loud_overlaps_are_clamped() {
        let mut map = HashMap::new();
        map.insert(1, constant(1000, 1.0, 0.9));
        let plan = vec![
            ClipPlan::plain(1, 0.0, 1.0, 0.0),
            ClipPlan::plain(1, 0.0, 1.0, 0.0),
        ];
        let mut out = vec![0.0; 2 * 500];
        mix_block(&plan, &map, 1000, 100, &mut out);
        assert!(out.iter().all(|s| s.abs() <= 1.0));
    }

    fn with(spec: AudioSpec, start: f64, duration: f64) -> ClipPlan {
        ClipPlan {
            audio: spec,
            ..ClipPlan::plain(1, start, duration, 0.0)
        }
    }

    fn render(plan: &[ClipPlan], rate: u32, frames: usize) -> Vec<f32> {
        let mut map = HashMap::new();
        map.insert(1, constant(rate, 4.0, 0.5));
        let mut out = vec![0.0; 2 * frames];
        mix_block(plan, &map, rate, 0, &mut out);
        out
    }

    #[test]
    fn volume_scales_the_clip() {
        let plan = [with(
            AudioSpec {
                volume: 0.5,
                ..Default::default()
            },
            0.0,
            1.0,
        )];
        let out = render(&plan, 1000, 1000);
        assert!((out[2 * 500] - 0.25).abs() < 1e-4);
        let plan = [with(
            AudioSpec {
                volume: 0.0,
                ..Default::default()
            },
            0.0,
            1.0,
        )];
        assert!(render(&plan, 1000, 1000).iter().all(|s| *s == 0.0));
    }

    #[test]
    fn fades_ramp_the_start_and_the_end() {
        use hk_core::Tick;
        let spec = AudioSpec {
            fade_in: Tick::from_secs(0.4),
            fade_out: Tick::from_secs(0.4),
            ..Default::default()
        };
        let out = render(&[with(spec, 0.0, 2.0)], 1000, 2000);
        assert!(out[2 * 50].abs() < 0.1, "quiet at the start");
        assert!(out[2 * 200] > 0.2 && out[2 * 200] < 0.3, "half way up");
        assert!((out[2 * 1000] - 0.5).abs() < 1e-3, "full in the middle");
        assert!(out[2 * 1950].abs() < 0.1, "quiet at the end");
    }

    #[test]
    fn echo_repeats_after_the_clip_and_rings_out() {
        let spec = AudioSpec {
            fx: AudioFx::Echo,
            fx_amount: 0.8,
            ..Default::default()
        };
        let plan = [with(spec, 0.0, 0.2)];
        let out = render(&plan, 1000, 2000);
        assert!(out[2 * 100] > 0.0, "the sound itself");
        assert!(
            out[2 * 250].abs() < 1e-3,
            "silence between sound and first echo"
        );
        assert!(out[2 * 400].abs() > 0.01, "first echo after 0.31 s");
        assert!(out[2 * 1700].abs() < 1e-3, "rung out");
        assert!(plan[0].tail() > 1.0);
    }

    #[test]
    fn every_effect_is_independent_of_block_boundaries() {
        for fx in AudioFx::ALL {
            let spec = AudioSpec {
                fx,
                fx_amount: 0.7,
                ..Default::default()
            };
            let plan = [with(spec, 0.2, 1.0)];
            let mut map = HashMap::new();
            map.insert(1, constant(1000, 4.0, 0.3));
            let mut whole = vec![0.0; 2 * 3000];
            mix_block(&plan, &map, 1000, 0, &mut whole);
            let mut parts = vec![0.0; 2 * 3000];
            for (n, chunk) in parts.chunks_mut(2 * 137).enumerate() {
                mix_block(&plan, &map, 1000, (n * 137) as u64, chunk);
            }
            assert_eq!(whole, parts, "{fx:?}");
            assert!(whole.iter().all(|s| s.is_finite() && s.abs() <= 1.0));
        }
    }

    #[test]
    fn hall_adds_a_tail_and_muffled_removes_highs() {
        let spec = AudioSpec {
            fx: AudioFx::Hall,
            fx_amount: 0.8,
            ..Default::default()
        };
        let out = render(&[with(spec, 0.0, 0.3)], 8000, 8000 * 2);
        let tail: f32 = out[2 * 3000..2 * 8000].iter().map(|s| s.abs()).sum();
        assert!(tail > 0.5, "the room rings after the clip");
        // an alternating signal is all highs: the low-pass flattens it
        let mut map = HashMap::new();
        let alt: Vec<f32> = (0..2 * 4000)
            .map(|i| if (i / 2) % 2 == 0 { 0.8 } else { -0.8 })
            .collect();
        map.insert(
            1,
            Arc::new(Pcm {
                rate: 8000,
                samples: alt,
                peaks: vec![],
            }),
        );
        let spec = AudioSpec {
            fx: AudioFx::Muffled,
            fx_amount: 0.5,
            ..Default::default()
        };
        let mut out = vec![0.0; 2 * 1000];
        mix_block(&[with(spec, 0.0, 0.5)], &map, 8000, 1000, &mut out);
        assert!(out.iter().all(|s| s.abs() < 0.2), "highs are gone");
    }
}
