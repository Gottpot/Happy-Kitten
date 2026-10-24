// Happy-Kitten transitions. Every blend is computed from signed distance
// fields and noise on the GPU; there are no pre-rendered paw sprites.
// All maths run on gamma-encoded RGB, which is also what the egui preview shows.

struct U {
    res: vec2<f32>,
    progress: f32,
    time: f32,
    kind: u32,
    fit: u32,        // 0 contain, 1 cover (crop), 2 contain over a blurred copy
    _p1: u32,
    _p2: u32,
    a_x: vec4<f32>, // uv scale.xy, offset.zw : maps output uv to source uv ("contain" fit)
    b_x: vec4<f32>,
    a_c: vec4<f32>, // same for the "cover" fit
    b_c: vec4<f32>,
    a_gain: vec4<f32>, // colour grade: per-channel gain of picture a / b
    b_gain: vec4<f32>,
}

@group(0) @binding(0) var tex_a: texture_2d<f32>;
@group(0) @binding(1) var tex_b: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var<uniform> u: U;

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
const PINK: vec3<f32> = vec3<f32>(1.0, 0.70, 0.79);
const CREAM: vec3<f32> = vec3<f32>(1.0, 0.97, 0.92);
const BROWN: vec3<f32> = vec3<f32>(0.29, 0.16, 0.12);

// ---------------------------------------------------------------- sampling

fn sample_src(tex: texture_2d<f32>, uv: vec2<f32>, x: vec4<f32>, xc: vec4<f32>) -> vec3<f32> {
    if u.fit == 1u {
        return textureSampleLevel(tex, samp, clamp(uv * xc.xy + xc.zw, vec2(0.0), vec2(1.0)), 0.0).rgb;
    }
    let p = uv * x.xy + x.zw;
    let inside = all(p >= vec2(0.0)) && all(p <= vec2(1.0));
    if inside || u.fit == 0u {
        let c = textureSampleLevel(tex, samp, p, 0.0).rgb;
        return select(vec3(0.0), c, inside);
    }
    // blurred, slightly darkened enlarged copy behind the picture
    let pc = uv * xc.xy + xc.zw;
    var acc = vec3(0.0);
    for (var i = 0; i < 24; i++) {
        let a = f32(i) * 2.39996;
        let r = 0.012 + 0.05 * sqrt(f32(i) / 24.0);
        acc += textureSampleLevel(tex, samp, clamp(pc + vec2(cos(a), sin(a)) * r, vec2(0.0), vec2(1.0)), 0.0).rgb;
    }
    return acc / 24.0 * 0.7;
}
fn sa(uv: vec2<f32>) -> vec3<f32> { return min(sample_src(tex_a, uv, u.a_x, u.a_c) * u.a_gain.rgb, vec3(1.0)); }
fn sb(uv: vec2<f32>) -> vec3<f32> { return min(sample_src(tex_b, uv, u.b_x, u.b_c) * u.b_gain.rgb, vec3(1.0)); }

fn aspect() -> f32 { return u.res.x / u.res.y; }
// uv -> centred coordinates, height = 1, y down
fn centered(uv: vec2<f32>) -> vec2<f32> { return vec2((uv.x - 0.5) * aspect(), uv.y - 0.5); }
fn from_centered(q: vec2<f32>) -> vec2<f32> { return vec2(q.x / aspect() + 0.5, q.y + 0.5); }

// ---------------------------------------------------------------- maths

fn rot(a: f32) -> mat2x2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c);
}
fn hash11(x: f32) -> f32 { return fract(sin(x * 127.1) * 43758.5453); }
fn hash21(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }
fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash21(i), hash21(i + vec2(1.0, 0.0)), w.x),
               mix(hash21(i + vec2(0.0, 1.0)), hash21(i + vec2(1.0, 1.0)), w.x), w.y);
}
// x * x. Not pow(x, 2.0): pow is undefined for negative x, which these rim formulas often have.
fn sq(x: f32) -> f32 { return x * x; }
fn ease_io(t: f32) -> f32 { return t * t * (3.0 - 2.0 * t); }
fn ease_out(t: f32) -> f32 { let v = 1.0 - t; return 1.0 - v * v * v; }
fn smin(a: f32, b: f32, k: f32) -> f32 {
    let h = max(k - abs(a - b), 0.0) / k;
    return min(a, b) - h * h * k * 0.25;
}

// ---------------------------------------------------------------- shapes

fn sd_ell(p: vec2<f32>, c: vec2<f32>, r: vec2<f32>, a: f32) -> f32 {
    let q = rot(-a) * (p - c);
    let k0 = length(q / r);
    let k1 = length(q / (r * r));
    return select(k0 * (k0 - 1.0) / max(k1, 1e-5), -min(r.x, r.y), k0 < 1e-4);
}

// A cat paw: one three-lobed pad and four toe beans. Toes point towards -y,
// the paw is about 1.6 units wide and 1.4 high.
fn sd_paw(p: vec2<f32>) -> f32 {
    var pad = smin(length(p - vec2(-0.27, 0.40)) - 0.27, length(p - vec2(0.27, 0.40)) - 0.27, 0.18);
    pad = smin(pad, length(p - vec2(0.0, 0.16)) - 0.34, 0.2);
    var d = pad;
    d = min(d, sd_ell(p, vec2(-0.66, -0.06), vec2(0.17, 0.23), -0.45));
    d = min(d, sd_ell(p, vec2(-0.23, -0.50), vec2(0.18, 0.25), -0.15));
    d = min(d, sd_ell(p, vec2(0.23, -0.50), vec2(0.18, 0.25), 0.15));
    d = min(d, sd_ell(p, vec2(0.66, -0.06), vec2(0.17, 0.23), 0.45));
    return d;
}

fn sd_tri(p: vec2<f32>, p0: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
    let e0 = p1 - p0; let e1 = p2 - p1; let e2 = p0 - p2;
    let v0 = p - p0; let v1 = p - p1; let v2 = p - p2;
    let pq0 = v0 - e0 * clamp(dot(v0, e0) / dot(e0, e0), 0.0, 1.0);
    let pq1 = v1 - e1 * clamp(dot(v1, e1) / dot(e1, e1), 0.0, 1.0);
    let pq2 = v2 - e2 * clamp(dot(v2, e2) / dot(e2, e2), 0.0, 1.0);
    let s = sign(e0.x * e2.y - e0.y * e2.x);
    let d = min(min(vec2(dot(pq0, pq0), s * (v0.x * e0.y - v0.y * e0.x)),
                    vec2(dot(pq1, pq1), s * (v1.x * e1.y - v1.y * e1.x))),
                    vec2(dot(pq2, pq2), s * (v2.x * e2.y - v2.y * e2.x)));
    return -sqrt(d.x) * sign(d.y);
}

fn sd_seg(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    return length(pa - ba * clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0));
}

// Side view of a galloping cat facing +x (units: body length about 1.8).
fn sd_runner(p: vec2<f32>, phase: f32) -> f32 {
    let sw = sin(phase);
    let cw = cos(phase);
    var d = sd_ell(p, vec2(-0.35, 0.0), vec2(0.58, 0.27), 0.0);
    d = min(d, length(p - vec2(0.30, -0.14)) - 0.26);
    d = min(d, sd_tri(p, vec2(0.14, -0.30), vec2(0.20, -0.64), vec2(0.38, -0.38)));
    d = min(d, sd_tri(p, vec2(0.30, -0.36), vec2(0.46, -0.62), vec2(0.50, -0.22)));
    d = min(d, sd_seg(p, vec2(0.12, 0.14), vec2(0.52 + 0.22 * sw, 0.46 - 0.06 * cw)) - 0.075);
    d = min(d, sd_seg(p, vec2(0.0, 0.16), vec2(0.20 - 0.22 * sw, 0.46 + 0.06 * cw)) - 0.075);
    d = min(d, sd_seg(p, vec2(-0.70, 0.12), vec2(-1.02 - 0.22 * sw, 0.42 - 0.06 * cw)) - 0.08);
    d = min(d, sd_seg(p, vec2(-0.58, 0.14), vec2(-0.78 + 0.22 * sw, 0.46 + 0.06 * cw)) - 0.08);
    d = min(d, sd_seg(p, vec2(-0.90, -0.06), vec2(-1.22, -0.28)) - 0.06);
    d = min(d, sd_seg(p, vec2(-1.22, -0.28), vec2(-1.32, -0.66)) - 0.055);
    return d;
}

fn sd_cat(p: vec2<f32>) -> f32 {
    let head = length(p * vec2(1.0, 1.05)) - 1.0;
    let l = sd_tri(p, vec2(-0.98, -0.2), vec2(-0.85, -1.15), vec2(-0.2, -0.85));
    let r = sd_tri(p, vec2(0.98, -0.2), vec2(0.85, -1.15), vec2(0.2, -0.85));
    return min(head, min(l, r));
}

// ---------------------------------------------------------------- transitions

// 1 - Claw Wipe: six curved scratches rip the picture open, centre ones first.
fn claw_wipe(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let ang = 0.52;
    let d = vec2(cos(ang), sin(ang));
    let n = vec2(-d.y, d.x);
    let along = dot(q, d);
    let across = dot(q, n);
    let px = 1.5 / u.res.y;
    var inside = -1.0;
    for (var i = 0; i < 6; i++) {
        let fi = f32(i);
        let c = (fi - 2.5) * 0.30;
        let v = across - c - 0.18 * along * along;
        let delay = abs(fi - 2.5) * 0.09;
        let t = clamp((p - delay) / 0.66, 0.0, 1.0);
        let front = mix(-1.15, 1.35, ease_out(t));
        let taper = sqrt(clamp((front - along) / 0.55, 0.0, 1.0));
        let w = (0.010 + 0.18 * t * t * t) * taper;
        inside = max(inside, w - abs(v));
    }
    let global = smoothstep(0.80, 1.0, p);
    let on = smoothstep(0.0, 0.08, p);       // no shading or rim before the first scratch appears
    let mask = max(smoothstep(-px, px, inside), global);
    let shade = 1.0 - 0.35 * exp(-max(-inside, 0.0) * 14.0) * (1.0 - global) * on;
    var col = mix(sa(uv) * shade, sb(uv), mask);
    let rim = exp(-sq(inside / 0.010)) * (1.0 - global) * on;
    col = mix(col, vec3(1.0, 0.88, 0.92), rim * 0.9);
    let core = exp(-sq((inside - 0.012) / 0.0035)) * (1.0 - global) * on;
    col = mix(col, BROWN, core * 0.55);
    return col;
}

// 2 - Furry Blur: hair-like jittered taps, picture changes along fur strands.
fn furry_blur(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let m = sin(PI * p);
    let radius = 0.05 * m;
    var acc = vec3(0.0);
    for (var i = 0; i < 14; i++) {
        let fi = f32(i);
        let h = hash21(vec2(fi, 3.7));
        let fur = vnoise(uv * vec2(7.0, 5.0) + fi * 0.13);
        let a = fur * 6.2831 + h * 1.5;
        let dir = vec2(cos(a), sin(a));
        let len = (hash21(vec2(fi, 9.1)) - 0.5) * 2.0 * radius;
        let su = uv + dir * len * vec2(1.0 / aspect(), 1.0);
        let strand = vnoise(su * vec2(90.0, 14.0) + vec2(fi * 3.1, 0.0));
        let w = clamp(p * 2.0 - 0.5 + (strand - 0.5) * 0.9, 0.0, 1.0);
        acc += mix(sa(su), sb(su), ease_io(w));
    }
    var col = acc / 14.0;
    col = mix(col, col * vec3(1.04, 0.98, 0.96) + vec3(1.0, 0.88, 0.82) * 0.10, m);
    return col;
}

// 3 - Paw Spin: one huge paw spins out of the centre; B lives inside it.
fn paw_spin(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let e = ease_io(p);
    let s = 0.03 + 1.45 * pow(e, 1.6);
    let ang = e * 7.85;
    var d = sd_paw(rot(-ang) * q / s) * s;
    d = min(d, length(q) - 1.4 * smoothstep(0.78, 1.0, p));
    let r = length(q);
    let sw = ang * 0.25 * exp(-r * r * 5.0) * (1.0 - smoothstep(0.7, 1.0, p));
    let colA = sa(clamp(from_centered(rot(sw) * q), vec2(0.0), vec2(1.0)));
    let colB = sb(uv);
    let px = 1.5 / u.res.y;
    let mask = smoothstep(px, -px, d);
    var col = mix(colA, colB, mask);
    let glow = exp(-max(d, 0.0) * 18.0) * (1.0 - mask);
    col = mix(col * (1.0 - 0.25 * glow), PINK, glow * 0.25);
    let fade = 1.0 - smoothstep(0.85, 1.0, p);
    col = mix(col, BROWN, smoothstep(0.012, 0.004, abs(d)) * 0.85 * fade);
    col = mix(col, PINK, smoothstep(0.06, 0.0, -d) * mask * 0.28 * fade);
    return col;
}

// 4 - Gentle Paw Fade: a trail of paw prints taps over the image, each one
// opens a little window onto B before a soft crossfade finishes the job.
fn gentle_paw_fade(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    var qg = q / 0.2;
    let row = floor(qg.y);
    qg.x += 0.5 * (row - 2.0 * floor(row * 0.5));
    let id = vec2(floor(qg.x), row);
    let f = vec2(fract(qg.x), fract(qg.y)) - 0.5;
    let h = hash21(id);
    let h2 = hash21(id + 17.0);
    let sweep = clamp(dot(uv, vec2(0.65, 0.35)), 0.0, 1.0);
    let t0 = 0.04 + 0.5 * (0.7 * sweep + 0.3 * h);
    let local = clamp((p - t0) / 0.28, 0.0, 1.0);
    let sc = 0.43 * ease_out(local);
    let ang = 2.0 + (h2 - 0.5) * 0.7;
    var d = 1.0;
    if local > 0.0 {
        d = sd_paw(rot(-ang) * f / sc) * sc;
    }
    let print = smoothstep(0.04, -0.04, d);
    let ov = smoothstep(0.62, 1.0, p);
    let w = max(print, ov);
    var colB = sb(uv);
    colB = mix(colB, PINK, print * (1.0 - local * 0.7) * (1.0 - ov) * 0.35);
    colB = mix(colB, BROWN, smoothstep(0.07, 0.0, abs(d)) * (1.0 - ov) * 0.22);
    return mix(sa(uv), colB, w);
}

// 5 - Kitty Dash: a cat sprints left to right, leaving paw prints and speed lines.
fn kitty_dash(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let e = ease_io(p);
    let xf = mix(-0.2, 1.2, e);
    let dist = uv.x - xf;
    let speed = sin(PI * p);
    let br = 0.10 * speed * exp(-abs(dist) * 5.0);
    var acc = vec3(0.0);
    for (var i = 0; i < 12; i++) {
        let t = f32(i) / 11.0 - 0.5;
        let su = uv + vec2(t * br, 0.0);
        acc += mix(sa(su), sb(su), smoothstep(0.004, -0.004, su.x - xf));
    }
    var col = acc / 12.0;

    let rows = 46.0;
    let rid = floor(uv.y * rows);
    let rh = hash21(vec2(rid, 5.0));
    let behind = xf - uv.x;
    let streak = select(0.0, 1.0, abs(fract(uv.y * rows) - 0.5) < 0.12)
        * step(0.0, behind) * smoothstep(0.12 + 0.55 * rh, 0.0, behind) * step(0.45, rh) * speed;
    col = mix(col, vec3(1.0, 0.94, 0.96), streak * 0.55);
    col = mix(col, vec3(1.0, 0.86, 0.92), exp(-sq(dist / 0.012)) * speed * 0.8);

    let a = aspect();
    let sp = 0.19;
    let qx = (uv.x - 0.5) * a;
    let qy = uv.y - 0.5;
    let k0 = floor(qx / sp);
    var prints = 0.0;
    for (var j = -1; j <= 1; j++) {
        let k = k0 + f32(j);
        let cx = (k + 0.5) * sp;
        let age = xf - (cx / a + 0.5);
        if age > 0.02 {
            let lane = select(-0.13, 0.13, (k - 2.0 * floor(k * 0.5)) > 0.5);
            let s = 0.075;
            let d = sd_paw(rot(-1.5708) * vec2(qx - cx, qy - lane) / s) * s;
            prints = max(prints, smoothstep(0.004, -0.004, d) * exp(-age * 3.2));
        }
    }
    col = mix(col, vec3(1.0, 0.62, 0.74), prints * 0.8);

    // the cat itself, ahead of the colour change
    let cs = 0.17;
    let cq = (vec2(qx - (xf - 0.5) * a + 0.10, qy - 0.02 + 0.025 * abs(sin(p * 46.0))) ) / cs;
    let cd = sd_runner(cq, p * 46.0) * cs;
    let body = smoothstep(0.004, -0.004, cd);
    let eye = smoothstep(0.004, -0.004, (length(cq - vec2(0.38, -0.18)) - 0.05) * cs);
    col = mix(col, BROWN, body * 0.95 * step(0.02, speed));
    col = mix(col, CREAM, eye * body * step(0.02, speed));
    return col;
}

// 6 - Meow Glitch: torn scanlines, RGB split, block swaps and neon cat faces.
fn meow_glitch(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let g = sin(PI * p);
    let tq = floor(u.time * 20.0);
    let rowh = floor(uv.y * 28.0 + hash11(tq) * 5.0);
    let hr = hash21(vec2(rowh, tq));
    let shift = (hash21(vec2(rowh, tq + 7.0)) - 0.5) * 0.22 * g * step(0.62, hr);
    let split = 0.014 * g * (0.5 + hash11(tq + 3.0));
    let blk = vec2(floor(uv.x * 12.0), floor(uv.y * 7.0));
    let hb = hash21(blk + 3.0) * 0.85 + hash21(blk + floor(u.time * 10.0) * 0.31) * 0.15;
    let use_b = step(hb, p * 1.15 - 0.08);
    let su = uv + vec2(shift, 0.0);
    let ca = vec3(sa(su + vec2(split, 0.0)).r, sa(su).g, sa(su - vec2(split, 0.0)).b);
    let cb = vec3(sb(su + vec2(split, 0.0)).r, sb(su).g, sb(su - vec2(split, 0.0)).b);
    var col = mix(ca, cb, use_b);
    col *= 1.0 - 0.10 * g * (0.5 + 0.5 * sin(uv.y * u.res.y * 1.2));
    col += (hash21(uv * u.res + tq) - 0.5) * 0.08 * g;
    let q = centered(uv);
    for (var i = 0; i < 4; i++) {
        let seed = vec2(f32(i), floor(u.time * 7.0));
        let on = step(0.45, hash21(seed + 2.0)) * step(0.15, g);
        let c = vec2((hash21(seed) - 0.5) * aspect() * 0.9, (hash21(seed + 5.0) - 0.5) * 0.8);
        let s = 0.05 + 0.09 * hash21(seed + 9.0);
        let pq = (q - c) / s;
        let eyes = min(length(pq - vec2(-0.38, -0.02)), length(pq - vec2(0.38, -0.02))) - 0.17;
        let inside = step(sd_cat(pq), 0.0) * step(0.0, eyes) * on;
        let neon = select(vec3(1.0, 0.35, 0.75), vec3(0.3, 0.95, 1.0), hash21(seed + 11.0) > 0.5);
        col = mix(col, neon, inside * 0.9);
    }
    return col;
}

// ---------------------------------------------------------------- classic blends

fn dip(uv: vec2<f32>, p: f32, c: vec3<f32>) -> vec3<f32> {
    let e = ease_io(p);
    if e < 0.5 { return mix(sa(uv), c, e * 2.0); }
    return mix(c, sb(uv), e * 2.0 - 1.0);
}

fn wipe(uv: vec2<f32>, p: f32, down: bool) -> vec3<f32> {
    let e = ease_io(p);
    let soft = 0.10;
    let edge = e * (1.0 + soft);
    let v = select(uv.x, uv.y, down);
    let m = 1.0 - smoothstep(edge - soft, edge, v);
    return mix(sa(uv), sb(uv), m);
}

fn push(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let e = ease_io(p);
    let x = uv.x + e;
    if x < 1.0 { return sa(vec2(x, uv.y)); }
    return sb(vec2(x - 1.0, uv.y));
}

fn zoom(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let e = ease_io(p);
    let c = vec2(0.5);
    let a = sa(clamp(c + (uv - c) / (1.0 + e * 0.6), vec2(0.0), vec2(1.0)));
    let b = sb(clamp(c + (uv - c) / (1.4 - 0.4 * e), vec2(0.0), vec2(1.0)));
    return mix(a, b, smoothstep(0.2, 0.8, p));
}

fn iris(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let maxr = length(vec2(aspect() * 0.5, 0.5)) * 1.05;
    let radius = ease_io(p) * maxr;
    let m = 1.0 - smoothstep(radius - 0.015, radius + 0.015, length(q));
    return mix(sa(uv), sb(uv), m);
}

fn clock(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    var a = atan2(q.x, -q.y);
    if a < 0.0 { a += 2.0 * PI; }
    let thr = ease_io(p) * 2.0 * PI;
    let m = 1.0 - smoothstep(thr - 0.04, thr + 0.04, a);
    return mix(sa(uv), sb(uv), m);
}

fn barn(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let d = abs(uv.x - 0.5) * 2.0;
    let e = ease_io(p) * 1.05;
    let m = 1.0 - smoothstep(e - 0.03, e, d);
    return mix(sa(uv), sb(uv), m);
}

fn pixelate(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let block = mix(1.0, 56.0, sin(PI * p));
    let bu = vec2(block) / u.res;
    let c = (floor(uv / bu) + 0.5) * bu;
    return mix(sa(c), sb(c), ease_io(p));
}

fn whirl(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let r = length(q);
    let ang = sin(PI * p) * 5.0 * exp(-r * 2.2);
    let s = clamp(from_centered(rot(ang) * q), vec2(0.0), vec2(1.0));
    return mix(sa(s), sb(s), ease_io(p));
}

fn blur_dissolve(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let r = sin(PI * p) * 0.03;
    var a = sa(uv);
    var b = sb(uv);
    for (var i = 0; i < 12; i++) {
        let ang = f32(i) * 0.5236;
        let ring = select(0.5, 1.0, i % 2 == 0);
        let off = vec2(cos(ang), sin(ang)) * r * ring * vec2(1.0 / aspect(), 1.0);
        a += sa(uv + off);
        b += sb(uv + off);
    }
    return mix(a / 13.0, b / 13.0, ease_io(p));
}

// ================================================================ version 1.2 blends
// Noise and cells, waves and optics, curves, manga look, 3D. Same rules as above: pure maths,
// no textures other than the two pictures.

fn hash22(p: vec2<f32>) -> vec2<f32> {
    let q = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(q) * 43758.5453);
}
// fractal noise, 0..1
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
fn luma(c: vec3<f32>) -> f32 { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
fn cl(uv: vec2<f32>) -> vec2<f32> { return clamp(uv, vec2(0.0), vec2(1.0)); }
fn max_radius() -> f32 { return length(vec2(aspect() * 0.5, 0.5)); }

struct Vor {
    f1: f32,
    f2: f32,
    id: vec2<f32>,
    seed: vec2<f32>,   // position of the nearest seed point, in the same space as the input
}

fn voronoi(p: vec2<f32>) -> Vor {
    let ip = floor(p);
    let fp = fract(p);
    var v: Vor;
    v.f1 = 8.0;
    v.f2 = 8.0;
    v.id = vec2(0.0);
    v.seed = p;
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let g = vec2(f32(i), f32(j));
            let r = g + hash22(ip + g) - fp;
            let d = length(r);
            if d < v.f1 {
                v.f2 = v.f1;
                v.f1 = d;
                v.id = ip + g;
                v.seed = p + r;
            } else if d < v.f2 {
                v.f2 = d;
            }
        }
    }
    return v;
}

// ---------------------------------------------------------------- noise and cells

// 19 - Paper burn: fractal noise against progress, a glowing rim at the threshold.
fn burn(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let n = fbm(q * 3.4 + vec2(3.1, 1.7));
    let thr = mix(-0.12, 1.12, p);
    let d = n - thr;                       // above 0: still the old picture
    let a = sa(uv);
    let b = sb(uv);
    let still = smoothstep(-0.004, 0.004, d);
    let charred = 1.0 - smoothstep(0.0, 0.09, d);
    var col_a = mix(a, vec3(0.10, 0.04, 0.02), charred * 0.9);
    let glow = 1.0 - smoothstep(0.0, 0.03, d);
    col_a = mix(col_a, mix(vec3(1.0, 0.35, 0.05), vec3(1.0, 0.9, 0.55), glow * glow), glow);
    let ember = 1.0 - smoothstep(0.0, 0.05, -d);
    let col_b = b + vec3(1.0, 0.45, 0.12) * ember * 0.6;
    return mix(col_b, col_a, still);
}

// 20 - Voronoi shatter: every cell tips away on its own schedule, the cell edges stay as lines.
fn shatter(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let scale = 9.0;
    let v = voronoi(q * scale);
    let seed_world = v.seed / scale;
    let delay = hash21(v.id) * 0.5 + length(seed_world) * 0.12;
    let t = clamp((p - delay) / 0.42, 0.0, 1.0);
    let te = t * t;
    // the cell is eroded from its borders inwards, so the shards stay polygons
    let alive = smoothstep(te * 1.25, te * 1.25 + 0.05, v.f2 - v.f1);
    let ang = (hash21(v.id + vec2(7.0, 3.0)) - 0.5) * 2.4 * te;
    let rel = q - seed_world;
    let src = seed_world + rot(ang) * rel / max(1.0 - te * 0.8, 0.2);
    let a = sa(cl(from_centered(src)));
    var col = mix(sb(uv), a, alive);
    let line = 1.0 - smoothstep(0.0, 0.07, v.f2 - v.f1);
    let amount = smoothstep(0.0, 0.08, p) * (1.0 - smoothstep(0.82, 1.0, p));
    return mix(col, vec3(0.96, 0.98, 1.0), line * 0.85 * amount);
}

// 21 - Domain warping: noise bends the picture before it turns into the next, like smoke or ink.
fn domain_warp(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let s = sin(PI * p);
    let n = vec2(fbm(q * 2.6 + vec2(p * 1.7, 0.0)), fbm(q * 2.6 + vec2(5.2, 1.3 + p * 1.7)));
    let off = (n - 0.5) * 0.7 * s;
    let a = sa(cl(from_centered(q + off)));
    let b = sb(cl(from_centered(q - off * 0.8)));
    let m = smoothstep(0.30, 0.70, p + (fbm(q * 1.8 + vec2(3.0, 8.0)) - 0.5) * 0.4);
    let ink = 1.0 - 0.18 * sin(PI * m);
    return mix(a, b, m) * ink;
}

// ---------------------------------------------------------------- waves and optics

// 22 - Raindrop: a ring of ripples runs out from the middle and brings the new picture.
fn raindrop(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let r = length(q);
    let front = p * max_radius() * 1.3;
    let d = r - front;
    let ring = exp(-sq(d * 11.0));
    let wave = sin(d * 70.0) * ring * 0.022 * (1.0 - p);
    let dir = q / max(r, 1e-4);
    let suv = cl(from_centered(q + dir * wave));
    let m = 1.0 - smoothstep(-0.015, 0.015, d);
    var col = mix(sa(suv), sb(suv), m);
    col += vec3(1.0) * ring * 0.10 * (1.0 - p) * (0.5 + 0.5 * sin(d * 70.0 + 1.0));
    return col;
}

// 23 - Fisheye zoom: a lens bulges the old picture while it zooms in, the new one unwinds out of it.
fn fisheye(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let e = ease_io(p);
    let k = 5.0 * sin(PI * p);
    let lens = 1.0 + k * dot(q, q);
    let a = sa(cl(from_centered(q * lens / (1.0 + 3.0 * e))));
    let b = sb(cl(from_centered(q * lens / (4.0 - 3.0 * e))));
    return mix(a, b, smoothstep(0.35, 0.65, p));
}

// 24 - Zoom blur: samples along the line to the centre, weighted towards the outside.
fn zoom_blur(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let s = sin(PI * p) * 0.5;
    var a = vec3(0.0);
    var b = vec3(0.0);
    var wsum = 0.0;
    let jit = hash21(uv * u.res) / 24.0;
    for (var i = 0; i < 24; i++) {
        let t = clamp(f32(i) / 23.0 + jit, 0.0, 1.0);
        let w = 1.0 + t * 1.5;
        let suv = cl(vec2(0.5) + (uv - vec2(0.5)) * (1.0 - s * t * 0.55));
        a += sa(suv) * w;
        b += sb(suv) * w;
        wsum += w;
    }
    return mix(a / wsum, b / wsum, ease_io(p));
}

// 25 - Spin blur: the same, but around the centre.
fn spin_blur(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let s = sin(PI * p) * 0.9;
    var a = vec3(0.0);
    var b = vec3(0.0);
    let jit = hash21(uv * u.res) / 32.0;
    for (var i = 0; i < 32; i++) {
        let t = f32(i) / 31.0 + jit - 0.5;
        let suv = cl(from_centered(rot(s * t) * q));
        a += sa(suv);
        b += sb(suv);
    }
    return mix(a / 32.0, b / 32.0, ease_io(p));
}

// 26 - Chromatic aberration: red, green and blue drift apart radially while the picture changes.
fn aberration(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let amt = sin(PI * p) * 0.07;
    let e = ease_io(p);
    let ur = cl(from_centered(q * (1.0 + amt)));
    let ug = cl(from_centered(q));
    let ub = cl(from_centered(q * (1.0 - amt)));
    return vec3(
        mix(sa(ur).r, sb(ur).r, e),
        mix(sa(ug).g, sb(ug).g, e),
        mix(sa(ub).b, sb(ub).b, e),
    );
}

// 27 - Light leak: a diagonal Gaussian stripe sweeps through and blows out the cut additively.
fn light_leak(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let d = normalize(vec2(0.82, -0.57));
    let along = dot(q, d);
    let ext = aspect() * 0.5 * abs(d.x) + 0.5 * abs(d.y);
    let c = mix(-ext - 0.5, ext + 0.5, ease_io(p));
    let g = exp(-sq((along - c) / 0.30));
    let core = exp(-sq((along - c + 0.28) / 0.14));
    let across = dot(q, vec2(-d.y, d.x));
    let streak = 0.8 + 0.4 * vnoise(vec2(across * 5.0, p * 3.0));
    let tint = mix(vec3(1.0, 0.38, 0.12), vec3(1.0, 0.82, 0.45), core);
    let m = smoothstep(-0.12, 0.12, c - along);
    return mix(sa(uv), sb(uv), m) + tint * (g * 0.5 + core * 0.4) * streak;
}

// ---------------------------------------------------------------- curves and geometry

// 28 - Sakura wipe: a five-petal blossom, r = |cos(5/2 theta)|, opens the new picture.
fn sakura(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let r = length(q);
    let th = atan2(q.y, q.x) + p * 1.2;
    let k = abs(cos(2.5 * th));
    let petal = (0.28 + 0.72 * k) * (1.0 - 0.10 * exp(-sq((1.0 - k) * 14.0)));
    let e = ease_io(p);
    let radius = e * e * max_radius() * 3.9 * petal;
    let m = 1.0 - smoothstep(-0.012, 0.012, r - radius);
    let edge = exp(-sq((r - radius) / 0.02));
    return mix(mix(sa(uv), sb(uv), m), PINK, edge * 0.8);
}

// 29 - Heart iris: the implicit heart (x^2+y^2-1)^3 - x^2 y^3 = 0 as an opening mask.
fn heart_iris(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let s = max(pow(ease_io(p), 1.5) * max_radius() * 1.7, 1e-4);
    let x = q.x / s;
    let y = -q.y / s + 0.35;
    var d = (length(vec2(x, y)) - 1.3) * s;     // far outside: plain distance (and no overflow)
    if x * x + y * y < 9.0 {
        let a = x * x + y * y - 1.0;
        let f = a * a * a - x * x * y * y * y;
        let gx = 6.0 * x * a * a - 2.0 * x * y * y * y;
        let gy = 6.0 * y * a * a - 3.0 * x * x * y * y;
        d = f / max(length(vec2(gx, gy)), 1e-3) * s;
    }
    var m = 1.0 - smoothstep(-0.006, 0.006, d);
    m = max(m, smoothstep(0.96, 1.0, p));
    let edge = exp(-sq(d / 0.014)) * (1.0 - smoothstep(0.9, 1.0, p));
    return mix(mix(sa(uv), sb(uv), m), vec3(1.0, 0.55, 0.7), edge * 0.85);
}

// 30 - Spiral wipe: arms of an Archimedean spiral (r = a theta) grow until they close up.
fn spiral(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let r = length(q) / max_radius();
    let th = atan2(q.y, q.x);
    let key = fract(th / (2.0 * PI) - 2.2 * r - p * 0.35);
    let thr = ease_io(p) * 1.08;
    let w = 0.012;
    var m = smoothstep(0.0, w, key) * (1.0 - smoothstep(thr - w * 1.5, thr, key));
    m = max(m, smoothstep(0.96, 1.0, p));
    let edge = (1.0 - smoothstep(0.0, 0.012, abs(key - thr))) * (1.0 - smoothstep(0.9, 1.0, p));
    return mix(mix(sa(uv), sb(uv), m), PINK, edge * 0.7);
}

// 31 - Paw bloom: phyllotaxis, points at the golden angle like a sunflower head. Every point
// grows a paw, one after the other from the middle outwards.
fn paw_bloom(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let count = 90.0;
    let c = max_radius() * 1.04 / sqrt(count);
    var cover = 0.0;
    var rim = 0.0;
    for (var i = 0; i < 90; i++) {
        let fi = f32(i);
        let t = clamp((p - fi / count * 0.68) / 0.30, 0.0, 1.0);
        if t <= 0.0 { continue; }
        let rr = c * sqrt(fi + 0.5);
        let th = fi * 2.39996323;
        let centre = vec2(cos(th), sin(th)) * rr;
        let size = max(c * 1.2 * ease_out(t), 1e-4);
        let local = (q - centre) / size;
        if dot(local, local) > 4.0 { continue; }
        let tilt = (hash11(fi + 1.3) - 0.5) * 1.2;
        let dd = sd_paw(rot(tilt) * local) * size;
        cover = max(cover, 1.0 - smoothstep(-0.004, 0.004, dd));
        rim = max(rim, exp(-sq(dd / 0.012)));
    }
    let fin = smoothstep(0.94, 1.0, p);
    let m = max(cover, fin);
    return mix(mix(sa(uv), sb(uv), m), PINK, rim * (1.0 - fin) * 0.7);
}

// pointy-top hexagon, inradius r
fn hex_sd(p: vec2<f32>, r: f32) -> f32 {
    let k = vec3(-0.866025404, 0.5, 0.577350269);
    var q = abs(p.yx);
    q -= 2.0 * min(dot(k.xy, q), 0.0) * k.xy;
    q -= vec2(clamp(q.x, -k.z * r, k.z * r), r);
    return length(q) * sign(q.y);
}

// 32 - Hex grid wipe: hexagonal tiles flip over one after the other, distance from the middle
// is the delay.
fn hex_wipe(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let s = 0.075;
    let fq = (0.57735027 * q.x - 0.33333333 * q.y) / s;
    let fr = (0.66666667 * q.y) / s;
    var rq = round(fq);
    var rr = round(fr);
    let rs = round(-fq - fr);
    let dq = abs(rq - fq);
    let dr = abs(rr - fr);
    let ds = abs(rs - (-fq - fr));
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    let centre = vec2(s * 1.7320508 * (rq + rr * 0.5), s * 1.5 * rr);
    let v = q - centre;
    let dist = length(centre) / max_radius();
    let t = clamp((p - dist * 0.55) / 0.45, 0.0, 1.0);
    let ang = ease_io(t) * PI;
    let sx = max(abs(cos(ang)), 0.02);
    let u = vec2(v.x / sx, v.y);
    let shrink = 1.0 - 0.07 * sin(PI * t);
    let inside = hex_sd(u, s * 0.866 * shrink) < 0.0005;
    let face = from_centered(centre + u);
    var col = mix(sa(uv), sb(uv), 0.5) * 0.45;
    if inside {
        col = select(sb(cl(face)), sa(cl(face)), ang < 0.5 * PI) * (0.55 + 0.45 * sx);
    }
    return col;
}

fn kaleido_q(q: vec2<f32>, n: f32, spin: f32) -> vec2<f32> {
    let r = length(q);
    let seg = 2.0 * PI / n;
    var a = atan2(q.y, q.x) + spin;
    a = a - seg * floor(a / seg);
    a = abs(a - seg * 0.5);
    return vec2(cos(a), sin(a)) * r;
}

// 33 - Kaleidoscope: the angle is folded with mod(theta, 2 pi / n) and mirrored, n runs 1..8.
fn kaleido(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let s = sin(PI * p);
    let nf = 1.0 + 7.0 * s;
    let n0 = floor(nf);
    let fr = fract(nf);
    let spin = p * 1.6;
    let strength = smoothstep(0.0, 0.25, s);
    let k0 = cl(from_centered(mix(q, kaleido_q(q, n0, spin), strength)));
    let k1 = cl(from_centered(mix(q, kaleido_q(q, n0 + 1.0, spin), strength)));
    let ca = mix(sa(k0), sa(k1), fr);
    let cb = mix(sb(k0), sb(k1), fr);
    return mix(ca, cb, smoothstep(0.4, 0.6, p));
}

// ---------------------------------------------------------------- manga look

// 34 - Halftone: a screen of dots turned by 45 degrees, dots grow with progress and darkness.
fn halftone(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let cell = 0.026;
    let g = rot(0.7853982) * q / cell;
    let id = floor(g);
    let f = fract(g) - vec2(0.5);
    let cuv = cl(from_centered(rot(-0.7853982) * ((id + vec2(0.5)) * cell)));
    let lum = luma(sb(cuv));
    let grow = clamp(ease_io(p) * 1.5 - lum * 0.5, 0.0, 1.0);
    let rad = 0.78 * grow;
    let d = length(f) - rad;
    let m = (1.0 - smoothstep(-0.03, 0.03, d)) * smoothstep(0.0, 0.05, rad);
    let ink = exp(-sq(d / 0.06)) * smoothstep(0.0, 0.05, rad);
    return mix(sa(uv), sb(uv), m) * (1.0 - 0.3 * ink);
}

// 8x8 Bayer matrix without a lookup table
fn bayer8(c: vec2<u32>) -> f32 {
    var v = 0u;
    for (var k = 2; k >= 0; k--) {
        let xb = (c.x >> u32(k)) & 1u;
        let yb = (c.y >> u32(k)) & 1u;
        v = v * 4u + ((xb ^ yb) * 2u + yb);
    }
    return (f32(v) + 0.5) / 64.0;
}

// 35 - Bayer dither dissolve: the matrix is the threshold, big pixels give the retro look.
fn bayer_dissolve(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let block = max(1.0, floor(u.res.y / 180.0));
    let px = floor(uv * u.res / block);
    let thr = bayer8(vec2<u32>(u32(px.x), u32(px.y)));
    let suv = cl((px + vec2(0.5)) * block / u.res);
    let m = select(0.0, 1.0, thr < ease_io(p));
    let c = mix(sa(suv), sb(suv), m);
    let post = floor(c * 5.0 + vec3(0.5)) / 5.0;
    return mix(c, post, sin(PI * p));
}

// 36 - Tabby stripes: sine stripes bent by noise, used as the wipe mask.
fn tabby(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let d = normalize(vec2(0.35, 1.0));
    let warp = (fbm(q * 2.2 + vec2(4.0, 2.0)) - 0.5) * 0.55;
    let v = 0.5 + 0.5 * sin((dot(q, d) * 7.0 + warp * 6.0) * 2.0 * PI);
    let ext = aspect() * 0.5 * abs(vec2(-d.y, d.x).x) + 0.5 * abs(vec2(-d.y, d.x).y);
    let a2 = clamp(dot(q, vec2(-d.y, d.x)) / (2.0 * ext) + 0.5, 0.0, 1.0);
    let thr = ease_io(p) * 1.75 - 0.25 - 0.5 * a2;
    let m = 1.0 - smoothstep(thr - 0.05, thr, v);
    let edge = exp(-sq((v - thr) / 0.05));
    return mix(mix(sa(uv), sb(uv), m), vec3(0.30, 0.17, 0.09), edge * 0.7);
}

// 37 - Calico: Voronoi blotches flip one by one, tinted orange, black and white at the front.
fn calico(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let n = fbm(q * 3.0 + vec2(1.7, 4.1)) - 0.5;
    let v = voronoi(q * 4.2 + vec2(n * 0.9));
    let h = hash21(v.id);
    let thr = mix(-0.2, 1.2, ease_io(p));
    let m = 1.0 - smoothstep(thr - 0.02, thr, h);
    let near = exp(-sq((h - thr) / 0.07));
    let pick = hash11(h * 97.0);
    var tint = vec3(1.0, 0.62, 0.25);
    if pick < 0.33 {
        tint = vec3(0.12, 0.09, 0.08);
    } else if pick < 0.66 {
        tint = vec3(1.0, 0.97, 0.92);
    }
    var col = mix(sa(uv), sb(uv), m);
    col = mix(col, tint, near * 0.75);
    let seam = 1.0 - smoothstep(0.0, 0.05, v.f2 - v.f1);
    return mix(col, vec3(0.22, 0.14, 0.10), seam * 0.35 * sin(PI * p));
}

// ---------------------------------------------------------------- 3D and physics

// 38 - Page curl: the old picture rolls up around a cylinder and shows the new one beneath.
fn page_curl(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let d = normalize(vec2(1.0, 0.55));
    let s = dot(q, d);
    let ext = aspect() * 0.5 * d.x + 0.5 * d.y;
    let radius = 0.07 + 0.05 * sin(PI * p);
    let f = mix(ext + radius * 0.5, -ext - radius * 1.5, ease_io(p));
    if s < f {
        let shade = 1.0 - exp(-(f - s) * 10.0) * 0.35;
        return sa(uv) * shade;
    }
    if s > f + radius {
        let shadow = exp(-(s - f - radius) * 9.0) * 0.45;
        return sb(uv) * (1.0 - shadow);
    }
    let t = (s - f) / radius;
    let u2 = f + radius * (PI - asin(clamp(t, 0.0, 1.0)));
    let src = q + d * (u2 - s);
    let back = mix(sa(cl(from_centered(src))) * 0.9, vec3(0.95, 0.93, 0.90), 0.55);
    let shade = 0.55 + 0.45 * (1.0 - t);
    let shine = exp(-sq((t - 0.55) / 0.12)) * 0.18;
    return back * shade + vec3(shine);
}

// 39 - Cube: perspective projection with a rotation matrix. Face A turns away, face B comes in.
fn cube(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let a = aspect();
    let x = (uv.x - 0.5) * 2.0 * a;
    let y = (0.5 - uv.y) * 2.0;
    let cam = vec3(0.0, 0.0, a + 2.4);
    let dir = vec3(x, y, a) - cam;
    let ph = ease_io(p) * 1.5707963;
    let cs = cos(ph);
    let sn = sin(ph);
    var col = mix(sa(uv), sb(uv), 0.5) * 0.25;
    var best = 1e9;
    let na = vec3(-sn, 0.0, cs);
    let den_a = dot(na, dir);
    if den_a < -1e-4 {
        let t = (a - dot(na, cam)) / den_a;
        let pt = cam + dir * t;
        let xl = pt.x * cs + pt.z * sn;
        if t > 0.0 && abs(xl) <= a && abs(pt.y) <= 1.0 {
            best = t;
            col = sa(cl(vec2(xl / (2.0 * a) + 0.5, 0.5 - pt.y * 0.5))) * mix(1.0, 0.5, ph / 1.5707963);
        }
    }
    let nb = vec3(cs, 0.0, sn);
    let den_b = dot(nb, dir);
    if den_b < -1e-4 {
        let t = (a - dot(nb, cam)) / den_b;
        let pt = cam + dir * t;
        let zl = -pt.x * sn + pt.z * cs;
        if t > 0.0 && t < best && abs(zl) <= a && abs(pt.y) <= 1.0 {
            col = sb(cl(vec2((a - zl) / (2.0 * a), 0.5 - pt.y * 0.5))) * mix(0.5, 1.0, ph / 1.5707963);
        }
    }
    return col;
}

// 40 - Flip: the picture turns over like a card, the back side shows the new one.
fn flip(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let a = aspect();
    let x = (uv.x - 0.5) * 2.0 * a;
    let y = (0.5 - uv.y) * 2.0;
    let cam = vec3(0.0, 0.0, 3.0);
    let dir = vec3(x, y, 0.0) - cam;
    let ph = ease_io(p) * PI;
    let cs = cos(ph);
    let sn = sin(ph);
    var col = mix(sa(uv), sb(uv), ease_io(p)) * 0.22;
    let n = vec3(-sn, 0.0, cs);
    let den = dot(n, dir);
    if abs(den) > 1e-4 {
        let t = -dot(n, cam) / den;
        let pt = cam + dir * t;
        let xl = pt.x * cs + pt.z * sn;
        if t > 0.0 && abs(xl) <= a && abs(pt.y) <= 1.0 {
            let shade = 0.45 + 0.55 * abs(cs);
            if ph < 0.5 * PI {
                col = sa(cl(vec2(xl / (2.0 * a) + 0.5, 0.5 - pt.y * 0.5))) * shade;
            } else {
                col = sb(cl(vec2(0.5 - xl / (2.0 * a), 0.5 - pt.y * 0.5))) * shade;
            }
        }
    }
    return col;
}

fn melt_offset(i: f32, p: f32) -> f32 {
    let delay = vnoise(vec2(i * 0.6, 1.0)) * 0.30 + hash11(i + 3.7) * 0.15;
    let t = clamp((p - delay) / 0.55, 0.0, 1.0);
    return 1.2 * t * t;                    // y = g t^2 / 2
}

// 41 - Melt / drip: every column falls on its own schedule, neighbours are blended into drips.
fn melt(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let x = uv.x * 40.0;
    let i = floor(x);
    let f = fract(x);
    let w = f * f * (3.0 - 2.0 * f);
    let off = mix(melt_offset(i, p), melt_offset(i + 1.0, p), w);
    let sy = uv.y - off;
    let inside = smoothstep(0.0, 0.004, sy);
    let a = sa(cl(vec2(uv.x, sy)));
    let rim = exp(-sq(sy / 0.02)) * 0.35;
    return mix(sb(uv), a * (1.0 - rim), inside);
}

// 42 - Droste: log-polar mapping (ln r, theta). The picture repeats into itself while it zooms
// and the new picture sits in the middle.
fn droste(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv);
    let e = ease_io(p);
    let ratio = 4.0;
    let r = max(length(q), 1e-4);
    let r0 = max_radius() / ratio;
    let lr = log(r / max_radius()) / log(ratio);
    let u0 = lr - 6.4 * e;
    let n = floor(u0);
    let f = u0 - n;
    let src = rot(0.6 * u0) * (q / r) * (r0 * pow(ratio, f));
    let s = smoothstep(0.0, 0.2, p) * (1.0 - smoothstep(0.8, 1.0, p));
    let a = sa(cl(from_centered(mix(q, src, s))));
    let frame = exp(-sq(f / 0.015)) + exp(-sq((1.0 - f) / 0.015));
    let col_a = a * (1.0 - 0.5 * frame * s);
    let mb = 1.0 - smoothstep(-6.05, -5.95, u0);
    return mix(col_a, sb(uv), mb);
}

// 43 - Julia reveal: the escape time of z^2 + c is the mask, c drifts with time.
fn julia(uv: vec2<f32>, p: f32) -> vec3<f32> {
    let q = centered(uv) * 2.1;
    let ang = 0.5 + p * 2.4;
    let c = 0.7885 * vec2(cos(ang), sin(ang));
    var z = q;
    var it = 0.0;
    for (var i = 0; i < 48; i++) {
        if dot(z, z) > 16.0 { break; }
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        it += 1.0;
    }
    var nu = it;
    if it < 48.0 {
        nu = it + 1.0 - log2(max(0.5 * log2(dot(z, z)), 1e-4));
    }
    let t = clamp(nu / 48.0, 0.0, 1.0);
    let thr = mix(-0.05, 1.08, pow(ease_io(p), 1.8));
    let m = 1.0 - smoothstep(thr - 0.03, thr, t);
    let glow = exp(-sq((t - thr) / 0.04));
    return mix(mix(sa(uv), sb(uv), m), mix(PINK, vec3(0.6, 0.9, 1.0), t), glow * 0.8);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let p = clamp(u.progress, 0.0, 1.0);
    if p <= 0.0 { return vec4(sa(uv), 1.0); }
    if p >= 1.0 { return vec4(sb(uv), 1.0); }
    var col: vec3<f32>;
    switch u.kind {
        case 1u: { col = claw_wipe(uv, p); }
        case 2u: { col = furry_blur(uv, p); }
        case 3u: { col = paw_spin(uv, p); }
        case 4u: { col = gentle_paw_fade(uv, p); }
        case 5u: { col = kitty_dash(uv, p); }
        case 6u: { col = meow_glitch(uv, p); }
        case 7u: { col = dip(uv, p, vec3(0.0)); }
        case 8u: { col = dip(uv, p, vec3(1.0)); }
        case 9u: { col = wipe(uv, p, false); }
        case 10u: { col = wipe(uv, p, true); }
        case 11u: { col = push(uv, p); }
        case 12u: { col = zoom(uv, p); }
        case 13u: { col = iris(uv, p); }
        case 14u: { col = clock(uv, p); }
        case 15u: { col = pixelate(uv, p); }
        case 16u: { col = whirl(uv, p); }
        case 17u: { col = blur_dissolve(uv, p); }
        case 18u: { col = barn(uv, p); }
        case 19u: { col = burn(uv, p); }
        case 20u: { col = shatter(uv, p); }
        case 21u: { col = domain_warp(uv, p); }
        case 22u: { col = raindrop(uv, p); }
        case 23u: { col = fisheye(uv, p); }
        case 24u: { col = zoom_blur(uv, p); }
        case 25u: { col = spin_blur(uv, p); }
        case 26u: { col = aberration(uv, p); }
        case 27u: { col = light_leak(uv, p); }
        case 28u: { col = sakura(uv, p); }
        case 29u: { col = heart_iris(uv, p); }
        case 30u: { col = spiral(uv, p); }
        case 31u: { col = paw_bloom(uv, p); }
        case 32u: { col = hex_wipe(uv, p); }
        case 33u: { col = kaleido(uv, p); }
        case 34u: { col = halftone(uv, p); }
        case 35u: { col = bayer_dissolve(uv, p); }
        case 36u: { col = tabby(uv, p); }
        case 37u: { col = calico(uv, p); }
        case 38u: { col = page_curl(uv, p); }
        case 39u: { col = cube(uv, p); }
        case 40u: { col = flip(uv, p); }
        case 41u: { col = melt(uv, p); }
        case 42u: { col = droste(uv, p); }
        case 43u: { col = julia(uv, p); }
        default: { col = mix(sa(uv), sb(uv), ease_io(p)); }
    }
    return vec4(clamp(col, vec3(0.0), vec3(1.0)), 1.0);
}
