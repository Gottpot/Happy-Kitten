//! Text clips: what to write and how it looks and animates.

use crate::time::Tick;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// (id, label). The rasteriser in `hk-text` maps ids to font files on the Mac.
pub const FONTS: [(&str, &str); 10] = [
    ("rounded", "Rund & niedlich"),
    ("comic", "Comic"),
    ("trebuchet", "Trebuchet"),
    ("arial", "Arial Fett"),
    ("georgia", "Georgia Fett"),
    ("impact", "Impact"),
    ("marker", "Handschrift"),
    ("chalk", "Kreide"),
    ("futura", "Futura"),
    ("typewriter", "Schreibmaschine"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Animation used to fade text in or out. The maths live in the GPU shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextFx {
    None,
    Fade,
    Pixelate,
    Swirl,
    Blur,
    SlideUp,
    Zoom,
    Wave,
    Glitch,
}

impl TextFx {
    pub const ALL: [TextFx; 9] = [
        TextFx::None,
        TextFx::Fade,
        TextFx::Pixelate,
        TextFx::Swirl,
        TextFx::Blur,
        TextFx::SlideUp,
        TextFx::Zoom,
        TextFx::Wave,
        TextFx::Glitch,
    ];

    pub fn shader_id(self) -> u32 {
        match self {
            TextFx::None => 0,
            TextFx::Fade => 1,
            TextFx::Pixelate => 2,
            TextFx::Swirl => 3,
            TextFx::Blur => 4,
            TextFx::SlideUp => 5,
            TextFx::Zoom => 6,
            TextFx::Wave => 7,
            TextFx::Glitch => 8,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TextFx::None => "Ohne",
            TextFx::Fade => "Überblenden",
            TextFx::Pixelate => "Verpixeln",
            TextFx::Swirl => "Verwirbeln",
            TextFx::Blur => "Weichzeichnen",
            TextFx::SlideUp => "Hochgleiten",
            TextFx::Zoom => "Heranzoomen",
            TextFx::Wave => "Wellen",
            TextFx::Glitch => "Glitch",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextSpec {
    pub text: String,
    pub font: String,
    /// Font size as a fraction of the frame height.
    pub size: f32,
    pub color: [u8; 4],
    pub outline: [u8; 4],
    /// Outline thickness as a fraction of the font size.
    pub outline_width: f32,
    pub shadow: bool,
    /// Box behind the text; alpha 0 means none.
    pub background: [u8; 4],
    pub align: Align,
    /// Centre of the text block, 0..1 of width and height.
    pub pos: [f32; 2],
    pub line_spacing: f32,
    pub fx_in: TextFx,
    pub fx_in_dur: Tick,
    pub fx_out: TextFx,
    pub fx_out_dur: Tick,
}

impl Default for TextSpec {
    fn default() -> Self {
        TextSpec {
            text: "Dein Text".into(),
            font: "rounded".into(),
            size: 0.09,
            color: [255, 255, 255, 255],
            outline: [92, 56, 44, 255],
            outline_width: 0.10,
            shadow: true,
            background: [0, 0, 0, 0],
            align: Align::Center,
            pos: [0.5, 0.5],
            line_spacing: 1.12,
            fx_in: TextFx::Fade,
            fx_in_dur: Tick(352_800_000),
            fx_out: TextFx::Fade,
            fx_out_dur: Tick(352_800_000),
        }
    }
}

impl TextSpec {
    /// Subtitle look: smaller, low in the frame.
    pub fn subtitle() -> Self {
        TextSpec {
            text: "Untertitel".into(),
            size: 0.055,
            pos: [0.5, 0.88],
            ..Default::default()
        }
    }

    /// Hash of everything that changes the rendered picture (not the animation).
    pub fn raster_key(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.text.hash(&mut h);
        self.font.hash(&mut h);
        self.size.to_bits().hash(&mut h);
        self.color.hash(&mut h);
        self.outline.hash(&mut h);
        self.outline_width.to_bits().hash(&mut h);
        self.shadow.hash(&mut h);
        self.background.hash(&mut h);
        self.align.hash(&mut h);
        self.pos[0].to_bits().hash(&mut h);
        self.pos[1].to_bits().hash(&mut h);
        self.line_spacing.to_bits().hash(&mut h);
        h.finish()
    }

    /// Apply only what changed between `old` and `new` (the words are never touched).
    /// Lets one edit change a field on many texts while their other differences stay.
    pub fn apply_diff(&mut self, old: &TextSpec, new: &TextSpec) {
        macro_rules! carry {
            ($($f:ident),*) => { $( if old.$f != new.$f { self.$f = new.$f.clone(); } )* };
        }
        carry!(
            font,
            size,
            color,
            outline,
            outline_width,
            shadow,
            background,
            align,
            pos,
            line_spacing,
            fx_in,
            fx_in_dur,
            fx_out,
            fx_out_dur
        );
    }

    /// Take over the look (not the words) of another text.
    pub fn copy_style_from(&mut self, other: &TextSpec) {
        let text = std::mem::take(&mut self.text);
        *self = other.clone();
        self.text = text;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_key_ignores_animation_but_not_look() {
        let a = TextSpec::default();
        let mut b = a.clone();
        b.fx_in = TextFx::Swirl;
        assert_eq!(a.raster_key(), b.raster_key());
        b.color = [255, 0, 0, 255];
        assert_ne!(a.raster_key(), b.raster_key());
    }

    #[test]
    fn copy_style_keeps_words() {
        let mut a = TextSpec {
            text: "Hallo".into(),
            ..Default::default()
        };
        let b = TextSpec {
            text: "anders".into(),
            size: 0.2,
            ..Default::default()
        };
        a.copy_style_from(&b);
        assert_eq!(a.text, "Hallo");
        assert_eq!(a.size, 0.2);
    }

    #[test]
    fn apply_diff_changes_only_the_edited_field() {
        let old = TextSpec::default();
        let mut new = old.clone();
        new.size = 0.2;
        let mut other = TextSpec {
            text: "anders".into(),
            color: [1, 2, 3, 255],
            ..Default::default()
        };
        other.apply_diff(&old, &new);
        assert_eq!(other.size, 0.2);
        assert_eq!(other.color, [1, 2, 3, 255]);
        assert_eq!(other.text, "anders");
    }
}
