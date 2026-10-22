// Draws a rasterised text picture (premultiplied RGBA, frame sized) over the
// current frame, with an animated appearance driven by `progress` (0 hidden, 1 shown).

struct U {
    res: vec2<f32>,
    progress: f32,
    time: f32,
    kind: u32,
    _pad: u32,
    anchor: vec2<f32>,
}

@group(0) @binding(0) var base_tex: texture_2d<f32>;
@group(0) @binding(1) var text_tex: texture_2d<f32>;
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

fn aspect() -> f32 { return u.res.x / u.res.y; }
fn hash21(p: vec2<f32>) -> f32 { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123); }
fn rot(a: f32) -> mat2x2<f32> {
    let c = cos(a);
    let s = sin(a);
    return mat2x2<f32>(c, s, -s, c);
}
fn ease_out(t: f32) -> f32 { let v = 1.0 - t; return 1.0 - v * v * v; }

fn tx(uv: vec2<f32>) -> vec4<f32> {
    let inside = all(uv >= vec2(0.0)) && all(uv <= vec2(1.0));
    return select(vec4(0.0), textureSampleLevel(text_tex, samp, uv, 0.0), inside);
}

fn text_fx(uv: vec2<f32>) -> vec4<f32> {
    let p = clamp(u.progress, 0.0, 1.0);
    let a = aspect();
    switch u.kind {
        case 1u: { return tx(uv) * p; }
        case 2u: {
            // Verpixeln: big blocks that sharpen into the text
            let block = mix(2.0, 90.0, pow(1.0 - p, 1.5));
            let bu = vec2(block) / u.res;
            let c = (floor(uv / bu) + 0.5) * bu;
            let o = bu * 0.28;
            let s = (tx(c + vec2(o.x, o.y)) + tx(c + vec2(-o.x, o.y)) + tx(c + vec2(o.x, -o.y)) + tx(c + vec2(-o.x, -o.y))) * 0.25;
            return s * smoothstep(0.0, 0.3, p);
        }
        case 3u: {
            // Verwirbeln: swirl around the text that unwinds
            let q = (uv - u.anchor) * vec2(a, 1.0);
            let r = length(q);
            let ang = (1.0 - p) * (1.0 - p) * 10.0 * exp(-r * 2.5);
            let s = u.anchor + (rot(ang) * q) / vec2(a, 1.0);
            return tx(s) * smoothstep(0.0, 0.5, p);
        }
        case 4u: {
            var acc = tx(uv);
            let r = (1.0 - p) * 0.03;
            for (var i = 0; i < 12; i++) {
                let ang = f32(i) * 0.5236;
                let ring = select(0.5, 1.0, i % 2 == 0);
                acc += tx(uv + vec2(cos(ang), sin(ang)) * r * ring * vec2(1.0 / a, 1.0));
            }
            return acc / 13.0 * p;
        }
        case 5u: {
            let off = (1.0 - ease_out(p)) * 0.18;
            return tx(uv + vec2(0.0, off)) * p;
        }
        case 6u: {
            let s = mix(0.3, 1.0, ease_out(p));
            return tx(u.anchor + (uv - u.anchor) / s) * p;
        }
        case 7u: {
            let k = 1.0 - p;
            let w = vec2(sin(uv.y * 40.0 + u.time * 10.0) * 0.03 * k, sin(uv.x * 25.0 + u.time * 8.0) * 0.02 * k);
            return tx(uv + w) * p;
        }
        case 8u: {
            let k = 1.0 - p;
            let tq = floor(u.time * 15.0);
            let row = floor(uv.y * 40.0);
            let h = hash21(vec2(row, tq));
            let shift = (h - 0.5) * 0.15 * k * step(0.5, h);
            let split = 0.012 * k;
            let r = tx(uv + vec2(shift + split, 0.0)).r;
            let g = tx(uv + vec2(shift, 0.0)).g;
            let b = tx(uv + vec2(shift - split, 0.0)).b;
            let al = tx(uv + vec2(shift, 0.0)).a;
            let show = step(hash21(vec2(row, 3.0)), p * 1.3);
            return vec4(r, g, b, al) * show;
        }
        default: { return tx(uv); }
    }
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let base = textureSampleLevel(base_tex, samp, in.uv, 0.0).rgb;
    let t = text_fx(in.uv);
    return vec4(base * (1.0 - t.a) + t.rgb, 1.0);
}
