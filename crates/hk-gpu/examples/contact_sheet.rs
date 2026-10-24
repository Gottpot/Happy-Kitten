//! Renders every transition at 5 progress steps into one PNG so the shaders
//! can be judged without opening the app:
//!   cargo run -p hk-gpu --example contact_sheet -- out.png [first] [count]
//! `first` and `count` pick a slice of the transition list (default: all).
use hk_core::TransitionKind;
use hk_gpu::{Compositor, headless};

fn main() {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "contact_sheet.png".into());
    let (device, queue) = headless().expect("GPU");
    let comp = Compositor::new(device, queue);
    let (w, h) = (256u32, 144u32);
    let steps = [0.15f32, 0.35, 0.5, 0.65, 0.85];
    let a = comp.upload(&hk_media::sample_frame(0, 640, 360).pixels, 640, 360);
    let b = comp.upload(&hk_media::sample_frame(2, 640, 360).pixels, 640, 360);
    let out = comp.create_output(w, h);
    let first: usize = std::env::args()
        .nth(2)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let count: usize = std::env::args()
        .nth(3)
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);
    let kinds: Vec<TransitionKind> = TransitionKind::ALL
        .iter()
        .copied()
        .skip(first)
        .take(count)
        .collect();
    let mut sheet = image::RgbaImage::new(w * steps.len() as u32, h * kinds.len() as u32);
    for (row, kind) in kinds.iter().enumerate() {
        for (col, p) in steps.iter().enumerate() {
            comp.render_blend(&out, &a, &b, *kind, *p, *p * 1.0);
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
