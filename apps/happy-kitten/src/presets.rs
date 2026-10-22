//! Built-in text looks and the colour swatches of the text editor.

use hk_core::{Align, TextFx, TextSpec, Tick};

/// Pastel swatches plus white and chocolate brown.
pub const PALETTE: [[u8; 3]; 12] = [
    [255, 255, 255],
    [255, 190, 207],
    [248, 150, 178],
    [255, 206, 158],
    [255, 232, 150],
    [178, 232, 208],
    [176, 214, 250],
    [216, 198, 246],
    [255, 120, 200],
    [120, 220, 255],
    [92, 56, 44],
    [30, 30, 36],
];

fn spec(f: impl FnOnce(&mut TextSpec)) -> TextSpec {
    let mut s = TextSpec::default();
    f(&mut s);
    s
}

fn secs(v: f64) -> Tick {
    Tick::from_secs(v)
}

/// (name, look). The words are the name, so a preset card shows what it will look like.
pub fn builtin() -> Vec<(&'static str, TextSpec)> {
    vec![
        (
            "Titel",
            spec(|s| {
                s.text = "Titel".into();
                s.size = 0.15;
                s.pos = [0.5, 0.42];
                s.fx_in = TextFx::Zoom;
                s.fx_in_dur = secs(0.8);
            }),
        ),
        ("Untertitel", TextSpec::subtitle()),
        (
            "Lyrics",
            spec(|s| {
                s.text = "Lyrics".into();
                s.size = 0.075;
                s.pos = [0.5, 0.82];
                s.color = [255, 249, 240, 255];
                s.outline = [248, 150, 178, 255];
                s.fx_in = TextFx::Pixelate;
                s.fx_in_dur = secs(0.4);
                s.fx_out = TextFx::Blur;
                s.fx_out_dur = secs(0.3);
            }),
        ),
        (
            "Neon-Pink",
            spec(|s| {
                s.text = "Neon".into();
                s.size = 0.13;
                s.color = [255, 120, 200, 255];
                s.outline = [255, 255, 255, 255];
                s.outline_width = 0.07;
                s.fx_in = TextFx::Glitch;
                s.fx_in_dur = secs(0.6);
                s.fx_out = TextFx::Glitch;
                s.fx_out_dur = secs(0.4);
            }),
        ),
        (
            "Comic",
            spec(|s| {
                s.text = "Miau!".into();
                s.font = "comic".into();
                s.size = 0.14;
                s.color = [255, 232, 150, 255];
                s.outline_width = 0.14;
                s.fx_in = TextFx::SlideUp;
            }),
        ),
        (
            "Schreibmaschine",
            spec(|s| {
                s.text = "Es war einmal …".into();
                s.font = "typewriter".into();
                s.size = 0.07;
                s.pos = [0.5, 0.8];
                s.outline_width = 0.0;
                s.shadow = false;
                s.background = [30, 24, 28, 170];
                s.fx_in = TextFx::Fade;
            }),
        ),
        (
            "Wölkchen",
            spec(|s| {
                s.text = "Wölkchen".into();
                s.size = 0.1;
                s.color = [92, 56, 44, 255];
                s.outline_width = 0.0;
                s.shadow = false;
                s.background = [255, 226, 233, 235];
                s.fx_in = TextFx::Blur;
                s.fx_out = TextFx::Blur;
            }),
        ),
        (
            "Strudel",
            spec(|s| {
                s.text = "Strudel".into();
                s.font = "marker".into();
                s.size = 0.15;
                s.color = [216, 198, 246, 255];
                s.fx_in = TextFx::Swirl;
                s.fx_in_dur = secs(1.0);
                s.fx_out = TextFx::Swirl;
                s.fx_out_dur = secs(1.0);
            }),
        ),
        (
            "Wellen",
            spec(|s| {
                s.text = "Wellen".into();
                s.font = "futura".into();
                s.size = 0.13;
                s.color = [120, 220, 255, 255];
                s.align = Align::Center;
                s.fx_in = TextFx::Wave;
                s.fx_out = TextFx::Wave;
            }),
        ),
    ]
}
