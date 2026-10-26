//! Overlay effects: mathematically drawn pictures (flames, ice, lightning, equalizers, glitch,
//! laser light, lens flares ...) that lie over the video like a graphic. An effect is a clip on a
//! video track, so its length is changed on the timeline like any other clip.
//!
//! The drawing itself happens on the GPU (`fx.wgsl`); this file only holds the settings and the
//! numbers an effect can react to.

use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// Number of frequency bands the equalizer effects get.
pub const BANDS: usize = 32;
/// Number of beat values (one every 50 ms into the past).
pub const BEAT_STEPS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FxKind {
    // frames
    FireFrame,
    IceFrame,
    LightningFrame,
    NeonFrame,
    // equalizers
    EqBars,
    EqCircle,
    EqWave,
    BeatRings,
    // glitch and cyberpunk
    Glitch,
    CyberGrid,
    LaserShow,
    // light
    LensFlare,
    LightLeak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxGroup {
    Frames,
    Equalizer,
    Cyber,
    Light,
}

impl FxGroup {
    pub const ALL: [FxGroup; 4] = [
        FxGroup::Frames,
        FxGroup::Equalizer,
        FxGroup::Cyber,
        FxGroup::Light,
    ];

    /// German name, also the translation key.
    pub fn name(self) -> &'static str {
        match self {
            FxGroup::Frames => "Umrandungen",
            FxGroup::Equalizer => "Equalizer",
            FxGroup::Cyber => "Glitch und Cyberpunk",
            FxGroup::Light => "Licht",
        }
    }
}

/// Which settings an effect has (the editor only shows these).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FxParams {
    pub width: bool,
    pub speed: bool,
    pub hue: bool,
    pub beat: bool,
    pub pos: bool,
    pub mirror: bool,
}

impl FxKind {
    pub const FRAMES: [FxKind; 4] = [
        FxKind::FireFrame,
        FxKind::IceFrame,
        FxKind::LightningFrame,
        FxKind::NeonFrame,
    ];
    pub const EQUALIZERS: [FxKind; 4] = [
        FxKind::EqBars,
        FxKind::EqCircle,
        FxKind::EqWave,
        FxKind::BeatRings,
    ];
    pub const CYBER: [FxKind; 3] = [FxKind::Glitch, FxKind::CyberGrid, FxKind::LaserShow];
    pub const LIGHT: [FxKind; 2] = [FxKind::LensFlare, FxKind::LightLeak];

    pub const ALL: [FxKind; 13] = [
        FxKind::FireFrame,
        FxKind::IceFrame,
        FxKind::LightningFrame,
        FxKind::NeonFrame,
        FxKind::EqBars,
        FxKind::EqCircle,
        FxKind::EqWave,
        FxKind::BeatRings,
        FxKind::Glitch,
        FxKind::CyberGrid,
        FxKind::LaserShow,
        FxKind::LensFlare,
        FxKind::LightLeak,
    ];

    pub fn group(self) -> FxGroup {
        match self {
            FxKind::FireFrame | FxKind::IceFrame | FxKind::LightningFrame | FxKind::NeonFrame => {
                FxGroup::Frames
            }
            FxKind::EqBars | FxKind::EqCircle | FxKind::EqWave | FxKind::BeatRings => {
                FxGroup::Equalizer
            }
            FxKind::Glitch | FxKind::CyberGrid | FxKind::LaserShow => FxGroup::Cyber,
            FxKind::LensFlare | FxKind::LightLeak => FxGroup::Light,
        }
    }

    pub fn of_group(g: FxGroup) -> &'static [FxKind] {
        match g {
            FxGroup::Frames => &Self::FRAMES,
            FxGroup::Equalizer => &Self::EQUALIZERS,
            FxGroup::Cyber => &Self::CYBER,
            FxGroup::Light => &Self::LIGHT,
        }
    }

    /// Index used by `fx.wgsl`.
    pub fn shader_id(self) -> u32 {
        match self {
            FxKind::FireFrame => 1,
            FxKind::IceFrame => 2,
            FxKind::LightningFrame => 3,
            FxKind::NeonFrame => 4,
            FxKind::EqBars => 5,
            FxKind::EqCircle => 6,
            FxKind::EqWave => 7,
            FxKind::BeatRings => 8,
            FxKind::Glitch => 9,
            FxKind::CyberGrid => 10,
            FxKind::LaserShow => 11,
            FxKind::LensFlare => 12,
            FxKind::LightLeak => 13,
        }
    }

    /// German name, also the translation key.
    pub fn name(self) -> &'static str {
        match self {
            FxKind::FireFrame => "Feuer-Rahmen",
            FxKind::IceFrame => "Eis-Rahmen",
            FxKind::LightningFrame => "Blitz-Rahmen",
            FxKind::NeonFrame => "Neon-Rahmen",
            FxKind::EqBars => "Balken-Equalizer",
            FxKind::EqCircle => "Kreis-Equalizer",
            FxKind::EqWave => "Wellen-Equalizer",
            FxKind::BeatRings => "Beat-Ringe",
            FxKind::Glitch => "Glitch",
            FxKind::CyberGrid => "Cyber-Raster",
            FxKind::LaserShow => "Laser-Show",
            FxKind::LensFlare => "Linsenreflex",
            FxKind::LightLeak => "Lichtleck",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            FxKind::FireFrame => "Flammen lecken am Bildrand",
            FxKind::IceFrame => "Frostkristalle wachsen vom Rand hinein",
            FxKind::LightningFrame => "Elektrische Blitze zucken am Rand",
            FxKind::NeonFrame => "Leuchtender Neonrahmen",
            FxKind::EqBars => "Leuchtende Balken tanzen zur Musik",
            FxKind::EqCircle => "Ein Ring aus Balken schlägt zur Musik aus",
            FxKind::EqWave => "Eine Klangwelle läuft übers Bild",
            FxKind::BeatRings => "Ringe wandern bei jedem Beat nach außen",
            FxKind::Glitch => "Bildstörung mit Farbversatz und Zeilen",
            FxKind::CyberGrid => "Neon-Gitter und Scanlinien wie im Cyberpunk",
            FxKind::LaserShow => "Laserstrahlen fächern durch den Nebel",
            FxKind::LensFlare => "Sonne mit Streifen und Linsenreflexen",
            FxKind::LightLeak => "Warme Lichtflecken wie auf altem Film",
        }
    }

    /// True when the effect is an audio visualiser and should work even with the beat at zero.
    pub fn is_visualiser(self) -> bool {
        matches!(
            self,
            FxKind::EqBars | FxKind::EqCircle | FxKind::EqWave | FxKind::BeatRings
        )
    }

    pub fn params(self) -> FxParams {
        let base = FxParams {
            width: true,
            speed: true,
            hue: true,
            beat: true,
            pos: false,
            mirror: false,
        };
        match self {
            FxKind::EqBars => FxParams {
                pos: true,
                mirror: true,
                ..base
            },
            FxKind::EqCircle => FxParams { pos: true, ..base },
            FxKind::EqWave => FxParams {
                pos: true,
                mirror: true,
                ..base
            },
            FxKind::BeatRings => FxParams { pos: true, ..base },
            FxKind::CyberGrid => FxParams { pos: true, ..base },
            FxKind::LaserShow => FxParams { pos: true, ..base },
            FxKind::LensFlare => FxParams { pos: true, ..base },
            _ => base,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FxSpec {
    pub kind: FxKind,
    /// 0..1: thickness of a frame, height of an equalizer, size of a flare.
    pub width: f32,
    /// 0..4, 1 is the natural speed.
    pub speed: f32,
    /// 0..2, 1 is the natural strength.
    pub intensity: f32,
    /// Colour shift in turns, -0.5..0.5 (0 keeps the effect's own colours).
    pub hue: f32,
    /// 0..1: how strongly the effect jumps with the bass of the music.
    pub beat: f32,
    /// Where the effect sits or comes from, 0..1 of width and height.
    pub pos: [f32; 2],
    /// Reflect the effect (equalizers).
    pub mirror: bool,
    /// Fade in at the start and out at the end of the clip.
    pub fade: Tick,
}

impl Default for FxSpec {
    fn default() -> Self {
        FxSpec::new(FxKind::FireFrame)
    }
}

impl FxSpec {
    /// Settings that look good at once, per effect.
    pub fn new(kind: FxKind) -> Self {
        let mut s = FxSpec {
            kind,
            width: 0.45,
            speed: 1.0,
            intensity: 1.0,
            hue: 0.0,
            beat: 0.0,
            pos: [0.5, 0.5],
            mirror: false,
            fade: Tick::from_secs(0.4),
        };
        match kind {
            FxKind::EqBars => {
                s.width = 0.4;
                s.beat = 1.0;
                s.pos = [0.5, 0.92];
            }
            FxKind::EqCircle => {
                s.width = 0.45;
                s.beat = 1.0;
            }
            FxKind::EqWave => {
                s.width = 0.4;
                s.beat = 1.0;
                s.pos = [0.5, 0.8];
            }
            FxKind::BeatRings => {
                s.width = 0.5;
                s.beat = 1.0;
            }
            FxKind::Glitch => s.width = 0.5,
            FxKind::CyberGrid => s.pos = [0.5, 0.55],
            FxKind::LaserShow => {
                s.pos = [0.5, 1.05];
                s.width = 0.4;
            }
            FxKind::LensFlare => {
                s.pos = [0.78, 0.22];
                s.width = 0.5;
            }
            FxKind::LightLeak => s.width = 0.55,
            _ => {}
        }
        s
    }

    /// Values from a damaged file or a slider can never break the shader.
    pub fn sanitized(mut self) -> Self {
        let fix = |v: f32, lo: f32, hi: f32, dflt: f32| {
            if v.is_finite() { v.clamp(lo, hi) } else { dflt }
        };
        self.width = fix(self.width, 0.0, 1.0, 0.45);
        self.speed = fix(self.speed, 0.0, 4.0, 1.0);
        self.intensity = fix(self.intensity, 0.0, 2.0, 1.0);
        self.hue = fix(self.hue, -0.5, 0.5, 0.0);
        self.beat = fix(self.beat, 0.0, 1.0, 0.0);
        self.pos = [
            fix(self.pos[0], -0.2, 1.2, 0.5),
            fix(self.pos[1], -0.2, 1.2, 0.5),
        ];
        self.fade = self.fade.max(Tick::ZERO);
        self
    }

    /// True when drawing the effect needs the sound analysed.
    pub fn needs_audio(&self) -> bool {
        self.kind.is_visualiser() || self.beat > 0.0
    }

    /// Apply only what changed between `old` and `new` (several clips edited together).
    pub fn apply_diff(&mut self, old: &FxSpec, new: &FxSpec) {
        macro_rules! carry {
            ($($f:ident),*) => { $( if old.$f != new.$f { self.$f = new.$f; } )* };
        }
        carry!(width, speed, intensity, hue, beat, pos, mirror, fade);
    }
}

/// What the sound is doing right now: the numbers the equalizers and the beat setting react to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioLevels {
    /// Loudness of [`BANDS`] frequency bands, low to high, 0..1.
    pub bands: [f32; BANDS],
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
    /// How strongly a beat hit 0, 50, 100 ... 350 milliseconds ago, 0..1.
    pub beat: [f32; BEAT_STEPS],
}

impl AudioLevels {
    pub const SILENT: AudioLevels = AudioLevels {
        bands: [0.0; BANDS],
        bass: 0.0,
        mid: 0.0,
        treble: 0.0,
        beat: [0.0; BEAT_STEPS],
    };

    /// A made-up, moving spectrum for the cards in the effects tab and for tests.
    pub fn demo(t: f32) -> AudioLevels {
        let mut l = AudioLevels::SILENT;
        let pulse = |t: f32| {
            let ph = (t * 2.0).fract();
            (1.0 - ph).powf(3.0)
        };
        for (i, b) in l.bands.iter_mut().enumerate() {
            let x = i as f32 / (BANDS - 1) as f32;
            let tilt = 1.0 - 0.55 * x;
            let wob = 0.5 + 0.5 * (t * (2.0 + x * 5.0) + i as f32 * 1.7).sin();
            *b = (tilt * (0.25 + 0.75 * wob) * (0.55 + 0.45 * pulse(t))).clamp(0.0, 1.0);
        }
        l.bass = l.bands[..6].iter().sum::<f32>() / 6.0;
        l.mid = l.bands[6..20].iter().sum::<f32>() / 14.0;
        l.treble = l.bands[20..].iter().sum::<f32>() / 12.0;
        for (k, b) in l.beat.iter_mut().enumerate() {
            *b = pulse(t - 0.05 * k as f32);
        }
        l
    }

    /// Numbers that cannot upset the shader.
    pub fn sanitized(mut self) -> Self {
        let fix = |v: f32| {
            if v.is_finite() {
                v.clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        self.bands.iter_mut().for_each(|b| *b = fix(*b));
        self.beat.iter_mut().for_each(|b| *b = fix(*b));
        self.bass = fix(self.bass);
        self.mid = fix(self.mid);
        self.treble = fix(self.treble);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_ids_are_complete_and_groups_cover_everything() {
        let mut ids: Vec<u32> = FxKind::ALL.iter().map(|k| k.shader_id()).collect();
        ids.sort_unstable();
        assert!(ids.iter().enumerate().all(|(i, id)| i as u32 + 1 == *id));
        let grouped: usize = FxGroup::ALL
            .iter()
            .map(|g| FxKind::of_group(*g).len())
            .sum();
        assert_eq!(grouped, FxKind::ALL.len());
        for k in FxKind::ALL {
            assert!(FxKind::of_group(k.group()).contains(&k));
        }
    }

    #[test]
    fn defaults_are_valid_and_old_projects_load() {
        for k in FxKind::ALL {
            let s = FxSpec::new(k);
            assert_eq!(s, s.sanitized(), "{k:?}");
        }
        let s: FxSpec = serde_json::from_str("{}").unwrap();
        assert_eq!(s.kind, FxKind::FireFrame);
    }

    #[test]
    fn sanitized_repairs_bad_numbers() {
        let s = FxSpec {
            width: f32::NAN,
            speed: 99.0,
            hue: -3.0,
            pos: [f32::INFINITY, 0.5],
            ..FxSpec::new(FxKind::Glitch)
        }
        .sanitized();
        assert_eq!((s.width, s.speed, s.hue, s.pos[0]), (0.45, 4.0, -0.5, 0.5));
    }

    #[test]
    fn only_visualisers_and_beat_reactive_effects_need_the_sound() {
        assert!(FxSpec::new(FxKind::EqBars).needs_audio());
        assert!(!FxSpec::new(FxKind::FireFrame).needs_audio());
        assert!(
            FxSpec {
                beat: 0.5,
                ..FxSpec::new(FxKind::FireFrame)
            }
            .needs_audio()
        );
    }

    #[test]
    fn demo_levels_move_and_stay_in_range() {
        let a = AudioLevels::demo(0.3);
        let b = AudioLevels::demo(0.9);
        assert_ne!(a, b);
        assert!(
            a.bands
                .iter()
                .chain(&b.bands)
                .all(|v| (0.0..=1.0).contains(v))
        );
    }
}
