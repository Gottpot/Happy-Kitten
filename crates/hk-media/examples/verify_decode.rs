//! Sequential and single-frame decoding must agree: cargo run --release -p hk-media --example verify_decode -- file.mp4
fn main() {
    let path = std::env::args().nth(1).expect("file");
    let p = std::path::Path::new(&path);
    let still = hk_media::VideoReader::open(p).expect("open");
    let mut src = hk_media::VideoSource::open(p).expect("open");
    let mut worst = 0.0f64;
    for i in 0..45 {
        let t = i as f64 / 30.0 + 0.5;
        let a = src.frame_at(t, 640).expect("seq");
        let b = still.frame_at(t, 640).expect("still");
        assert_eq!((a.width, a.height), (b.width, b.height), "size at {t}");
        let diff: f64 = a
            .pixels
            .iter()
            .zip(&b.pixels)
            .map(|(x, y)| (*x as f64 - *y as f64).abs())
            .sum::<f64>()
            / a.pixels.len() as f64;
        worst = worst.max(diff);
    }
    println!("worst mean abs difference over 45 frames: {worst:.2} / 255");
}
