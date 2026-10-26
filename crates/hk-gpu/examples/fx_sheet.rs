//! Renders every overlay effect at four moments into one PNG, over a sample scene and with a made-up
//! spectrum, so the shaders can be judged without opening the app:
//!   cargo run -p hk-gpu --example fx_sheet -- out.png [first] [count]
use hk_core::{AudioLevels, FxKind, FxSpec};
use hk_gpu::{Compositor, headless};

fn main() {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fx_sheet.png".into());
    let first: usize = std::env::args()
        .nth(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let count: usize = std::env::args()
        .nth(3)
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    let (device, queue) = headless().expect("GPU");
    let comp = Compositor::new(device, queue);
    let (w, h) = (320u32, 180u32);
    let times = [0.4f32, 1.1, 1.9, 2.7];
    let bg = comp.upload(&hk_media::sample_frame(2, 640, 360).pixels, 640, 360);
    let out = comp.create_output(w, h);
    let kinds: Vec<FxKind> = FxKind::ALL
        .iter()
        .copied()
        .skip(first)
        .take(count)
        .collect();
    let mut sheet = image::RgbaImage::new(w * times.len() as u32, h * kinds.len() as u32);
    for (row, kind) in kinds.iter().enumerate() {
        for (col, t) in times.iter().enumerate() {
            comp.render_single(&out, &bg);
            comp.overlay_fx(
                &out,
                &FxSpec::new(*kind),
                *t,
                1.0,
                0.0,
                &AudioLevels::demo(*t),
            );
            let px = comp.read_output(&out).expect("readback");
            let tile = image::RgbaImage::from_raw(w, h, px).expect("size");
            image::imageops::replace(
                &mut sheet,
                &tile,
                (col as u32 * w) as i64,
                (row as u32 * h) as i64,
            );
        }
    }
    sheet.save(&out_path).expect("save");
    println!("wrote {out_path}");
}
