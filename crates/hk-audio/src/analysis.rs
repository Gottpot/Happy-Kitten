//! Looks at the sound at one moment of the timeline: how loud are the frequency bands, how much
//! bass, did a beat just hit. The equalizer effects draw from these numbers.
//!
//! Like the mixer this needs no history: everything is computed from the timeline position alone,
//! so the preview, the playback and the export see exactly the same numbers.

use crate::decode::Pcm;
use crate::mix::{ClipPlan, mix_block};
use hk_core::{AudioLevels, BANDS, BEAT_STEPS, Id};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::Arc;

const RATE: u32 = 44_100;
const N: usize = 2048;
const LOW_HZ: f32 = 40.0;
const HIGH_HZ: f32 = 14_000.0;
/// Bands that count as bass, mids and highs.
const BASS: std::ops::Range<usize> = 0..8;
const MID: std::ops::Range<usize> = 8..20;
const TREBLE: std::ops::Range<usize> = 20..BANDS;
/// Windows looked at, one every 50 ms into the past.
const STEP: f64 = 0.05;
const WINDOWS: usize = BEAT_STEPS + 5;

/// In-place radix-2 FFT; `re.len()` must be a power of two.
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        len <<= 1;
    }
}

/// Loudness of the [`BANDS`] frequency bands of `N` mono samples, 0..1 (0 at -60 dB, 1 at -6 dB).
fn band_levels(mono: &[f32]) -> [f32; BANDS] {
    let mut re = vec![0.0f32; N];
    let mut im = vec![0.0f32; N];
    for (i, (r, s)) in re.iter_mut().zip(mono).enumerate() {
        let hann = 0.5 - 0.5 * (2.0 * PI * i as f32 / N as f32).cos();
        *r = s * hann;
    }
    fft(&mut re, &mut im);
    let bin_hz = RATE as f32 / N as f32;
    let ratio = (HIGH_HZ / LOW_HZ).powf(1.0 / BANDS as f32);
    let mut out = [0.0f32; BANDS];
    for (b, level) in out.iter_mut().enumerate() {
        let f_lo = LOW_HZ * ratio.powi(b as i32);
        let f_hi = f_lo * ratio;
        let lo = ((f_lo / bin_hz).floor() as usize).clamp(1, N / 2 - 1);
        let hi = ((f_hi / bin_hz).ceil() as usize).clamp(lo + 1, N / 2);
        let mut peak = 0.0f32;
        for k in lo..hi {
            // a full-scale sine gives 1.0 (the Hann window passes half of it)
            let a = (re[k] * re[k] + im[k] * im[k]).sqrt() / (N as f32 * 0.25);
            peak = peak.max(a);
        }
        let db = 20.0 * peak.max(1e-6).log10();
        *level = ((db + 60.0) / 54.0).clamp(0.0, 1.0);
    }
    out
}

/// The `N` mono frames of the whole mix around `t` seconds. Silence before the start.
fn window_at(plan: &[ClipPlan], pcm: &HashMap<Id, Arc<Pcm>>, t: f64) -> Option<Vec<f32>> {
    let centre = (t * RATE as f64).round();
    let start = centre - (N / 2) as f64;
    if start < 0.0 {
        return None;
    }
    let mut stereo = vec![0.0f32; N * 2];
    mix_block(plan, pcm, RATE, start as u64, &mut stereo);
    Some(
        (0..N)
            .map(|i| (stereo[2 * i] + stereo[2 * i + 1]) * 0.5)
            .collect(),
    )
}

fn mean(v: &[f32]) -> f32 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f32>() / v.len() as f32
    }
}

/// The sound of the timeline at `t` seconds.
pub fn analyze(plan: &[ClipPlan], pcm: &HashMap<Id, Arc<Pcm>>, t: f64) -> AudioLevels {
    let mut history: Vec<[f32; BANDS]> = Vec::with_capacity(WINDOWS);
    for j in 0..WINDOWS {
        let bands = match window_at(plan, pcm, t - j as f64 * STEP) {
            Some(w) if w.iter().any(|s| *s != 0.0) => band_levels(&w),
            _ => [0.0; BANDS],
        };
        history.push(bands);
    }
    let mut levels = AudioLevels::SILENT;
    // the bands fall back slowly instead of flickering: the loudest of the last windows wins
    let release = [1.0f32, 0.82, 0.66, 0.52];
    for (i, band) in levels.bands.iter_mut().enumerate() {
        *band = history
            .iter()
            .zip(release)
            .map(|(h, r)| h[i] * r)
            .fold(0.0, f32::max);
    }
    levels.bass = mean(&levels.bands[BASS]);
    levels.mid = mean(&levels.bands[MID]);
    levels.treble = mean(&levels.bands[TREBLE]);
    let bass_at: Vec<f32> = history.iter().map(|h| mean(&h[BASS])).collect();
    for (k, beat) in levels.beat.iter_mut().enumerate() {
        // a beat is bass that is louder than it was just before
        let before = mean(&bass_at[k + 1..(k + 5).min(WINDOWS)]);
        *beat = ((bass_at[k] - before) * 3.0).clamp(0.0, 1.0);
    }
    levels.sanitized()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(rate: u32, secs: f64, hz: f32, amp: f32, from: f64) -> Arc<Pcm> {
        let frames = (rate as f64 * secs) as usize;
        let mut samples = vec![0.0f32; frames * 2];
        for i in 0..frames {
            let t = i as f64 / rate as f64;
            if t >= from {
                let v = amp * (2.0 * PI as f64 * hz as f64 * t).sin() as f32;
                samples[i * 2] = v;
                samples[i * 2 + 1] = v;
            }
        }
        Arc::new(Pcm {
            rate,
            samples,
            peaks: vec![],
        })
    }

    fn scene(pcm: Arc<Pcm>, secs: f64) -> (Vec<ClipPlan>, HashMap<Id, Arc<Pcm>>) {
        let mut map = HashMap::new();
        map.insert(1, pcm);
        (vec![ClipPlan::plain(1, 0.0, secs, 0.0)], map)
    }

    #[test]
    fn fft_finds_a_sine_in_the_right_bin() {
        let n = 256;
        let mut re: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * 10.0 * i as f32 / n as f32).sin())
            .collect();
        let mut im = vec![0.0; n];
        fft(&mut re, &mut im);
        let mags: Vec<f32> = re
            .iter()
            .zip(&im)
            .map(|(a, b)| (a * a + b * b).sqrt())
            .collect();
        let peak = (0..n / 2)
            .max_by(|a, b| mags[*a].total_cmp(&mags[*b]))
            .unwrap();
        assert_eq!(peak, 10);
    }

    #[test]
    fn a_1_khz_tone_lights_the_band_around_1_khz() {
        let (plan, map) = scene(tone(44_100, 3.0, 1000.0, 0.5, 0.0), 3.0);
        let l = analyze(&plan, &map, 1.5);
        let top = (0..BANDS)
            .max_by(|a, b| l.bands[*a].total_cmp(&l.bands[*b]))
            .unwrap();
        assert!((16..=19).contains(&top), "peak in band {top}");
        assert!(l.bands[top] > 0.6, "level {}", l.bands[top]);
        assert!(l.treble < 0.3 && l.bass < 0.3);
    }

    #[test]
    fn a_bass_tone_is_bass_and_silence_is_silent() {
        let (plan, map) = scene(tone(44_100, 3.0, 80.0, 0.8, 0.0), 3.0);
        let l = analyze(&plan, &map, 1.5);
        assert!(
            l.bass > l.treble + 0.2 && l.bass > 0.3,
            "bass {} treble {}",
            l.bass,
            l.treble
        );
        assert_eq!(analyze(&[], &HashMap::new(), 1.0), AudioLevels::SILENT);
        assert_eq!(analyze(&plan, &map, -1.0), AudioLevels::SILENT);
    }

    #[test]
    fn a_beat_shows_up_only_when_the_bass_jumps() {
        // silence, then a loud bass tone from 1.0 s on
        let (plan, map) = scene(tone(44_100, 3.0, 70.0, 0.9, 1.0), 3.0);
        let hit = analyze(&plan, &map, 1.06);
        assert!(hit.beat[0] > 0.5, "beat {:?}", hit.beat);
        let steady = analyze(&plan, &map, 2.2);
        assert!(steady.beat[0] < 0.1, "beat {:?}", steady.beat);
        // a tenth of a second later the same hit sits two steps back
        let later = analyze(&plan, &map, 1.16);
        assert!(later.beat[2] > 0.4, "beat {:?}", later.beat);
    }
}
