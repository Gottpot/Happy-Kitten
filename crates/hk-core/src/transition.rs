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
    BurnDissolve,
    VoronoiShatter,
    DomainWarp,
    Raindrop,
    Fisheye,
    ZoomBlur,
    SpinBlur,
    Aberration,
    LightLeak,
    SakuraWipe,
    HeartIris,
    SpiralWipe,
    PawBloom,
    HexWipe,
    Kaleidoscope,
    Halftone,
    BayerDither,
    Tabby,
    Calico,
    PageCurl,
    Cube,
    Flip,
    Melt,
    Droste,
    Julia,
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

    /// Noise and cells (version 1.2).
    pub const NOISE: [TransitionKind; 3] = [
        TransitionKind::BurnDissolve,
        TransitionKind::VoronoiShatter,
        TransitionKind::DomainWarp,
    ];

    /// Waves and optics (version 1.2).
    pub const OPTICS: [TransitionKind; 6] = [
        TransitionKind::Raindrop,
        TransitionKind::Fisheye,
        TransitionKind::ZoomBlur,
        TransitionKind::SpinBlur,
        TransitionKind::Aberration,
        TransitionKind::LightLeak,
    ];

    /// Curves and geometry (version 1.2).
    pub const CURVES: [TransitionKind; 6] = [
        TransitionKind::SakuraWipe,
        TransitionKind::HeartIris,
        TransitionKind::SpiralWipe,
        TransitionKind::PawBloom,
        TransitionKind::HexWipe,
        TransitionKind::Kaleidoscope,
    ];

    /// Manga look (version 1.2).
    pub const MANGA: [TransitionKind; 4] = [
        TransitionKind::Halftone,
        TransitionKind::BayerDither,
        TransitionKind::Tabby,
        TransitionKind::Calico,
    ];

    /// 3d and physics (version 1.2).
    pub const PHYSICS: [TransitionKind; 6] = [
        TransitionKind::PageCurl,
        TransitionKind::Cube,
        TransitionKind::Flip,
        TransitionKind::Melt,
        TransitionKind::Droste,
        TransitionKind::Julia,
    ];

    pub const ALL: [TransitionKind; 44] = [
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
        TransitionKind::BurnDissolve,
        TransitionKind::VoronoiShatter,
        TransitionKind::DomainWarp,
        TransitionKind::Raindrop,
        TransitionKind::Fisheye,
        TransitionKind::ZoomBlur,
        TransitionKind::SpinBlur,
        TransitionKind::Aberration,
        TransitionKind::LightLeak,
        TransitionKind::SakuraWipe,
        TransitionKind::HeartIris,
        TransitionKind::SpiralWipe,
        TransitionKind::PawBloom,
        TransitionKind::HexWipe,
        TransitionKind::Kaleidoscope,
        TransitionKind::Halftone,
        TransitionKind::BayerDither,
        TransitionKind::Tabby,
        TransitionKind::Calico,
        TransitionKind::PageCurl,
        TransitionKind::Cube,
        TransitionKind::Flip,
        TransitionKind::Melt,
        TransitionKind::Droste,
        TransitionKind::Julia,
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
            TransitionKind::BurnDissolve => 19,
            TransitionKind::VoronoiShatter => 20,
            TransitionKind::DomainWarp => 21,
            TransitionKind::Raindrop => 22,
            TransitionKind::Fisheye => 23,
            TransitionKind::ZoomBlur => 24,
            TransitionKind::SpinBlur => 25,
            TransitionKind::Aberration => 26,
            TransitionKind::LightLeak => 27,
            TransitionKind::SakuraWipe => 28,
            TransitionKind::HeartIris => 29,
            TransitionKind::SpiralWipe => 30,
            TransitionKind::PawBloom => 31,
            TransitionKind::HexWipe => 32,
            TransitionKind::Kaleidoscope => 33,
            TransitionKind::Halftone => 34,
            TransitionKind::BayerDither => 35,
            TransitionKind::Tabby => 36,
            TransitionKind::Calico => 37,
            TransitionKind::PageCurl => 38,
            TransitionKind::Cube => 39,
            TransitionKind::Flip => 40,
            TransitionKind::Melt => 41,
            TransitionKind::Droste => 42,
            TransitionKind::Julia => 43,
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
            TransitionKind::BurnDissolve => "Papierbrand",
            TransitionKind::VoronoiShatter => "Scherben",
            TransitionKind::DomainWarp => "Tinte im Wasser",
            TransitionKind::Raindrop => "Wassertropfen",
            TransitionKind::Fisheye => "Fischauge",
            TransitionKind::ZoomBlur => "Zoom-Unschärfe",
            TransitionKind::SpinBlur => "Dreh-Unschärfe",
            TransitionKind::Aberration => "Farbsaum",
            TransitionKind::LightLeak => "Lichtleck",
            TransitionKind::SakuraWipe => "Sakura",
            TransitionKind::HeartIris => "Herz öffnen",
            TransitionKind::SpiralWipe => "Spirale",
            TransitionKind::PawBloom => "Pfotenblüte",
            TransitionKind::HexWipe => "Waben",
            TransitionKind::Kaleidoscope => "Kaleidoskop",
            TransitionKind::Halftone => "Rasterpunkte",
            TransitionKind::BayerDither => "Retro-Raster",
            TransitionKind::Tabby => "Tigerstreifen",
            TransitionKind::Calico => "Glückskatze",
            TransitionKind::PageCurl => "Seite umblättern",
            TransitionKind::Cube => "Würfel",
            TransitionKind::Flip => "Umklappen",
            TransitionKind::Melt => "Schmelzen",
            TransitionKind::Droste => "Droste-Zoom",
            TransitionKind::Julia => "Fraktal",
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
            TransitionKind::BurnDissolve => "Das Bild verglüht wie Papier mit leuchtender Kante",
            TransitionKind::VoronoiShatter => "Das Bild zerspringt in Zellen und kippt weg",
            TransitionKind::DomainWarp => "Das Bild verfließt wie Rauch oder Tinte",
            TransitionKind::Raindrop => "Ein Tropfen schickt Wellen übers Bild",
            TransitionKind::Fisheye => "Eine Linse zieht das Bild hinein",
            TransitionKind::ZoomBlur => "Rasanter Zoom mit Wischeffekt",
            TransitionKind::SpinBlur => "Das Bild dreht sich verschwommen um die Mitte",
            TransitionKind::Aberration => "Rot, Grün und Blau laufen auseinander",
            TransitionKind::LightLeak => "Ein Lichtstreifen überstrahlt den Schnitt",
            TransitionKind::SakuraWipe => "Eine Kirschblüte öffnet das neue Bild",
            TransitionKind::HeartIris => "Ein Herz geht auf und zeigt das neue Bild",
            TransitionKind::SpiralWipe => "Eine Spirale wischt das Bild weg",
            TransitionKind::PawBloom => "Pfotenballen wachsen wie ein Sonnenblumenkopf",
            TransitionKind::HexWipe => "Sechsecke klappen von der Mitte aus um",
            TransitionKind::Kaleidoscope => "Das Bild spiegelt sich in immer mehr Segmenten",
            TransitionKind::Halftone => "Manga-Rasterpunkte wachsen zusammen",
            TransitionKind::BayerDither => "Pixelmuster wie auf einem alten Bildschirm",
            TransitionKind::Tabby => "Verwackelte Streifen wischen das Bild weg",
            TransitionKind::Calico => "Fellflecken wie bei einer Glückskatze",
            TransitionKind::PageCurl => "Das Bild rollt sich wie ein Blatt Papier auf",
            TransitionKind::Cube => "Das Bild dreht sich um die Würfelkante",
            TransitionKind::Flip => "Das Bild wird wie eine Karte umgedreht",
            TransitionKind::Melt => "Das Bild tropft nach unten weg",
            TransitionKind::Droste => "Das Bild zoomt endlos in sich hinein",
            TransitionKind::Julia => "Ein Julia-Fraktal gibt das neue Bild frei",
        }
    }

    /// Progress at which the picture card in the pool shows the blend (some look empty at 0.5).
    pub fn card_progress(self) -> f32 {
        match self {
            TransitionKind::Flip => 0.3,
            TransitionKind::PageCurl | TransitionKind::Cube | TransitionKind::LightLeak => 0.4,
            TransitionKind::Aberration => 0.45,
            _ => 0.5,
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
            TransitionKind::Aberration | TransitionKind::ZoomBlur => Tick::from_secs(0.7),
            TransitionKind::SpinBlur | TransitionKind::BayerDither => Tick::from_secs(0.8),
            TransitionKind::LightLeak => Tick::from_secs(0.9),
            TransitionKind::PageCurl | TransitionKind::Julia => Tick::from_secs(1.2),
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
            TransitionKind::CAT.len()
                + TransitionKind::CLASSIC.len()
                + TransitionKind::NOISE.len()
                + TransitionKind::OPTICS.len()
                + TransitionKind::CURVES.len()
                + TransitionKind::MANGA.len()
                + TransitionKind::PHYSICS.len(),
            TransitionKind::ALL.len()
        );
        // the shader dispatches on ids 0..N without gaps
        let mut sorted: Vec<u32> = TransitionKind::ALL.iter().map(|k| k.shader_id()).collect();
        sorted.sort_unstable();
        assert!(sorted.iter().enumerate().all(|(i, id)| i as u32 == *id));
        assert!(TransitionKind::CAT.iter().all(|k| k.is_cat()));
        assert!(TransitionKind::CLASSIC.iter().all(|k| !k.is_cat()));
    }
}
