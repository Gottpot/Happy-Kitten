use crate::time::Tick;
use serde::{Deserialize, Serialize};

/// All GPU blends. `shader_id` is the index used by `transitions.wgsl`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionKind {
    // cat blends
    GentlePawFade,
    ClawWipe,
    FurryBlur,
    PawSpin,
    KittyDash,
    MeowGlitch,
    // classic blends
    Crossfade,
    DipBlack,
    DipWhite,
    WipeRight,
    WipeDown,
    Push,
    Zoom,
    Iris,
    Clock,
    Pixelate,
    Whirl,
    BlurDissolve,
    BarnDoors,
}

impl TransitionKind {
    pub const CAT: [TransitionKind; 6] = [
        TransitionKind::ClawWipe,
        TransitionKind::FurryBlur,
        TransitionKind::PawSpin,
        TransitionKind::GentlePawFade,
        TransitionKind::KittyDash,
        TransitionKind::MeowGlitch,
    ];

    pub const CLASSIC: [TransitionKind; 13] = [
        TransitionKind::Crossfade,
        TransitionKind::DipBlack,
        TransitionKind::DipWhite,
        TransitionKind::WipeRight,
        TransitionKind::WipeDown,
        TransitionKind::Push,
        TransitionKind::Zoom,
        TransitionKind::Iris,
        TransitionKind::Clock,
        TransitionKind::BarnDoors,
        TransitionKind::Pixelate,
        TransitionKind::Whirl,
        TransitionKind::BlurDissolve,
    ];

    pub const ALL: [TransitionKind; 19] = [
        TransitionKind::ClawWipe,
        TransitionKind::FurryBlur,
        TransitionKind::PawSpin,
        TransitionKind::GentlePawFade,
        TransitionKind::KittyDash,
        TransitionKind::MeowGlitch,
        TransitionKind::Crossfade,
        TransitionKind::DipBlack,
        TransitionKind::DipWhite,
        TransitionKind::WipeRight,
        TransitionKind::WipeDown,
        TransitionKind::Push,
        TransitionKind::Zoom,
        TransitionKind::Iris,
        TransitionKind::Clock,
        TransitionKind::BarnDoors,
        TransitionKind::Pixelate,
        TransitionKind::Whirl,
        TransitionKind::BlurDissolve,
    ];

    pub fn is_cat(self) -> bool {
        Self::CAT.contains(&self)
    }

    pub fn shader_id(self) -> u32 {
        match self {
            TransitionKind::Crossfade => 0,
            TransitionKind::ClawWipe => 1,
            TransitionKind::FurryBlur => 2,
            TransitionKind::PawSpin => 3,
            TransitionKind::GentlePawFade => 4,
            TransitionKind::KittyDash => 5,
            TransitionKind::MeowGlitch => 6,
            TransitionKind::DipBlack => 7,
            TransitionKind::DipWhite => 8,
            TransitionKind::WipeRight => 9,
            TransitionKind::WipeDown => 10,
            TransitionKind::Push => 11,
            TransitionKind::Zoom => 12,
            TransitionKind::Iris => 13,
            TransitionKind::Clock => 14,
            TransitionKind::Pixelate => 15,
            TransitionKind::Whirl => 16,
            TransitionKind::BlurDissolve => 17,
            TransitionKind::BarnDoors => 18,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TransitionKind::Crossfade => "Überblenden",
            TransitionKind::ClawWipe => "Claw Wipe",
            TransitionKind::FurryBlur => "Furry Blur",
            TransitionKind::PawSpin => "Paw Spin",
            TransitionKind::GentlePawFade => "Gentle Paw Fade",
            TransitionKind::KittyDash => "Kitty Dash",
            TransitionKind::MeowGlitch => "Meow Glitch",
            TransitionKind::DipBlack => "Schwarzblende",
            TransitionKind::DipWhite => "Weißblende",
            TransitionKind::WipeRight => "Wischen →",
            TransitionKind::WipeDown => "Wischen ↓",
            TransitionKind::Push => "Schieben",
            TransitionKind::Zoom => "Zoom",
            TransitionKind::Iris => "Kreis öffnen",
            TransitionKind::Clock => "Uhrzeiger",
            TransitionKind::Pixelate => "Verpixeln",
            TransitionKind::Whirl => "Verwirbeln",
            TransitionKind::BlurDissolve => "Weichzeichner",
            TransitionKind::BarnDoors => "Flügeltür",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            TransitionKind::Crossfade => "Weiches Überblenden",
            TransitionKind::ClawWipe => "Krallen kratzen das Bild auf",
            TransitionKind::FurryBlur => "Flauschiger Fell-Nebel",
            TransitionKind::PawSpin => "Riesenpfote wirbelt herein",
            TransitionKind::GentlePawFade => "Pfotenabdrücke tapsen durchs Bild",
            TransitionKind::KittyDash => "Das Kätzchen flitzt durchs Bild",
            TransitionKind::MeowGlitch => "Pixelmiau mit Bildstörung",
            TransitionKind::DipBlack => "Über Schwarz zum nächsten Bild",
            TransitionKind::DipWhite => "Heller Blitz dazwischen",
            TransitionKind::WipeRight => "Weicher Wisch nach rechts",
            TransitionKind::WipeDown => "Weicher Wisch nach unten",
            TransitionKind::Push => "Das neue Bild schiebt das alte raus",
            TransitionKind::Zoom => "Hineinzoomen und überblenden",
            TransitionKind::Iris => "Ein Kreis öffnet das neue Bild",
            TransitionKind::Clock => "Ein Zeiger wischt rundherum",
            TransitionKind::Pixelate => "Das Bild zerfällt in Pixel",
            TransitionKind::Whirl => "Ein Strudel zieht das Bild um",
            TransitionKind::BlurDissolve => "Unscharf überblenden",
            TransitionKind::BarnDoors => "Das Bild öffnet sich wie eine Tür",
        }
    }

    pub fn default_duration(self) -> Tick {
        match self {
            TransitionKind::MeowGlitch => Tick::from_secs(0.6),
            TransitionKind::KittyDash => Tick::from_secs(0.8),
            TransitionKind::DipBlack | TransitionKind::DipWhite => Tick::from_secs(0.8),
            TransitionKind::Push
            | TransitionKind::WipeRight
            | TransitionKind::WipeDown
            | TransitionKind::BarnDoors => Tick::from_secs(0.7),
            _ => Tick::from_secs(1.0),
        }
    }
}

/// A blend between a clip and the clip right before it on the same track.
/// It is centred on the cut: half runs over the end of the left clip, half over
/// the start of the right one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub kind: TransitionKind,
    pub duration: Tick,
}

impl Transition {
    pub fn with_duration(kind: TransitionKind, duration: Tick) -> Self {
        Transition { kind, duration }
    }

    pub fn new(kind: TransitionKind) -> Self {
        Transition {
            kind,
            duration: kind.default_duration(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_ids_are_unique_and_lists_agree() {
        let mut ids: Vec<u32> = TransitionKind::ALL.iter().map(|k| k.shader_id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), TransitionKind::ALL.len());
        assert_eq!(
            TransitionKind::CAT.len() + TransitionKind::CLASSIC.len(),
            TransitionKind::ALL.len()
        );
        assert!(TransitionKind::CAT.iter().all(|k| k.is_cat()));
        assert!(TransitionKind::CLASSIC.iter().all(|k| !k.is_cat()));
    }
}
