//! Renders every text effect at five progress steps into one PNG:
//!   cargo run -p hk-gpu --example text_sheet -- out.png
use hk_core::{TextFx, TextSpec};
use hk_gpu::{Compositor, headless};

fn main() {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "text_sheet.png".into());
    let (device, queue) = headless().expect("GPU");
    let comp = Compositor::new(device, queue);
    let (w, h) = (320u32, 180u32);
    let bg = comp.upload(&hk_media::sample_frame(0, 640, 360).pixels, 640, 360);
    let spec = TextSpec {
        text: "Hallo Kätzchen".into(),
        size: 0.13,
        ..Default::default()
    };
    let r = hk_text::render(&spec, w, h).expect("font");
    let text = comp.upload(&r.pixels, w, h);
    let out = comp.create_output(w, h);
    let steps = [0.1f32, 0.3, 0.5, 0.75, 1.0];
    let mut sheet = image::RgbaImage::new(w * steps.len() as u32, h * TextFx::ALL.len() as u32);
    for (row, fx) in TextFx::ALL.iter().enumerate() {
        for (col, p) in steps.iter().enumerate() {
            comp.render_single(&out, &bg);
            comp.overlay_text(&out, &text, fx.shader_id(), *p, *p, spec.pos);
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
