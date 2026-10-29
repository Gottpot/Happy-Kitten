//! Four procedurally painted demo scenes so a fresh install has something to
//! cut and blend. They are drawn per pixel from simple shapes, no asset files.

use crate::Rgba;

pub const SAMPLE_COUNT: u8 = 4;

pub fn sample_name(index: u8) -> &'static str {
    match index {
        0 => "Sonnenwiese",
        1 => "Wollknäuel",
        2 => "Abendkatze",
        _ => "Minzgarten",
    }
}

type V3 = [f32; 3];

fn mix(a: V3, b: V3, t: f32) -> V3 {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Anti-aliased coverage of a signed distance (negative = inside).
fn cover(d: f32, px: f32) -> f32 {
    (0.5 - d / px).clamp(0.0, 1.0)
}

fn circle(x: f32, y: f32, cx: f32, cy: f32, r: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

fn hash(x: f32, y: f32) -> f32 {
    ((x * 127.1 + y * 311.7).sin() * 43758.547).fract().abs()
}

pub fn sample_frame(index: u8, width: u32, height: u32) -> Rgba {
    let (w, h) = (width.max(1), height.max(1));
    let aspect = w as f32 / h as f32;
    let px = 1.5 / h as f32;
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for j in 0..h {
        for i in 0..w {
            // x in 0..aspect, y in 0..1
            let x = (i as f32 + 0.5) / h as f32;
            let y = (j as f32 + 0.5) / h as f32;
            let c = match index {
                0 => meadow(x, y, aspect, px),
                1 => yarn(x, y, aspect, px),
                2 => evening(x, y, aspect, px),
                _ => mint(x, y, aspect, px),
            };
            pixels.extend_from_slice(&[
                (c[0] * 255.0) as u8,
                (c[1] * 255.0) as u8,
                (c[2] * 255.0) as u8,
                255,
            ]);
        }
    }
    Rgba {
        width: w,
        height: h,
        pixels,
    }
}

fn meadow(x: f32, y: f32, aspect: f32, px: f32) -> V3 {
    let mut c = mix([0.50, 0.78, 1.0], [0.88, 0.96, 1.0], y / 0.65);
    let sun = circle(x, y, aspect * 0.78, 0.24, 0.11);
    c = mix(c, [1.0, 0.98, 0.78], (-sun * 6.0 + 0.5).clamp(0.0, 0.7));
    c = mix(c, [1.0, 0.89, 0.40], cover(sun, px));
    // clouds
    for (cx, cy, r) in [
        (0.30, 0.20, 0.06),
        (0.37, 0.18, 0.08),
        (0.45, 0.21, 0.06),
        (1.05, 0.12, 0.05),
        (1.11, 0.11, 0.07),
    ] {
        c = mix(
            c,
            [1.0, 1.0, 1.0],
            cover(circle(x, y, cx * aspect / 1.78, cy, r), px) * 0.9,
        );
    }
    let far = 0.62 + 0.05 * (x * 3.0).sin();
    c = mix(c, [0.62, 0.84, 0.55], cover(far - y, px));
    let near = 0.78 + 0.06 * (x * 2.2 + 1.0).sin();
    c = mix(c, [0.45, 0.74, 0.42], cover(near - y, px));
    // flowers
    let (gx, gy) = (x * 9.0, y * 9.0);
    let (cx, cy) = (gx.floor(), gy.floor());
    if y > near + 0.03 && hash(cx, cy) > 0.55 {
        let fx = cx + 0.5 + (hash(cx, 1.0) - 0.5) * 0.5;
        let fy = cy + 0.5 + (hash(1.0, cy) - 0.5) * 0.5;
        let d = ((gx - fx).powi(2) + (gy - fy).powi(2)).sqrt() - 0.16;
        let col = if hash(cx, cy) > 0.8 {
            [1.0, 0.72, 0.82]
        } else {
            [1.0, 0.95, 0.7]
        };
        c = mix(c, col, cover(d / 9.0, px));
    }
    c
}

fn yarn(x: f32, y: f32, aspect: f32, px: f32) -> V3 {
    let (cx, cy) = (aspect * 0.5, 0.5);
    let r = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    let mut c = mix([1.0, 0.86, 0.90], [1.0, 0.68, 0.78], r / 0.9);
    // soft shadow
    c = mix(
        c,
        [0.85, 0.5, 0.6],
        cover(
            circle(x, y, cx + 0.02, cy + 0.30, 0.0)
                + ((x - cx).powi(2) / 0.09 + (y - cy - 0.33).powi(2) / 0.006).sqrt() * 0.2
                - 0.1,
            0.15,
        ) * 0.4,
    );
    let ball = circle(x, y, cx, cy, 0.30);
    let stripes = (((x - cx) * 14.0 + (y - cy) * 22.0).sin() * 0.5 + 0.5).powi(3);
    let ball_col = mix([0.96, 0.48, 0.62], [1.0, 0.72, 0.8], stripes);
    let shade = mix([1.0, 1.0, 1.0], [0.82, 0.62, 0.7], (r / 0.3).powi(3));
    c = mix(
        c,
        [
            ball_col[0] * shade[0],
            ball_col[1] * shade[1],
            ball_col[2] * shade[2],
        ],
        cover(ball, px),
    );
    // trailing thread
    let tx = x - cx;
    let ty = y - (cy + 0.3);
    let curve = ty - 0.12 * (tx * 5.0).sin() * (tx > 0.0) as i32 as f32 - 0.02;
    if tx > 0.0 && tx < aspect * 0.45 {
        c = mix(c, [0.94, 0.45, 0.58], cover(curve.abs() - 0.008, px));
    }
    c
}

fn evening(x: f32, y: f32, aspect: f32, px: f32) -> V3 {
    let mut c = mix([0.42, 0.34, 0.66], [1.0, 0.66, 0.50], y / 0.85);
    if hash((x * 90.0).floor(), (y * 90.0).floor()) > 0.985 && y < 0.55 {
        c = mix(c, [1.0, 1.0, 0.92], 0.9);
    }
    let moon = circle(x, y, aspect * 0.2, 0.22, 0.09);
    let bite = circle(x, y, aspect * 0.2 + 0.05, 0.20, 0.08);
    c = mix(c, [1.0, 0.97, 0.85], cover(moon.max(-bite), px));
    c = mix(
        c,
        [0.30, 0.22, 0.42],
        cover((0.80 + 0.03 * (x * 4.0).sin()) - y, px),
    );
    // cat silhouette on the hill
    let (cx, cy) = (aspect * 0.62, 0.70);
    let body = (((x - cx) / 0.15).powi(2) + ((y - cy - 0.02) / 0.11).powi(2)).sqrt() - 1.0;
    let head = circle(x, y, cx - 0.12, cy - 0.10, 0.065);
    let ear = |ex: f32| {
        let dx = (x - (cx - 0.12 + ex)).abs();
        (dx * 1.6 + (cy - 0.19 - y).abs() * 0.0).max(0.0) + (y - (cy - 0.13)).max(0.0) * 0.0
            - (0.04 - (cy - 0.13 - y).max(-0.1)) * 0.0
            + (dx * 1.4 - (y - (cy - 0.21)).clamp(0.0, 0.08) * 0.8)
    };
    let tail = circle(x, y, cx + 0.19, cy - 0.04, 0.012)
        .min(circle(x, y, cx + 0.22, cy - 0.10, 0.012))
        .min(circle(x, y, cx + 0.2, cy - 0.17, 0.012));
    let mut sil = body.min(head).min(tail - 0.0);
    sil = sil.min(ear(-0.035).max((cy - 0.205) - y).max(y - (cy - 0.14)) - 0.0);
    sil = sil.min(ear(0.035).max((cy - 0.205) - y).max(y - (cy - 0.14)) - 0.0);
    c = mix(c, [0.16, 0.11, 0.24], cover(sil, px));
    let eye =
        circle(x, y, cx - 0.14, cy - 0.105, 0.008).min(circle(x, y, cx - 0.10, cy - 0.105, 0.008));
    c = mix(c, [1.0, 0.95, 0.5], cover(eye, px));
    c
}

fn mint(x: f32, y: f32, aspect: f32, px: f32) -> V3 {
    let mut c = mix([0.74, 0.96, 0.88], [0.62, 0.90, 0.82], x / aspect);
    // paw grid
    let s = 0.22;
    let gy = (y / s).floor();
    let gx = ((x / s) + 0.5 * (gy as i32 % 2) as f32).floor();
    let fx = ((x / s) + 0.5 * (gy as i32 % 2) as f32).fract() - 0.5;
    let fy = (y / s).fract() - 0.5;
    let pad = ((fx / 0.17).powi(2) + ((fy - 0.1) / 0.13).powi(2)).sqrt() - 1.0;
    let mut paw = pad * 0.13;
    for (tx, ty) in [(-0.17, -0.1), (-0.06, -0.2), (0.06, -0.2), (0.17, -0.1)] {
        paw = paw.min(circle(fx, fy, tx, ty, 0.055));
    }
    let _ = gx;
    c = mix(c, [0.52, 0.82, 0.74], cover(paw * s, px) * 0.7);
    // big kitten face
    let (cx, cy) = (aspect * 0.5, 0.52);
    let head = ((x - cx).powi(2) / 1.0 + (y - cy).powi(2) * 1.15).sqrt() - 0.27;
    let ear = |sx: f32| {
        let ex = cx + sx * 0.19;
        let ey = cy - 0.27;
        let dx = (x - ex).abs();
        (dx * 1.8).max(0.0) - (ey + 0.0 - y + 0.12).clamp(0.0, 0.2) * 0.9
            + (y - (ey + 0.1)).max(0.0) * 2.0
            - 0.0
    };
    let face = head
        .min(ear(-1.0).max(cy - 0.43 - y).max(y - (cy - 0.14)) + 0.0)
        .min(ear(1.0).max(cy - 0.43 - y).max(y - (cy - 0.14)) + 0.0);
    c = mix(c, [0.30, 0.20, 0.17], cover(face, px));
    c = mix(c, [1.0, 0.97, 0.92], cover(face + 0.012, px));
    for sx in [-1.0f32, 1.0] {
        c = mix(
            c,
            [0.30, 0.20, 0.17],
            cover(circle(x, y, cx + sx * 0.10, cy - 0.03, 0.035), px),
        );
        c = mix(
            c,
            [1.0, 0.7, 0.78],
            cover(circle(x, y, cx + sx * 0.17, cy + 0.06, 0.04), px) * 0.7,
        );
    }
    c = mix(
        c,
        [1.0, 0.55, 0.65],
        cover(circle(x, y, cx, cy + 0.03, 0.02), px),
    );
    c
}
