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
fn sa(uv: vec2<f32>) -> vec3<f32> { return sample_src(tex_a, uv, u.a_x, u.a_c); }
fn sb(uv: vec2<f32>) -> vec3<f32> { return sample_src(tex_b, uv, u.b_x, u.b_c); }

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
    let mask = max(smoothstep(-px, px, inside), global);
    let shade = 1.0 - 0.35 * exp(-max(-inside, 0.0) * 14.0) * (1.0 - global);
    var col = mix(sa(uv) * shade, sb(uv), mask);
    let rim = exp(-pow(inside / 0.010, 2.0)) * (1.0 - global);
    col = mix(col, vec3(1.0, 0.88, 0.92), rim * 0.9);
    let core = exp(-pow((inside - 0.012) / 0.0035, 2.0)) * (1.0 - global);
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
    col = mix(col, vec3(1.0, 0.86, 0.92), exp(-pow(dist / 0.012, 2.0)) * speed * 0.8);

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
        default: { col = mix(sa(uv), sb(uv), ease_io(p)); }
    }
    return vec4(clamp(col, vec3(0.0), vec3(1.0)), 1.0);
}
