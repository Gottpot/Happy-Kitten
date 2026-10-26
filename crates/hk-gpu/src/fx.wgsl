// Happy-Kitten overlay effects: flames, ice, lightning, equalizers, glitch, laser light, lens
// flares. Everything is computed from maths and noise on the GPU; the only texture is the picture
// the effect lies over (it is copied first, so effects can also distort what is underneath).

struct U {
    res: vec2<f32>,
    time: f32,
    alpha: f32,          // fade in / out of the clip
    kind: u32,
    mirror: u32,
    _p0: u32,
    _p1: u32,
    p0: vec4<f32>,       // width, speed, intensity, hue
    p1: vec4<f32>,       // beat, pos.x, pos.y, seed
    audio: vec4<f32>,    // bass, mid, treble, unused
    hist0: vec4<f32>,    // beat hits 0..150 ms ago
    hist1: vec4<f32>,    // beat hits 200..350 ms ago
    bands: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var base_tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> u: U;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
    let p = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    var o: VsOut;
    o.pos = vec4(p[i], 0.0, 1.0);
    o.uv = vec2((p[i].x + 1.0) * 0.5, (1.0 - p[i].y) * 0.5);
    return o;
}

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

// ---------------------------------------------------------------- settings

fn width_s() -> f32 { return u.p0.x; }
fn speed_s() -> f32 { return u.p0.y; }
fn gain() -> f32 { return u.p0.z; }
fn hue_s() -> f32 { return u.p0.w; }
fn beat_s() -> f32 { return u.p1.x; }
fn pos_s() -> vec2<f32> { return u.p1.yz; }
fn seed() -> f32 { return u.p1.w; }
/// animation clock, already scaled by the speed setting
fn clock() -> f32 { return u.time * speed_s() + seed() * 3.7; }
/// how far the bass pushes the effect, 0..1 (0 when the beat setting is 0)
fn bump() -> f32 { return beat_s() * clamp(u.audio.x * 1.4, 0.0, 1.0); }
fn aspect() -> f32 { return u.res.x / u.res.y; }
/// centred coordinates, height = 1, y down
fn centered(uv: vec2<f32>) -> vec2<f32> { return vec2((uv.x - 0.5) * aspect(), uv.y - 0.5); }

// ---------------------------------------------------------------- maths

fn sq(x: f32) -> f32 { return x * x; }
fn hash11(x: f32) -> f32 { return fract(sin(x * 127.1) * 43758.5453); }
fn hash21(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }
fn hash22(p: vec2<f32>) -> vec2<f32> {
    let q = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(q) * 43758.5453);
}
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2(1.0, 0.0)), w.x),
               mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), w.x), w.y);
}
fn fbm(p: vec2<f32>) -> f32 {
    var amp = 0.5;
    var sum = 0.0;
    var q = p;
    for (var i = 0; i < 4; i++) {
        sum += amp * vnoise(q);
        q = q * 2.03 + vec2(17.3, 9.1);
        amp *= 0.5;
    }
    return sum / 0.9375;
}
fn rot(a: f32) -> mat2x2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c);
}
fn screen(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    return vec3(1.0) - (vec3(1.0) - a) * (vec3(1.0) - clamp(b, vec3(0.0), vec3(1.0)));
}
fn rainbow(t: f32) -> vec3<f32> { return 0.5 + 0.5 * cos(TAU * (t + vec3(0.0, 0.33, 0.67))); }
/// rotate the colour around the grey axis, `turns` of a full circle
fn hue_shift(c: vec3<f32>, turns: f32) -> vec3<f32> {
    let a = turns * TAU;
    let cs = cos(a);
    let sn = sin(a);
    let k = vec3(0.57735027);
    return c * cs + cross(k, c) * sn + k * dot(k, c) * (1.0 - cs);
}
fn tint(c: vec3<f32>) -> vec3<f32> { return hue_shift(c, hue_s()); }

struct Vor {
    f1: f32,
    f2: f32,
    id: vec2<f32>,
}

fn voronoi(p: vec2<f32>) -> Vor {
    let ip = floor(p);
    let fp = fract(p);
    var v: Vor;
    v.f1 = 8.0;
    v.f2 = 8.0;
    v.id = vec2(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2(f32(i), f32(j));
            let d = length(g + hash22(ip + g) - fp);
            if d < v.f1 {
                v.f2 = v.f1;
                v.f1 = d;
                v.id = ip + g;
            } else if d < v.f2 {
                v.f2 = d;
            }
        }
    }
    return v;
}

// ---------------------------------------------------------------- the sound

fn band_i(i: i32) -> f32 {
    let k = u32(clamp(i, 0, 31));
    return u.bands[k / 4u][k % 4u];
}
/// loudness at a position 0..1 across the spectrum (low to high), interpolated
fn band(x: f32) -> f32 {
    let f = clamp(x, 0.0, 1.0) * 31.0;
    let i = i32(floor(f));
    return mix(band_i(i), band_i(i + 1), f - floor(f));
}
fn hist_at(i: u32) -> f32 {
    if i < 4u { return u.hist0[i]; }
    return u.hist1[i - 4u];
}
/// beat strength `age` seconds ago, interpolated
fn onset(age: f32) -> f32 {
    let f = clamp(age / 0.05, 0.0, 7.0);
    let i = u32(floor(f));
    return mix(hist_at(i), hist_at(min(i + 1u, 7u)), f - floor(f));
}

// ---------------------------------------------------------------- frames

/// distance from the picture border (height units, 0 at the border), corners square
fn border_d(uv: vec2<f32>) -> f32 {
    return min(min(uv.x, 1.0 - uv.x) * aspect(), min(uv.y, 1.0 - uv.y));
}
/// the same with rounded corners of radius r
fn border_round(uv: vec2<f32>, r: f32) -> f32 {
    let q = abs(centered(uv)) - vec2(aspect() * 0.5 - r, 0.5 - r);
    return -(length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - r);
}
/// direction pointing from the middle out to the nearest border (smooth round the corners)
fn out_dir(uv: vec2<f32>) -> vec2<f32> {
    let q = centered(uv);
    let a = abs(q) / vec2(aspect() * 0.5, 0.5);
    let v = vec2(sign(q.x) * a.x * a.x * a.x * a.x * a.x * a.x, sign(q.y) * a.y * a.y * a.y * a.y * a.y * a.y);
    return v / max(length(v), 1e-5);
}

fn fire_ramp(f: f32) -> vec3<f32> {
    let a = mix(vec3(0.35, 0.02, 0.0), vec3(1.0, 0.25, 0.0), smoothstep(0.0, 0.4, f));
    let b = mix(a, vec3(1.0, 0.70, 0.10), smoothstep(0.35, 0.75, f));
    return mix(b, vec3(1.0, 0.96, 0.72), smoothstep(0.7, 1.0, f));
}

// 1 - Fire frame: flames drift inwards from all four borders.
fn fx_fire(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let w = mix(0.05, 0.30, width_s()) * (1.0 + 0.5 * bump());
    let b = border_d(uv);
    let q = centered(uv);
    let t = clock() * 1.4;
    let p = q * 5.0 + out_dir(uv) * t * 1.8;
    let n = 0.75 * fbm(p) + 0.25 * fbm(p * 2.3 + vec2(t * 0.7, -t));
    let heat = clamp(1.0 - b / w, 0.0, 1.0);
    let f = smoothstep(0.16, 1.0, heat * (0.35 + 1.3 * n) * gain());
    let flame = tint(fire_ramp(f));
    var col = mix(base, flame, clamp(f * 1.35, 0.0, 1.0));
    let glow = exp(-b / (w * 1.5)) * 0.35 * gain();
    col = screen(col, tint(vec3(1.0, 0.35, 0.05)) * glow);
    return col;
}

// 2 - Ice frame: frost crystals grow in from the border, glittering.
fn fx_ice(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let w = mix(0.05, 0.32, width_s()) * (1.0 + 0.3 * bump());
    let b = border_d(uv);
    let q = centered(uv);
    let t = clock() * 0.4;
    let v = voronoi(q * 9.0);
    let front = w * (0.55 + 0.9 * fbm(q * 3.5 + vec2(7.0, 2.0)));
    let inside = (1.0 - smoothstep(front * 0.85, front, b + 0.025 * v.f1)) * min(gain(), 1.0);
    let h = hash21(v.id);
    let facet = 0.55 + 0.45 * sin(h * TAU + t + dot(q, vec2(1.0, 0.7)) * 3.0);
    var ice = mix(vec3(0.20, 0.55, 0.95), vec3(0.85, 0.97, 1.0), facet * 0.8);
    let edge = 1.0 - smoothstep(0.0, 0.06, v.f2 - v.f1);
    ice = mix(ice, vec3(1.0), edge * 0.7);
    ice = mix(ice, vec3(0.95, 0.99, 1.0), clamp(1.0 - b / w, 0.0, 1.0) * 0.5);
    ice = tint(ice);
    var col = mix(base, ice, inside * 0.88);
    let sp = hash21(v.id * 7.3 + vec2(floor(t * 3.0)));
    let sparkle = step(0.965, sp) * exp(-6.0 * v.f1) * (0.5 + 0.5 * sin(t * 20.0 + h * 40.0));
    col += vec3(1.0) * sparkle * inside * gain();
    return screen(col, tint(vec3(0.35, 0.70, 1.0)) * exp(-b / (w * 1.6)) * 0.25 * gain());
}

// 3 - Lightning frame: jagged bolts flicker along the border.
fn fx_lightning(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let w = mix(0.05, 0.28, width_s()) * (1.0 + 0.5 * bump());
    let b = border_d(uv);
    let q = centered(uv);
    let ring = vec2(cos(atan2(q.y, q.x)), sin(atan2(q.y, q.x)));
    let step_t = floor(u.time * speed_s() * 10.0 + seed());
    var core = 0.0;
    var glow = 0.0;
    for (var i = 0; i < 3; i++) {
        let s = step_t * 0.37 + f32(i) * 11.3 + seed();
        var off = 0.0;
        var amp = 0.5;
        var fr = 3.0;
        for (var o = 0; o < 4; o++) {
            off += (vnoise(ring * fr + vec2(s * 13.0, s)) - 0.5) * amp;
            fr *= 2.1;
            amp *= 0.55;
        }
        let d = abs(b - w * (0.5 + off * 1.6));
        core += exp(-sq(d / 0.004));
        glow += exp(-sq(d / 0.030)) * 0.6;
    }
    let bolt = mix(tint(vec3(0.30, 0.50, 1.0)), tint(vec3(0.92, 0.96, 1.0)), clamp(core, 0.0, 1.0));
    var col = screen(base, bolt * (glow * 0.6 + core) * gain());
    col = screen(col, tint(vec3(0.3, 0.5, 1.0)) * bump() * exp(-b / w) * 0.35);
    return col;
}

// 4 - Neon frame: a glowing line round the picture, the colour runs along it.
fn fx_neon(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let w = mix(0.02, 0.10, width_s());
    let b = border_round(uv, 0.07);
    let inset = 0.03 + 0.5 * w;
    let d = abs(b - inset);
    let q = centered(uv);
    let ang = atan2(q.y, q.x);
    let c = tint(mix(vec3(1.0, 0.10, 0.80), vec3(0.10, 0.90, 1.0), 0.5 + 0.5 * sin(ang * 2.0 + clock())));
    let pulse = 1.0 + bump() * 0.9;
    let line = exp(-sq(d / 0.006));
    let glow = exp(-sq(d / (w * 0.7))) * 0.6;
    let outer = exp(-sq((b - inset * 0.4) / (w * 1.8))) * 0.18;
    return screen(base, c * (line * 1.6 + glow + outer) * gain() * pulse);
}

// ---------------------------------------------------------------- equalizers

// 5 - Bar equalizer: LED-style bars, 32 of them, one per frequency band.
fn fx_bars(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let h_max = mix(0.12, 0.55, width_s());
    let y0 = pos_s().y;
    let x0 = 0.07 + (pos_s().x - 0.5);
    let x1 = 0.93 + (pos_s().x - 0.5);
    let px = (uv.x - x0) / (x1 - x0);
    var col = base;
    if px < 0.0 || px >= 1.0 { return col; }
    let idx = px * 32.0;
    let i = floor(idx);
    let fx = fract(idx);
    let v = clamp(band_i(i32(i)) * gain() * (1.0 + 0.25 * bump()), 0.0, 1.0);
    let h = v * h_max;
    let dy = y0 - uv.y;                       // up from the baseline
    let refl = dy < 0.0;
    let yb = abs(dy);
    // a darker strip behind the bars so they stay readable on bright pictures
    let backdrop = (1.0 - smoothstep(0.0, h_max * 1.15, yb)) * select(0.22, 0.10, refl);
    col *= 1.0 - backdrop;
    let in_bar = abs(fx - 0.5) < 0.36;
    let seg = 0.014;
    let lit = fract(yb / seg) < 0.78;
    if in_bar && yb <= h && lit {
        let t = clamp(yb / h_max, 0.0, 1.0);
        var c = mix(vec3(0.15, 1.0, 0.45), vec3(1.0, 0.90, 0.20), smoothstep(0.35, 0.65, t));
        c = mix(c, vec3(1.0, 0.20, 0.30), smoothstep(0.65, 0.95, t));
        c = tint(c);
        let fade = select(1.0, 0.38 * (1.0 - yb / max(h, 1e-3)), refl);
        col = mix(col, c, 0.95 * fade);
    }
    // the small cap that stays on top of each bar
    let cap = in_bar && abs(yb - h - 0.012) < 0.005 && !refl && h > 0.01;
    if cap { col = mix(col, vec3(1.0), 0.9); }
    return col;
}

// 6 - Circle equalizer: a ring of bars round a centre, the colour runs round the ring.
fn fx_circle(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let c = pos_s();
    let q = (uv - c) * vec2(aspect(), 1.0);
    let r = length(q);
    let r0 = mix(0.10, 0.24, width_s()) * (1.0 + 0.12 * bump());
    let ang = atan2(q.y, q.x) + clock() * 0.25;
    let a01 = fract(ang / TAU);
    let idx = a01 * 64.0;
    let i = floor(idx);
    let f = fract(idx);
    let j = select(63.0 - i, i, i < 32.0);
    let v = clamp(band_i(i32(j)) * gain(), 0.0, 1.0);
    let len = v * 0.24 + 0.01;
    let col_bar = tint(rainbow(a01 + 0.05));
    var col = base;
    let in_bar = abs(f - 0.5) < 0.30 && r >= r0 && r <= r0 + len;
    if in_bar { col = mix(col, col_bar, 0.95); }
    let tip = exp(-sq((r - r0 - len) / 0.006)) * step(abs(f - 0.5), 0.34);
    col = screen(col, col_bar * tip * 0.8);
    let ring = exp(-sq((r - r0) / 0.004));
    col = screen(col, tint(vec3(1.0)) * ring * 0.7);
    col = screen(col, tint(vec3(0.6, 0.8, 1.0)) * exp(-r / (r0 * 0.9)) * u.audio.x * 0.45 * gain());
    return col;
}

// 7 - Wave equalizer: layered glowing curves that follow the spectrum.
fn fx_wave(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let h_max = mix(0.10, 0.50, width_s());
    let y0 = pos_s().y;
    var col = base;
    for (var i = 0; i < 3; i++) {
        let k = f32(i);
        let x = uv.x + (k - 1.0) * 0.015;
        let v = 0.25 * (band(x - 0.01) + 2.0 * band(x) + band(x + 0.01));
        let wob = sin(x * 40.0 + clock() * 3.0 + k) * 0.004 * v;
        let amp = (1.0 - 0.2 * k) * (1.0 + 0.2 * bump());
        let yv = y0 - v * h_max * gain() * amp + wob;
        let c = tint(mix(vec3(0.10, 0.90, 1.0), vec3(1.0, 0.15, 0.85), k / 2.0));
        let d = abs(uv.y - yv);
        let line = exp(-sq(d / 0.003)) + 0.45 * exp(-sq(d / 0.02));
        let below = uv.y > yv && uv.y < y0 + 0.02;
        let fill = select(0.0, 0.22 * (1.0 - clamp((uv.y - yv) / max(y0 - yv + 0.02, 1e-3), 0.0, 1.0)), below);
        col = screen(col, c * (line + fill));
        if u.mirror != 0u {
            let ym = y0 + (y0 - yv);
            let dm = abs(uv.y - ym);
            col = screen(col, c * (exp(-sq(dm / 0.003)) + 0.45 * exp(-sq(dm / 0.02))) * 0.45);
        }
    }
    return col;
}

// 8 - Beat rings: every beat sends a ring out from the centre.
fn fx_rings(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let c = pos_s();
    let q = (uv - c) * vec2(aspect(), 1.0);
    let r = length(q);
    let thick = mix(0.01, 0.05, width_s());
    var col = base;
    var sum = vec3(0.0);
    for (var k = 0; k < 8; k++) {
        let age = 0.05 * f32(k);
        let hit = hist_at(u32(k)) * gain();
        let radius = 0.04 + age * 1.7 * speed_s();
        let ring = exp(-sq((r - radius) / (thick * (1.0 + 0.6 * f32(k) / 7.0))));
        sum += tint(rainbow(f32(k) * 0.11 + 0.55)) * ring * hit;
    }
    col = screen(col, sum * 1.2);
    col = screen(col, tint(vec3(1.0, 0.85, 0.7)) * exp(-r * 5.0) * u.audio.x * 0.5 * gain());
    col = screen(col, tint(vec3(0.5, 0.3, 1.0)) * smoothstep(0.35, 0.9, r) * u.audio.x * 0.30 * gain());
    return col;
}

// ---------------------------------------------------------------- glitch and cyberpunk

// 9 - Glitch: bursts of shifted rows, colour split, scanlines and flashing blocks.
fn fx_glitch(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let step_t = floor(u.time * speed_s() * 12.0 + seed() * 5.0);
    let burst = smoothstep(0.55, 0.9, hash11(step_t * 3.1));
    let amount = mix(0.22, 1.0, burst) * gain() * (1.0 + 1.5 * bump());
    let w = mix(0.04, 0.22, width_s());
    let rows = mix(10.0, 48.0, hash11(step_t * 1.7));
    let row = floor(uv.y * rows);
    let on_row = step(0.55, hash21(vec2(row, step_t + 7.0)));
    let shift = (hash21(vec2(row, step_t)) - 0.5) * 2.0 * w * on_row * amount;
    let uvs = vec2(uv.x + shift, uv.y);
    let split = 0.012 * amount * (0.5 + width_s());
    let r = textureSampleLevel(base_tex, samp, clamp(uvs + vec2(split, 0.0), vec2(0.0), vec2(1.0)), 0.0).r;
    let g = textureSampleLevel(base_tex, samp, clamp(uvs, vec2(0.0), vec2(1.0)), 0.0).g;
    let b = textureSampleLevel(base_tex, samp, clamp(uvs - vec2(split, 0.0), vec2(0.0), vec2(1.0)), 0.0).b;
    var col = tint(vec3(r, g, b));
    col = mix(base, col, clamp(0.35 + 0.65 * amount, 0.0, 1.0));
    col *= 1.0 - 0.10 * (0.5 + 0.5 * sin(uv.y * u.res.y * PI * 0.75)) * min(gain(), 1.0);
    let cell = floor(uv * vec2(14.0, 8.0));
    let blk = step(0.975 - 0.05 * burst, hash21(cell + vec2(step_t)));
    let blk_col = tint(select(vec3(1.0, 0.1, 0.8), vec3(0.1, 0.95, 1.0), hash21(cell + vec2(3.0, step_t)) < 0.5));
    col = mix(col, blk_col, blk * 0.55 * amount);
    let noise = hash21(floor(uv * u.res * 0.5) + vec2(step_t * 17.0));
    col += (noise - 0.5) * 0.10 * burst * gain();
    return col;
}

// 10 - Cyber grid: neon perspective floor, horizon glow, scan lines and corner brackets.
fn fx_grid(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let horizon = pos_s().y;
    let t = clock();
    var col = base;
    let v = uv.y - horizon;
    if v > 0.001 {
        let zz = 0.18 / v;                                    // depth: grows towards the horizon
        let x = (uv.x - 0.5) * aspect() * zz * 4.0;
        let dx = abs(fract(x + 0.5) - 0.5);
        let dz = abs(fract(zz * 1.2 - t * 0.6) - 0.5);
        // lines keep about the same thickness on screen at every depth
        let th = 0.005 * mix(0.6, 2.0, width_s());
        let lx = 1.0 - smoothstep(0.0, th * aspect() * zz * 4.0 + 1e-4, dx);
        let lz = 1.0 - smoothstep(0.0, th * zz * zz / 0.18 * 1.2 + 1e-4, dz);
        let fade = exp(-zz * 0.30);
        let cx = select(vec3(1.0, 0.10, 0.80), vec3(0.10, 0.90, 1.0), fract(floor(x) * 0.5) < 0.25);
        let line = tint(cx) * max(lx, lz) * fade;
        col = mix(col, col * vec3(0.55, 0.45, 0.85), 0.5 * min(gain(), 1.0) * smoothstep(0.0, 0.12, v));
        col = screen(col, line * 1.5 * gain());
    }
    col = screen(col, tint(vec3(1.0, 0.25, 0.75)) * exp(-sq(v / 0.05)) * 0.7 * gain());
    // horizontal scan bar
    let bar_y = fract(t * 0.12);
    col = screen(col, tint(vec3(0.1, 0.9, 1.0)) * exp(-sq((uv.y - bar_y) / 0.012)) * 0.35 * gain());
    col *= 1.0 - 0.10 * (0.5 + 0.5 * sin(uv.y * u.res.y * PI)) * min(gain(), 1.0);
    // corner brackets
    let a = aspect();
    let cx = min(uv.x, 1.0 - uv.x) * a;
    let cy = min(uv.y, 1.0 - uv.y);
    let inset = 0.04;
    let arm = 0.10;
    let on_x = abs(cy - inset) < 0.0025 && cx > inset - 0.0025 && cx < inset + arm;
    let on_y = abs(cx - inset) < 0.0025 && cy > inset - 0.0025 && cy < inset + arm;
    if on_x || on_y { col = mix(col, tint(vec3(0.1, 0.95, 1.0)), 0.9 * min(gain(), 1.0)); }
    col = screen(col, tint(vec3(0.25, 0.1, 0.5)) * smoothstep(0.55, 1.1, length(centered(uv))) * 0.25 * gain());
    return col;
}

// 11 - Laser show: a fan of beams sweeps through the haze.
fn fx_laser(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let o = pos_s();
    let q = (uv - o) * vec2(aspect(), 1.0);
    let r = length(q);
    let th = atan2(q.y, q.x);
    let t = clock();
    var col = base;
    var sum = vec3(0.0);
    let thin = mix(0.0015, 0.007, width_s());
    for (var i = 0; i < 14; i++) {
        let fi = f32(i);
        let a = -0.5 * PI + (fi / 13.0 - 0.5) * 1.9 + 0.35 * sin(t * 0.8 + fi * 0.9);
        let dir = vec2(cos(a), sin(a));
        let along = dot(q, dir);
        if along <= 0.0 { continue; }
        let d = abs(q.x * dir.y - q.y * dir.x);
        let c = tint(rainbow(fi / 14.0 + t * 0.03));
        let fall = 0.55 + 0.45 * exp(-r * 0.7);
        sum += c * (exp(-sq(d / (thin * (1.0 + 0.7 * bump())))) * 1.3 + exp(-sq(d / 0.035)) * 0.28) * fall;
    }
    col = screen(col, sum * gain() * (1.0 + 0.5 * bump()));
    col = screen(col, tint(vec3(0.5, 0.6, 1.0)) * exp(-r * 3.0) * 0.18 * gain());
    return col;
}

// ---------------------------------------------------------------- light

// 12 - Lens flare: bright core, streak, rays and ghosts along the line through the centre.
fn fx_flare(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let t = clock();
    let s = pos_s() + 0.02 * vec2(sin(t * 0.7), cos(t * 0.9));
    let asp = vec2(aspect(), 1.0);
    let p = (uv - s) * asp;
    let r = length(p);
    let size = mix(0.04, 0.16, width_s()) * (1.0 + 0.4 * bump());
    var f = vec3(0.0);
    f += vec3(1.0, 0.95, 0.80) * (exp(-sq(r / size * 1.2)) + 0.25 * exp(-r / (size * 4.0)));
    f += tint(vec3(0.40, 0.60, 1.0)) * exp(-abs(p.y) / (size * 0.08)) * exp(-abs(p.x) / (size * 5.0)) * 0.8;
    let ang = atan2(p.y, p.x);
    f += tint(vec3(1.0, 0.85, 0.6)) * pow(abs(cos(ang * 5.0 + t * 0.1)), 40.0) * exp(-r / (size * 3.0)) * 0.4;
    f += tint(vec3(1.0, 0.8, 0.6)) * exp(-sq((r - size * 3.2) / (size * 0.25))) * 0.2;
    let axis = (vec2(0.5, 0.5) - s) * asp;
    for (var k = 0; k < 5; k++) {
        let fk = f32(k);
        let g = (s * asp) + axis * (0.35 + fk * 0.45);
        let gr = size * (0.6 + 0.35 * fract(fk * 0.618));
        let d = length(uv * asp - g);
        let c = tint(rainbow(fk * 0.19 + 0.1));
        f += c * (exp(-sq((d - gr) / (gr * 0.15))) * 0.45 + smoothstep(gr, gr * 0.85, d) * 0.10);
    }
    return screen(base, f * gain());
}

// 13 - Light leak: warm blobs of light drift over the picture like film burns.
fn fx_leak(uv: vec2<f32>, base: vec3<f32>) -> vec3<f32> {
    let t = clock() * 0.5;
    let rho = mix(0.25, 0.6, width_s());
    var sum = vec3(0.0);
    for (var i = 0; i < 4; i++) {
        let fi = f32(i);
        let c = vec2(0.5 + 0.62 * sin(t * 0.42 + fi * 1.9), 0.5 + 0.6 * cos(t * 0.34 + fi * 2.7));
        let d = length((uv - c) * vec2(aspect(), 1.0));
        let radius = rho * (0.8 + 0.2 * sin(t + fi));
        let flick = 0.85 + 0.15 * vnoise(vec2(t * 8.0, fi * 3.0));
        var col = vec3(1.0, 0.45, 0.10);
        if i == 1 { col = vec3(1.0, 0.20, 0.35); }
        if i == 2 { col = vec3(1.0, 0.80, 0.30); }
        if i == 3 { col = vec3(0.95, 0.25, 0.60); }
        sum += tint(col) * exp(-sq(d / radius)) * flick;
    }
    let d2 = dot(centered(uv), normalize(vec2(0.8, -0.6)));
    sum += tint(vec3(1.0, 0.55, 0.25)) * exp(-sq((d2 - 0.6 * sin(t * 0.3)) / 0.12)) * 0.35;
    return screen(base, sum * 0.8 * gain() * (1.0 + 0.8 * bump()));
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let base = textureSampleLevel(base_tex, samp, uv, 0.0).rgb;
    var col = base;
    switch u.kind {
        case 1u: { col = fx_fire(uv, base); }
        case 2u: { col = fx_ice(uv, base); }
        case 3u: { col = fx_lightning(uv, base); }
        case 4u: { col = fx_neon(uv, base); }
        case 5u: { col = fx_bars(uv, base); }
        case 6u: { col = fx_circle(uv, base); }
        case 7u: { col = fx_wave(uv, base); }
        case 8u: { col = fx_rings(uv, base); }
        case 9u: { col = fx_glitch(uv, base); }
        case 10u: { col = fx_grid(uv, base); }
        case 11u: { col = fx_laser(uv, base); }
        case 12u: { col = fx_flare(uv, base); }
        case 13u: { col = fx_leak(uv, base); }
        default: {}
    }
    return vec4(clamp(mix(base, col, clamp(u.alpha, 0.0, 1.0)), vec3(0.0), vec3(1.0)), 1.0);
}
