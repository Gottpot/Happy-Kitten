//! Draws a [`TextSpec`] into a transparent frame-sized picture (premultiplied
//! RGBA8): word wrap, alignment, outline, shadow and background box.
//! Fonts are read from Android's own /system/fonts; nothing is bundled.

use fontdue::{Font, FontSettings};
use hk_core::{Align, TextSpec};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// A frame-sized picture with premultiplied alpha.
pub struct Raster {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

const SYSTEM_FONTS: &str = "/system/fonts";

/// Path and collection index of the Android font behind a font id.
pub fn font_path(id: &str) -> Option<(String, u32)> {
    font_file(id)
}

/// The first of `names` that exists in /system/fonts. Devices differ, so every id has a few
/// candidates; the order is "closest to the Mac look first".
fn first_existing(names: &[&str]) -> Option<(String, u32)> {
    names
        .iter()
        .map(|n| format!("{SYSTEM_FONTS}/{n}"))
        .find(|p| std::path::Path::new(p).exists())
        .map(|p| (p, 0))
}

const SANS_BOLD: &[&str] = &[
    "Roboto-Bold.ttf",
    "RobotoStatic-Bold.ttf",
    "Roboto[wdth,wght].ttf",
    "Roboto-Regular.ttf",
    "DroidSans-Bold.ttf",
];

fn font_file(id: &str) -> Option<(String, u32)> {
    let with_sans = |first: &[&str]| {
        let mut all = first.to_vec();
        all.extend_from_slice(SANS_BOLD);
        first_existing(&all)
    };
    match id {
        "rounded" => with_sans(&["GoogleSans-Bold.ttf", "GoogleSansText-Bold.ttf", "ProductSans-Bold.ttf"]),
        "comic" => with_sans(&["ComingSoon.ttf"]),
        "trebuchet" => with_sans(&["SourceSansPro-Bold.ttf"]),
        "arial" => with_sans(&[]),
        "georgia" => with_sans(&["NotoSerif-Bold.ttf", "DroidSerif-Bold.ttf", "NotoSerif-Regular.ttf"]),
        "impact" => with_sans(&["Roboto-Black.ttf", "RobotoStatic-Black.ttf"]),
        "marker" => with_sans(&["DancingScript-Bold.ttf", "DancingScript-Regular.ttf"]),
        "chalk" => with_sans(&["ComingSoon.ttf"]),
        "futura" => with_sans(&["Roboto-Medium.ttf", "RobotoStatic-Medium.ttf"]),
        "typewriter" => with_sans(&["CutiveMono.ttf", "DroidSansMono.ttf"]),
        _ => None,
    }
}

/// Fonts for letters the chosen font does not have (Japanese, Chinese, Korean, symbols),
/// in the order they are tried. `serif` prefers the serif CJK file.
pub fn fallback_paths(serif: bool, _round: bool) -> Vec<(String, u32)> {
    let mut v: Vec<String> = Vec::new();
    if serif {
        v.push(format!("{SYSTEM_FONTS}/NotoSerifCJK-Regular.ttc"));
    }
    v.push(format!("{SYSTEM_FONTS}/NotoSansCJK-Regular.ttc"));
    v.push(format!("{SYSTEM_FONTS}/NotoSansJP-Regular.otf"));
    v.push(format!("{SYSTEM_FONTS}/NotoSansKR-Regular.otf"));
    v.push(format!("{SYSTEM_FONTS}/NotoSansSymbols-Regular-Subsetted.ttf"));
    v.into_iter()
        .filter(|p| std::path::Path::new(p).exists())
        .map(|p| (p, 0))
        .collect()
}

/// The chosen font plus its fallbacks; each character is drawn with the first font that has it.
struct Chain {
    fonts: Vec<Arc<Font>>,
}

impl Chain {
    fn pick(&self, ch: char) -> &Arc<Font> {
        self.fonts
            .iter()
            .find(|f| f.lookup_glyph_index(ch) != 0)
            .unwrap_or(&self.fonts[0])
    }
}

type FontCache = Mutex<HashMap<String, Option<Arc<Font>>>>;

fn cache() -> &'static FontCache {
    static CACHE: OnceLock<FontCache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

fn load(path: &str, index: u32) -> Option<Font> {
    let bytes = std::fs::read(path).ok()?;
    Font::from_bytes(
        bytes,
        FontSettings {
            collection_index: index,
            ..Default::default()
        },
    )
    .ok()
}

/// The font for `id`, falling back to the rounded font and then Helvetica.
fn font_cached(key: &str, make: impl FnOnce() -> Option<Font>) -> Option<Arc<Font>> {
    let mut map = cache().lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(f) = map.get(key) {
        return f.clone();
    }
    let loaded = make().map(Arc::new);
    map.insert(key.to_string(), loaded.clone());
    loaded
}

fn chain(id: &str) -> Option<Chain> {
    let primary = font(id)?;
    let mut fonts = vec![primary];
    for (p, i) in fallback_paths(id == "georgia", matches!(id, "rounded" | "comic")) {
        if let Some(f) = font_cached(&format!("fb:{p}:{i}"), || load(&p, i)) {
            fonts.push(f);
        }
    }
    Some(Chain { fonts })
}

fn font(id: &str) -> Option<Arc<Font>> {
    let mut map = cache().lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(f) = map.get(id) {
        return f.clone();
    }
    let loaded = font_file(id)
        .and_then(|(p, i)| load(&p, i))
        .or_else(|| font_file("rounded").and_then(|(p, i)| load(&p, i)))
        .or_else(|| load("/system/fonts/Roboto-Regular.ttf", 0))
        .map(Arc::new);
    map.insert(id.to_string(), loaded.clone());
    loaded
}

/// True when the machine has this exact font (the UI greys out the missing ones).
pub fn font_available(id: &str) -> bool {
    font_file(id).is_some_and(|(p, _)| std::path::Path::new(&p).exists())
}

struct Line {
    text: String,
    width: f32,
}

fn measure(chain: &Chain, s: &str, px: f32) -> f32 {
    let mut w = 0.0;
    let mut prev: Option<(char, &Arc<Font>)> = None;
    for ch in s.chars() {
        let f = chain.pick(ch);
        if let Some((p, pf)) = prev
            && Arc::ptr_eq(pf, f)
        {
            w += f.horizontal_kern(p, ch, px).unwrap_or(0.0);
        }
        w += f.metrics(ch, px).advance_width;
        prev = Some((ch, f));
    }
    w
}

/// Break text into lines no wider than `max`: at spaces, and inside words that are too long alone
/// (which is also how Japanese, written without spaces, gets wrapped).
fn wrap(chain: &Chain, text: &str, px: f32, max: f32) -> Vec<Line> {
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        for word in para.split(' ') {
            let candidate = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{cur} {word}")
            };
            if measure(chain, &candidate, px) <= max || cur.is_empty() && word.is_empty() {
                cur = candidate;
                continue;
            }
            if !cur.is_empty() {
                lines.push(Line {
                    width: measure(chain, &cur, px),
                    text: std::mem::take(&mut cur),
                });
            }
            let mut chunk = String::new();
            for ch in word.chars() {
                let mut t = chunk.clone();
                t.push(ch);
                if measure(chain, &t, px) > max && !chunk.is_empty() {
                    lines.push(Line {
                        width: measure(chain, &chunk, px),
                        text: std::mem::take(&mut chunk),
                    });
                }
                chunk.push(ch);
            }
            cur = chunk;
        }
        lines.push(Line {
            width: measure(chain, &cur, px),
            text: cur,
        });
    }
    lines
}

/// Euclidean-ish distance (chamfer 3-4) of every pixel to the nearest "inside" pixel.
fn distance_to_inside(inside: &[bool], w: usize, h: usize) -> Vec<f32> {
    const INF: f32 = 1e9;
    let mut d: Vec<f32> = inside.iter().map(|&i| if i { 0.0 } else { INF }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut v = d[i];
            if x > 0 {
                v = v.min(d[i - 1] + a);
            }
            if y > 0 {
                v = v.min(d[i - w] + a);
                if x > 0 {
                    v = v.min(d[i - w - 1] + b);
                }
                if x + 1 < w {
                    v = v.min(d[i - w + 1] + b);
                }
            }
            d[i] = v;
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            let mut v = d[i];
            if x + 1 < w {
                v = v.min(d[i + 1] + a);
            }
            if y + 1 < h {
                v = v.min(d[i + w] + a);
                if x + 1 < w {
                    v = v.min(d[i + w + 1] + b);
                }
                if x > 0 {
                    v = v.min(d[i + w - 1] + b);
                }
            }
            d[i] = v;
        }
    }
    d
}

fn box_blur(a: &mut [f32], w: usize, h: usize, r: usize) {
    if r == 0 || w == 0 || h == 0 {
        return;
    }
    let mut tmp = vec![0.0f32; a.len()];
    let n = (2 * r + 1) as f32;
    for y in 0..h {
        let row = &a[y * w..(y + 1) * w];
        let mut sum: f32 = row.iter().take(r + 1).sum();
        for x in 0..w {
            tmp[y * w + x] = sum / n;
            if x + r + 1 < w {
                sum += row[x + r + 1];
            }
            if x >= r {
                sum -= row[x - r];
            }
        }
    }
    for x in 0..w {
        let mut sum: f32 = (0..=r.min(h - 1)).map(|y| tmp[y * w + x]).sum();
        for y in 0..h {
            a[y * w + x] = sum / n;
            if y + r + 1 < h {
                sum += tmp[(y + r + 1) * w + x];
            }
            if y >= r {
                sum -= tmp[(y - r) * w + x];
            }
        }
    }
}

/// Draw `spec` into a `width` x `height` picture. `None` only when no font could be loaded at all.
pub fn render(spec: &TextSpec, width: u32, height: u32) -> Option<Raster> {
    let (w, h) = (width.max(1) as usize, height.max(1) as usize);
    let mut pixels = vec![0u8; w * h * 4];
    let chain = chain(&spec.font)?;
    let font = chain.fonts[0].clone();
    if spec.text.trim().is_empty() {
        return Some(Raster {
            width,
            height,
            pixels,
        });
    }
    // Size follows the frame width: in portrait the whole width counts (captions on a phone are
    // read small), in landscape the height, capped at the 16:9 look.
    let reference = if h > w {
        w as f32
    } else {
        (h as f32).min(w as f32 * 0.5625)
    };
    let px = (spec.size * reference).clamp(8.0, h as f32);
    // A long Latin word is shrunk to fit instead of being torn apart (Japanese, which has no
    // spaces, is still wrapped between characters).
    let max_w = w as f32 * 0.9;
    let widest_word = spec
        .text
        .split([' ', '\n'])
        .filter(|word| word.chars().all(|c| (c as u32) < 0x2E80))
        .map(|word| measure(&chain, word, px))
        .fold(0.0f32, f32::max);
    let px = if widest_word > max_w {
        (px * max_w / widest_word).max(px * 0.45)
    } else {
        px
    };
    let lines = wrap(&chain, &spec.text, px, max_w);
    let line_h = px * spec.line_spacing.clamp(0.8, 2.5);
    let block_w = lines.iter().map(|l| l.width).fold(0.0, f32::max);
    let block_h = line_h * lines.len() as f32;
    let ascent = font
        .horizontal_line_metrics(px)
        .map_or(px * 0.8, |m| m.ascent);
    let pad = px * 0.15;
    // keep the block inside the frame
    let cx = (spec.pos[0] * w as f32).clamp(
        block_w / 2.0 + pad,
        (w as f32 - block_w / 2.0 - pad).max(block_w / 2.0 + pad),
    );
    let cy = (spec.pos[1] * h as f32).clamp(
        block_h / 2.0 + pad,
        (h as f32 - block_h / 2.0 - pad).max(block_h / 2.0 + pad),
    );
    let (x0, y0) = (cx - block_w / 2.0, cy - block_h / 2.0);

    // glyph coverage in a frame-sized buffer
    let mut fill = vec![0.0f32; w * h];
    for (li, line) in lines.iter().enumerate() {
        let lx = match spec.align {
            Align::Left => x0,
            Align::Center => cx - line.width / 2.0,
            Align::Right => x0 + block_w - line.width,
        };
        let baseline = y0 + li as f32 * line_h + (line_h - px) / 2.0 + ascent;
        let mut pen = lx;
        let mut prev: Option<(char, Arc<Font>)> = None;
        for ch in line.text.chars() {
            let gf = chain.pick(ch).clone();
            if let Some((p, pf)) = &prev
                && Arc::ptr_eq(pf, &gf)
            {
                pen += gf.horizontal_kern(*p, ch, px).unwrap_or(0.0);
            }
            let (m, bitmap) = gf.rasterize(ch, px);
            let gx = (pen + m.xmin as f32).round() as i64;
            let gy = (baseline - m.ymin as f32 - m.height as f32).round() as i64;
            for row in 0..m.height {
                for col in 0..m.width {
                    let (x, y) = (gx + col as i64, gy + row as i64);
                    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
                        let c =
                            bitmap.get(row * m.width + col).copied().unwrap_or(0) as f32 / 255.0;
                        let cell = &mut fill[y as usize * w + x as usize];
                        *cell = cell.max(c);
                    }
                }
            }
            pen += m.advance_width;
            prev = Some((ch, gf));
        }
    }

    // work region around the text
    let outline_r = (spec.outline_width * px).clamp(0.0, px * 0.4);
    let margin = (outline_r
        + px * 0.5
        + if spec.background[3] > 0 {
            px * 0.6
        } else {
            0.0
        })
    .ceil() as i64;
    let rx0 = ((x0 as i64) - margin).clamp(0, w as i64) as usize;
    let ry0 = ((y0 as i64) - margin).clamp(0, h as i64) as usize;
    let rx1 = ((x0 + block_w) as i64 + margin).clamp(0, w as i64) as usize;
    let ry1 = ((y0 + block_h) as i64 + margin).clamp(0, h as i64) as usize;
    let (rw, rh) = (rx1.saturating_sub(rx0), ry1.saturating_sub(ry0));
    if rw == 0 || rh == 0 {
        return Some(Raster {
            width,
            height,
            pixels,
        });
    }
    let at = |x: usize, y: usize| (y + ry0) * w + x + rx0;

    // outline from the distance to the glyphs
    let mut outline = vec![0.0f32; rw * rh];
    if outline_r > 0.3 {
        let inside: Vec<bool> = (0..rh)
            .flat_map(|y| (0..rw).map(move |x| (x, y)))
            .map(|(x, y)| fill[at(x, y)] >= 0.5)
            .collect();
        let dist = distance_to_inside(&inside, rw, rh);
        for (o, d) in outline.iter_mut().zip(&dist) {
            *o = (outline_r - d + 0.5).clamp(0.0, 1.0);
        }
    }
    // shadow: the silhouette, moved down-right and blurred
    let mut shadow = vec![0.0f32; rw * rh];
    if spec.shadow {
        let (sx, sy) = ((px * 0.05).round() as i64, (px * 0.07).round() as i64);
        for y in 0..rh as i64 {
            for x in 0..rw as i64 {
                let (ux, uy) = (x - sx, y - sy);
                if ux >= 0 && uy >= 0 && (ux as usize) < rw && (uy as usize) < rh {
                    let i = uy as usize * rw + ux as usize;
                    shadow[(y as usize) * rw + x as usize] =
                        outline[i].max(fill[at(ux as usize, uy as usize)]);
                }
            }
        }
        box_blur(&mut shadow, rw, rh, (px * 0.04).round() as usize);
        box_blur(&mut shadow, rw, rh, (px * 0.04).round() as usize);
    }

    let over = |dst: &mut [u8], color: [u8; 4], a: f32| {
        let a = (a * color[3] as f32 / 255.0).clamp(0.0, 1.0);
        if a <= 0.0 {
            return;
        }
        for c in 0..3 {
            let src = color[c] as f32 * a;
            dst[c] = (src + dst[c] as f32 * (1.0 - a)).round().min(255.0) as u8;
        }
        dst[3] = (a * 255.0 + dst[3] as f32 * (1.0 - a)).round().min(255.0) as u8;
    };

    // background box with rounded corners
    let (bx0, by0) = (x0 - px * 0.35, y0 - px * 0.1);
    let (bx1, by1) = (x0 + block_w + px * 0.35, y0 + block_h + px * 0.1);
    let corner = px * 0.3;
    for y in 0..rh {
        for x in 0..rw {
            let i = (y + ry0) * w + x + rx0;
            let px4 = &mut pixels[i * 4..i * 4 + 4];
            if spec.background[3] > 0 {
                let (fx, fy) = ((x + rx0) as f32 + 0.5, (y + ry0) as f32 + 0.5);
                let dx = ((bx0 + corner - fx).max(fx - (bx1 - corner))).max(0.0);
                let dy = ((by0 + corner - fy).max(fy - (by1 - corner))).max(0.0);
                let d = (dx * dx + dy * dy).sqrt() - corner;
                if fx > bx0 && fx < bx1 && fy > by0 && fy < by1 || d < 0.0 {
                    over(px4, spec.background, (0.5 - d).clamp(0.0, 1.0));
                }
            }
            over(px4, [0, 0, 0, 150], shadow[y * rw + x]);
            over(px4, spec.outline, outline[y * rw + x]);
            over(px4, spec.color, fill[i]);
        }
    }
    Some(Raster {
        width,
        height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha_bounds(r: &Raster) -> Option<(usize, usize, usize, usize)> {
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0, 0);
        for y in 0..r.height as usize {
            for x in 0..r.width as usize {
                if r.pixels[(y * r.width as usize + x) * 4 + 3] > 8 {
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        (x0 != usize::MAX).then_some((x0, y0, x1, y1))
    }

    #[test]
    fn text_lands_where_pos_says() {
        let spec = TextSpec {
            text: "Hallo".into(),
            pos: [0.5, 0.5],
            shadow: false,
            ..Default::default()
        };
        let r = render(&spec, 640, 360).expect("a system font");
        let (x0, y0, x1, y1) = alpha_bounds(&r).expect("something drawn");
        let (cx, cy) = ((x0 + x1) as f32 / 2.0, (y0 + y1) as f32 / 2.0);
        assert!((cx - 320.0).abs() < 25.0, "centre x {cx}");
        assert!((cy - 180.0).abs() < 25.0, "centre y {cy}");
    }

    #[test]
    fn empty_text_is_transparent_and_huge_text_stays_inside() {
        let blank = render(
            &TextSpec {
                text: "  ".into(),
                ..Default::default()
            },
            320,
            180,
        )
        .unwrap();
        assert!(alpha_bounds(&blank).is_none());
        let long = "Wort ".repeat(200);
        let spec = TextSpec {
            text: long,
            size: 0.2,
            pos: [0.9, 0.95],
            ..Default::default()
        };
        let r = render(&spec, 320, 180).unwrap();
        assert_eq!(r.pixels.len(), 320 * 180 * 4);
    }

    #[test]
    fn colour_outline_and_background_show_up() {
        let spec = TextSpec {
            text: "I".into(),
            color: [255, 0, 0, 255],
            outline: [0, 0, 255, 255],
            background: [0, 255, 0, 255],
            shadow: false,
            size: 0.3,
            ..Default::default()
        };
        let r = render(&spec, 320, 180).unwrap();
        let has = |f: &dyn Fn(&[u8]) -> bool| r.pixels.chunks(4).any(f);
        assert!(
            has(&|p| p[0] > 200 && p[1] < 40 && p[2] < 40 && p[3] > 200),
            "red fill"
        );
        assert!(
            has(&|p| p[2] > 200 && p[0] < 40 && p[1] < 40),
            "blue outline"
        );
        assert!(has(&|p| p[1] > 200 && p[0] < 40 && p[2] < 40), "green box");
    }

    #[test]
    fn unknown_font_falls_back() {
        let spec = TextSpec {
            text: "x".into(),
            font: "gibt-es-nicht".into(),
            ..Default::default()
        };
        assert!(render(&spec, 160, 90).is_some());
    }

    #[test]
    fn right_alignment_pushes_short_lines_right() {
        let spec = TextSpec {
            text: "Ein sehr langer Anfang\nkurz".into(),
            align: Align::Right,
            shadow: false,
            outline_width: 0.0,
            size: 0.1,
            ..Default::default()
        };
        let r = render(&spec, 640, 360).unwrap();
        let (_, _, x1, _) = alpha_bounds(&r).unwrap();
        // both lines end at the same right edge: the last row of the block has ink near x1
        let w = r.width as usize;
        let ink_right_of = |x: usize| {
            (0..r.height as usize).any(|y| (x..w).any(|xx| r.pixels[(y * w + xx) * 4 + 3] > 128))
        };
        assert!(ink_right_of(x1.saturating_sub(10)));
    }

    #[test]
    fn japanese_uses_a_fallback_font_instead_of_boxes() {
        let chain = chain("rounded").expect("font");
        for ch in "こんにちは世界、漢字".chars() {
            assert_ne!(chain.pick(ch).lookup_glyph_index(ch), 0, "no font for {ch}");
        }
        let spec = TextSpec {
            text: "こんにちは世界".into(),
            shadow: false,
            outline_width: 0.0,
            ..Default::default()
        };
        let r = render(&spec, 640, 360).unwrap();
        assert!(alpha_bounds(&r).is_some());
    }

    #[test]
    fn long_japanese_without_spaces_wraps() {
        let chain = chain("rounded").expect("font");
        let lines = wrap(&chain, &"あ".repeat(80), 60.0, 600.0);
        assert!(lines.len() > 1 && lines.iter().all(|l| l.width <= 600.0 + 1.0));
    }

    #[test]
    fn portrait_frames_size_text_by_their_width() {
        let spec = TextSpec {
            text: "I".into(),
            shadow: false,
            outline_width: 0.0,
            size: 0.2,
            ..Default::default()
        };
        let tall = alpha_bounds(&render(&spec, 360, 640).unwrap()).unwrap();
        let wide = alpha_bounds(&render(&spec, 640, 360).unwrap()).unwrap();
        // portrait text is sized by the full width, so it is relatively bigger than in landscape
        let (th, wh) = (
            (tall.3 - tall.1) as f32 / 360.0,
            (wide.3 - wide.1) as f32 / 640.0,
        );
        assert!(
            th > wh * 1.4,
            "letter height per width: portrait {th} vs landscape {wh}"
        );
    }

    #[test]
    fn a_long_latin_word_shrinks_instead_of_breaking() {
        let spec = TextSpec {
            text: "Happy-Kitten".into(),
            size: 0.5,
            shadow: false,
            outline_width: 0.0,
            ..Default::default()
        };
        let r = render(&spec, 360, 640).unwrap();
        let (x0, y0, x1, y1) = alpha_bounds(&r).unwrap();
        assert!(x1 - x0 < 330, "fits the width");
        assert!(y1 - y0 < 200, "one line only");
    }
}
