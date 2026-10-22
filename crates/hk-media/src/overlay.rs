//! Lays a picture (a PNG with transparency, a logo, a button) onto a transparent
//! frame-sized canvas with its scale, position, rotation and opacity. The result is
//! premultiplied RGBA, ready for the GPU text/graphic overlay pass.

use crate::Rgba;
use hk_core::OverlaySpec;

fn premultiply(img: &Rgba) -> Vec<u8> {
    let mut out = img.pixels.clone();
    for px in out.as_chunks_mut::<4>().0 {
        let a = px[3] as u32;
        for c in px.iter_mut().take(3) {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
    out
}

/// Bilinear sample of a premultiplied image; texels outside count as transparent.
fn sample(px: &[u8], w: usize, h: usize, u: f32, v: f32) -> [f32; 4] {
    let (x0, y0) = (u.floor(), v.floor());
    let (fx, fy) = (u - x0, v - y0);
    let at = |x: i64, y: i64| -> [f32; 4] {
        if x < 0 || y < 0 || x as usize >= w || y as usize >= h {
            return [0.0; 4];
        }
        let i = (y as usize * w + x as usize) * 4;
        [
            px[i] as f32,
            px[i + 1] as f32,
            px[i + 2] as f32,
            px[i + 3] as f32,
        ]
    };
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (a, b, c, d) = (
        at(x0, y0),
        at(x0 + 1, y0),
        at(x0, y0 + 1),
        at(x0 + 1, y0 + 1),
    );
    let mut out = [0.0; 4];
    for k in 0..4 {
        let top = a[k] + (b[k] - a[k]) * fx;
        let bot = c[k] + (d[k] - c[k]) * fx;
        out[k] = top + (bot - top) * fy;
    }
    out
}

/// `img` has straight (non-premultiplied) alpha. The result is `width * height` premultiplied RGBA.
pub fn compose_overlay(img: &Rgba, spec: &OverlaySpec, width: u32, height: u32) -> Rgba {
    let (w, h) = (width.max(1) as usize, height.max(1) as usize);
    let mut pixels = vec![0u8; w * h * 4];
    let (iw, ih) = (img.width as usize, img.height as usize);
    if iw == 0 || ih == 0 || img.pixels.len() < iw * ih * 4 {
        return Rgba {
            width,
            height,
            pixels,
        };
    }
    let scale = if spec.scale.is_finite() {
        spec.scale.clamp(0.02, 4.0)
    } else {
        0.3
    };
    let target_w = (scale * w as f32).max(1.0);
    let target_h = target_w * ih as f32 / iw as f32;
    let mut src = premultiply(img);
    let (mut sw, mut sh) = (iw, ih);
    // much bigger than needed: shrink first (premultiplied, so edges stay clean) to avoid aliasing
    if iw as f32 > target_w * 2.0 {
        let nw = ((target_w * 1.5).ceil() as u32).max(1);
        let nh = ((target_h * 1.5).ceil() as u32).max(1);
        if let Some(buf) = image::RgbaImage::from_raw(iw as u32, ih as u32, src.clone()) {
            let small =
                image::imageops::resize(&buf, nw, nh, image::imageops::FilterType::Triangle);
            (sw, sh) = (small.width() as usize, small.height() as usize);
            src = small.into_raw();
        }
    }
    let theta = if spec.rotation.is_finite() {
        spec.rotation.to_radians()
    } else {
        0.0
    };
    let (sin, cos) = theta.sin_cos();
    let c = [
        spec.pos[0].clamp(-1.0, 2.0) * w as f32,
        spec.pos[1].clamp(-1.0, 2.0) * h as f32,
    ];
    let (hw, hh) = (target_w / 2.0, target_h / 2.0);
    let bw = cos.abs() * hw + sin.abs() * hh + 2.0;
    let bh = sin.abs() * hw + cos.abs() * hh + 2.0;
    let opacity = if spec.opacity.is_finite() {
        spec.opacity.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let (y0, y1) = (
        (c[1] - bh).floor().max(0.0) as usize,
        ((c[1] + bh).ceil().max(0.0) as usize).min(h),
    );
    let (x0, x1) = (
        (c[0] - bw).floor().max(0.0) as usize,
        ((c[0] + bw).ceil().max(0.0) as usize).min(w),
    );
    for y in y0..y1 {
        for x in x0..x1 {
            let (px, py) = (x as f32 + 0.5 - c[0], y as f32 + 0.5 - c[1]);
            let lx = px * cos + py * sin;
            let ly = -px * sin + py * cos;
            let u = (lx + hw) / target_w * sw as f32 - 0.5;
            let v = (ly + hh) / target_h * sh as f32 - 0.5;
            if u < -1.0 || v < -1.0 || u > sw as f32 || v > sh as f32 {
                continue;
            }
            let s = sample(&src, sw, sh, u, v);
            let o = (y * w + x) * 4;
            for k in 0..4 {
                pixels[o + k] = (s[k] * opacity).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    Rgba {
        width,
        height,
        pixels,
    }
}

/// True when the image has see-through pixels (only formats that can carry alpha are decoded).
pub fn image_has_alpha(path: &std::path::Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if !matches!(ext.as_str(), "png" | "webp" | "gif" | "tif" | "tiff") {
        return false;
    }
    match image::open(path) {
        Ok(img) if img.color().has_alpha() => img.to_rgba8().pixels().any(|p| p.0[3] < 255),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, px: [u8; 4]) -> Rgba {
        Rgba {
            width: w,
            height: h,
            pixels: px
                .iter()
                .copied()
                .cycle()
                .take((w * h * 4) as usize)
                .collect(),
        }
    }

    fn at(r: &Rgba, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * r.width + x) * 4) as usize;
        [
            r.pixels[i],
            r.pixels[i + 1],
            r.pixels[i + 2],
            r.pixels[i + 3],
        ]
    }

    #[test]
    fn opaque_picture_lands_in_the_middle_and_leaves_the_rest_clear() {
        let spec = OverlaySpec {
            scale: 0.5,
            ..Default::default()
        };
        let out = compose_overlay(&solid(10, 10, [255, 0, 0, 255]), &spec, 100, 100);
        assert_eq!(at(&out, 50, 50), [255, 0, 0, 255]);
        assert_eq!(at(&out, 5, 5)[3], 0);
        assert_eq!(at(&out, 90, 50)[3], 0);
    }

    #[test]
    fn transparency_is_kept_and_premultiplied() {
        let out = compose_overlay(
            &solid(8, 8, [255, 0, 0, 128]),
            &OverlaySpec {
                scale: 0.5,
                ..Default::default()
            },
            100,
            100,
        );
        let p = at(&out, 50, 50);
        assert!((p[3] as i32 - 128).abs() <= 2, "alpha {p:?}");
        assert!((p[0] as i32 - 128).abs() <= 2, "premultiplied red {p:?}");
        // a fully clear picture draws nothing
        let clear = compose_overlay(
            &solid(8, 8, [255, 0, 0, 0]),
            &OverlaySpec::default(),
            100,
            100,
        );
        assert!(clear.pixels.iter().all(|b| *b == 0));
    }

    #[test]
    fn position_rotation_and_opacity() {
        // 2:1 picture, 40 px wide, 20 px high; turned 90 degrees it is 20 wide and 40 high
        let spec = OverlaySpec {
            scale: 0.4,
            rotation: 90.0,
            ..Default::default()
        };
        let out = compose_overlay(&solid(20, 10, [0, 255, 0, 255]), &spec, 100, 100);
        assert_eq!(at(&out, 50, 66)[3], 255, "tall now");
        assert_eq!(at(&out, 66, 50)[3], 0, "no longer wide");
        let moved = compose_overlay(
            &solid(10, 10, [0, 0, 255, 255]),
            &OverlaySpec {
                scale: 0.2,
                pos: [0.8, 0.2],
                ..Default::default()
            },
            100,
            100,
        );
        assert_eq!(at(&moved, 80, 20)[3], 255);
        assert_eq!(at(&moved, 20, 80)[3], 0);
        let faint = compose_overlay(
            &solid(10, 10, [255, 255, 255, 255]),
            &OverlaySpec {
                opacity: 0.5,
                ..Default::default()
            },
            100,
            100,
        );
        assert!((at(&faint, 50, 50)[3] as i32 - 128).abs() <= 2);
    }

    #[test]
    fn huge_or_odd_values_do_not_panic() {
        let img = solid(300, 200, [1, 2, 3, 255]);
        for spec in [
            OverlaySpec {
                scale: f32::NAN,
                rotation: f32::INFINITY,
                opacity: f32::NAN,
                pos: [f32::NAN, 9.0],
                ..Default::default()
            },
            OverlaySpec {
                scale: 100.0,
                ..Default::default()
            },
            OverlaySpec {
                scale: 0.0,
                ..Default::default()
            },
        ] {
            let out = compose_overlay(&img, &spec, 64, 64);
            assert_eq!(out.pixels.len(), 64 * 64 * 4);
        }
        let empty = Rgba {
            width: 0,
            height: 0,
            pixels: vec![],
        };
        assert_eq!(
            compose_overlay(&empty, &OverlaySpec::default(), 8, 8)
                .pixels
                .len(),
            256
        );
    }

    #[test]
    fn alpha_detection_reads_the_file() {
        let dir = std::env::temp_dir().join(format!("hk-alpha-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let clear = dir.join("clear.png");
        let opaque = dir.join("opaque.png");
        image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 100]))
            .save(&clear)
            .unwrap();
        image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]))
            .save(&opaque)
            .unwrap();
        assert!(image_has_alpha(&clear));
        assert!(!image_has_alpha(&opaque));
        assert!(!image_has_alpha(&dir.join("missing.png")));
        std::fs::remove_dir_all(dir).ok();
    }
}
