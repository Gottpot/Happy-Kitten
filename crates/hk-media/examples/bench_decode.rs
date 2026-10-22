//! cargo run --release -p hk-media --example bench_decode -- file.mp4
fn main() {
    let path = std::env::args().nth(1).expect("file");
    let mut src = hk_media::VideoSource::open(std::path::Path::new(&path)).expect("open");
    let t0 = std::time::Instant::now();
    let n = 90;
    for i in 0..n {
        src.frame_at(i as f64 / 30.0, 1280).expect("frame");
    }
    let dt = t0.elapsed();
    println!(
        "{} sequential frames in {:?} = {:.1} ms/frame",
        n,
        dt,
        dt.as_secs_f64() * 1000.0 / n as f64
    );
    let t1 = std::time::Instant::now();
    for i in 0..10 {
        src.frame_at(3.0 - i as f64 * 0.25, 1280).expect("frame");
    }
    println!(
        "10 backwards seeks: {:.1} ms each",
        t1.elapsed().as_secs_f64() * 100.0
    );
}
