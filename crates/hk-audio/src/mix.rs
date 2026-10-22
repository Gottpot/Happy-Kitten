use crate::decode::Pcm;
use hk_core::Id;
use std::collections::HashMap;
use std::sync::Arc;

/// One clip of sound on the timeline, in seconds.
#[derive(Clone, Debug, PartialEq)]
pub struct ClipPlan {
    pub media: Id,
    pub start: f64,
    pub duration: f64,
    pub source_in: f64,
    pub gain: f32,
}

/// Short ramp at clip edges so cuts never click.
const FADE: f64 = 0.004;

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
        if clip.start >= block_end || clip.start + clip.duration <= block_start {
            continue;
        }
        let Some(p) = pcm.get(&clip.media) else {
            continue;
        };
        let src_rate = p.rate as f64;
        // first/last output frame of this block that fall inside the clip
        let first = ((clip.start * rate).ceil() as i64 - start_frame as i64).clamp(0, frames as i64)
            as usize;
        let last = (((clip.start + clip.duration) * rate).ceil() as i64 - start_frame as i64)
            .clamp(0, frames as i64) as usize;
        for i in first..last {
            let t = (start_frame + i as u64) as f64 / rate;
            let rel = t - clip.start;
            let fade = (rel / FADE)
                .min((clip.duration - rel) / FADE)
                .clamp(0.0, 1.0) as f32;
            let (l, r) = p.at((clip.source_in + rel) * src_rate);
            let g = clip.gain * fade;
            out[i * 2] += l * g;
            out[i * 2 + 1] += r * g;
        }
    }
    for s in out.iter_mut() {
        *s = s.clamp(-1.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let plan = vec![ClipPlan {
            media: 1,
            start: 1.0,
            duration: 0.5,
            source_in: 0.0,
            gain: 1.0,
        }];
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
        let plan = vec![ClipPlan {
            media: 1,
            start: 0.3,
            duration: 1.0,
            source_in: 0.0,
            gain: 1.0,
        }];
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
            ClipPlan {
                media: 1,
                start: 0.0,
                duration: 0.5,
                source_in: 1.0,
                gain: 1.0,
            },
            ClipPlan {
                media: 99,
                start: 0.0,
                duration: 0.5,
                source_in: 0.0,
                gain: 1.0,
            },
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
            ClipPlan {
                media: 1,
                start: 0.0,
                duration: 1.0,
                source_in: 0.0,
                gain: 1.0,
            },
            ClipPlan {
                media: 1,
                start: 0.0,
                duration: 1.0,
                source_in: 0.0,
                gain: 1.0,
            },
        ];
        let mut out = vec![0.0; 2 * 500];
        mix_block(&plan, &map, 1000, 100, &mut out);
        assert!(out.iter().all(|s| s.abs() <= 1.0));
    }
}
