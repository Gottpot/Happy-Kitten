//! One transition frame at a chosen size: single <kind 0..6> <progress> <w> <h> <out.png>
use hk_core::TransitionKind;
use hk_gpu::{Compositor, headless};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: &str| a.get(i).cloned().unwrap_or_else(|| d.to_string());
    let kind_id: usize = arg(1, "4").parse().unwrap_or(4);
    let p: f32 = arg(2, "0.5").parse().unwrap_or(0.5);
    let (w, h): (u32, u32) = (
        arg(3, "960").parse().unwrap_or(960),
        arg(4, "540").parse().unwrap_or(540),
    );
    let kind = TransitionKind::ALL
        .iter()
        .copied()
        .find(|k| k.shader_id() as usize == kind_id)
        .unwrap_or(TransitionKind::Crossfade);
    let (device, queue) = headless().expect("GPU");
    let comp = Compositor::new(device, queue);
    let fa = comp.upload(&hk_media::sample_frame(0, 1280, 720).pixels, 1280, 720);
    let fb = comp.upload(&hk_media::sample_frame(2, 1280, 720).pixels, 1280, 720);
    let out = comp.create_output(w, h);
    comp.render_blend(&out, &fa, &fb, kind, p, p);
    let px = comp.read_output(&out).expect("readback");
    image::RgbaImage::from_raw(w, h, px)
        .expect("size")
        .save(arg(5, "single.png"))
        .expect("save");
}
