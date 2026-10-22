//! A 16:9 picture on a 9:16 canvas in the three fit modes: cargo run -p hk-gpu --example fit_sheet -- out.png
use hk_core::FitMode;
use hk_gpu::{Compositor, headless};

fn main() {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "fit_sheet.png".into());
    let (device, queue) = headless().expect("GPU");
    let comp = Compositor::new(device, queue);
    let (w, h) = (270u32, 480u32);
    let a = comp.upload(&hk_media::sample_frame(0, 1280, 720).pixels, 1280, 720);
    let out = comp.create_output(w, h);
    let mut sheet = image::RgbaImage::new(w * 3, h);
    for (i, mode) in FitMode::ALL.iter().enumerate() {
        comp.set_fit(*mode);
        comp.render_single(&out, &a);
        let px = comp.read_output(&out).expect("readback");
        image::imageops::replace(
            &mut sheet,
            &image::RgbaImage::from_raw(w, h, px).expect("size"),
            (i as u32 * w) as i64,
            0,
        );
    }
    sheet.save(&out_path).expect("save");
}
