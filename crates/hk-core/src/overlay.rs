//! Picture overlays: a PNG (or any image) laid over the video with its own size, position,
//! rotation, opacity and fade-in / fade-out effect. Transparency of the file is kept.

use crate::text::TextFx;
use crate::time::Tick;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OverlaySpec {
    /// Width of the picture as a fraction of the frame width.
    pub scale: f32,
    /// Centre of the picture, 0..1 of width and height.
    pub pos: [f32; 2],
    /// Degrees, clockwise.
    pub rotation: f32,
    pub opacity: f32,
    pub fx_in: TextFx,
    pub fx_in_dur: Tick,
    pub fx_out: TextFx,
    pub fx_out_dur: Tick,
}

impl Default for OverlaySpec {
    fn default() -> Self {
        OverlaySpec {
            scale: 0.3,
            pos: [0.5, 0.5],
            rotation: 0.0,
            opacity: 1.0,
            fx_in: TextFx::Zoom,
            fx_in_dur: Tick(352_800_000),
            fx_out: TextFx::Fade,
            fx_out_dur: Tick(352_800_000),
        }
    }
}

impl OverlaySpec {
    /// Hash of what changes the composed picture (not the animation).
    pub fn raster_key(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.scale.to_bits().hash(&mut h);
        self.pos[0].to_bits().hash(&mut h);
        self.pos[1].to_bits().hash(&mut h);
        self.rotation.to_bits().hash(&mut h);
        self.opacity.to_bits().hash(&mut h);
        h.finish()
    }

    /// Apply only what changed between `old` and `new` (see `TextSpec::apply_diff`).
    pub fn apply_diff(&mut self, old: &OverlaySpec, new: &OverlaySpec) {
        macro_rules! carry {
            ($($f:ident),*) => { $( if old.$f != new.$f { self.$f = new.$f.clone(); } )* };
        }
        carry!(
            scale, pos, rotation, opacity, fx_in, fx_in_dur, fx_out, fx_out_dur
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_ignores_animation_only() {
        let a = OverlaySpec::default();
        let mut b = a.clone();
        b.fx_in = TextFx::Swirl;
        assert_eq!(a.raster_key(), b.raster_key());
        b.scale = 0.5;
        assert_ne!(a.raster_key(), b.raster_key());
    }

    #[test]
    fn diff_carries_only_edited_fields() {
        let old = OverlaySpec::default();
        let new = OverlaySpec {
            opacity: 0.4,
            ..old.clone()
        };
        let mut other = OverlaySpec {
            scale: 0.9,
            ..old.clone()
        };
        other.apply_diff(&old, &new);
        assert_eq!((other.opacity, other.scale), (0.4, 0.9));
    }
}
