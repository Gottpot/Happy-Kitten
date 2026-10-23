//! Per-clip sound settings: volume, fade-in / fade-out and one effect. The mixer applies them
//! to every sample, so preview, playback and export always sound the same.

use crate::time::Tick;
use serde::{Deserialize, Serialize};

pub const MAX_VOLUME: f32 = 2.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFx {
    #[default]
    None,
    Echo,
    Hall,
    Muffled,
    Telephone,
}

impl AudioFx {
    pub const ALL: [AudioFx; 5] = [
        AudioFx::None,
        AudioFx::Echo,
        AudioFx::Hall,
        AudioFx::Muffled,
        AudioFx::Telephone,
    ];

    /// German name, also the translation key.
    pub fn name(self) -> &'static str {
        match self {
            AudioFx::None => "Ohne",
            AudioFx::Echo => "Echo",
            AudioFx::Hall => "Hall",
            AudioFx::Muffled => "Dumpf",
            AudioFx::Telephone => "Telefon",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            AudioFx::None => "Der Ton bleibt, wie er ist",
            AudioFx::Echo => "Der Ton wiederholt sich und klingt leiser aus",
            AudioFx::Hall => "Klingt wie in einem großen Raum",
            AudioFx::Muffled => "Wie hinter einer Wand: die Höhen fehlen",
            AudioFx::Telephone => "Schmaler Klang wie aus einem alten Telefon",
        }
    }

    /// Seconds the effect rings on after the clip has ended.
    pub fn tail_secs(self, amount: f32) -> f64 {
        let a = amount.clamp(0.0, 1.0) as f64;
        match self {
            AudioFx::Echo => 0.31 * 4.0 + 0.1,
            AudioFx::Hall => 0.4 + 1.6 * a,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSpec {
    /// 1.0 leaves the level as it is, 0 is silent, up to [`MAX_VOLUME`].
    pub volume: f32,
    pub fade_in: Tick,
    pub fade_out: Tick,
    pub fx: AudioFx,
    /// How much of the effect is mixed in, 0..1.
    pub fx_amount: f32,
}

impl Default for AudioSpec {
    fn default() -> Self {
        AudioSpec {
            volume: 1.0,
            fade_in: Tick::ZERO,
            fade_out: Tick::ZERO,
            fx: AudioFx::None,
            fx_amount: 0.4,
        }
    }
}

impl AudioSpec {
    /// Values from a damaged file or a slider can never break the mixer.
    pub fn sanitized(mut self) -> Self {
        let fix = |v: f32, lo: f32, hi: f32, dflt: f32| {
            if v.is_finite() { v.clamp(lo, hi) } else { dflt }
        };
        self.volume = fix(self.volume, 0.0, MAX_VOLUME, 1.0);
        self.fx_amount = fix(self.fx_amount, 0.0, 1.0, 0.4);
        self.fade_in = self.fade_in.max(Tick::ZERO);
        self.fade_out = self.fade_out.max(Tick::ZERO);
        self
    }

    /// True when the mixer has nothing to do for this clip.
    pub fn is_neutral(&self) -> bool {
        self.volume == 1.0
            && self.fade_in == Tick::ZERO
            && self.fade_out == Tick::ZERO
            && self.fx == AudioFx::None
    }

    /// Apply only what changed between `old` and `new` (several clips edited together).
    pub fn apply_diff(&mut self, old: &AudioSpec, new: &AudioSpec) {
        macro_rules! carry {
            ($($f:ident),*) => { $( if old.$f != new.$f { self.$f = new.$f; } )* };
        }
        carry!(volume, fade_in, fade_out, fx, fx_amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_neutral_and_old_projects_load() {
        assert!(AudioSpec::default().is_neutral());
        let s: AudioSpec = serde_json::from_str("{}").unwrap();
        assert_eq!(s, AudioSpec::default());
    }

    #[test]
    fn sanitized_repairs_bad_numbers() {
        let s = AudioSpec {
            volume: f32::NAN,
            fx_amount: 9.0,
            fade_in: Tick(-5),
            ..Default::default()
        }
        .sanitized();
        assert_eq!(s.volume, 1.0);
        assert_eq!(s.fx_amount, 1.0);
        assert_eq!(s.fade_in, Tick::ZERO);
        assert_eq!(
            AudioSpec {
                volume: 7.0,
                ..Default::default()
            }
            .sanitized()
            .volume,
            MAX_VOLUME
        );
    }

    #[test]
    fn diff_carries_only_edited_fields() {
        let old = AudioSpec::default();
        let new = AudioSpec { volume: 0.5, ..old };
        let mut other = AudioSpec {
            fx: AudioFx::Hall,
            ..old
        };
        other.apply_diff(&old, &new);
        assert_eq!(other.volume, 0.5);
        assert_eq!(other.fx, AudioFx::Hall);
    }
}
