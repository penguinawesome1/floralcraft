// -- Surface Terrain --
const CONTINENT_FREQ: f32 = 0.0005;
const CONTINENT_OCTAVES: u32 = 7u;

const WARP_FREQ: f32 = 0.00012;
const WARP_STRENGTH: f32 = 500.0;
const WARP_OCTAVES: u32 = 3u;
const WARP_OFFSET_Y: vec2f = vec2f(42.5, 11.8);

const TERRAIN_BASE_Y: f32 = 350.0;
const TERRAIN_HEIGHT_RANGE: f32 = 1200.0;
const TERRAIN_SPREAD: f32 = 50.0;
const TERRAIN_BIAS_CLAMP_RANGE: f32 = 5.0;

// -- Caves --
const CAVE_FREQ: f32 = 0.003;
const CAVE_OCTAVES: u32 = 4u;

const CHEESE_OFFSET: vec3f = vec3f(123.45, 678.90, -456.78);
const SPAGHETTI_OFFSET: vec3f = vec3f(-89.12, 34.56, 789.01);

const SPAGHETTI_FREQ_SCALE: f32 = 0.7;
const SPAGHETTI_TUNNEL_WIDTH_MULT: f32 = 3.0;

const CAVE_CRUST_THICKNESS: f32 = 1.5;

// -- Internal Hash Constants --
const HASH_SINE_SCALE: f32 = 43758.5453123;

const HASH1_DOT: f32 = 127.1;

const HASH2_DOT1: vec2f = vec2f(127.1, 311.7); // or this.. vec2f(12.9898, 78.233)
const HASH2_DOT2: vec2f = vec2f(269.5, 183.3);

const HASH3_DOT1: vec3f = vec3f(127.1, 311.7, 74.7);
const HASH3_DOT2: vec3f = vec3f(269.5, 183.3, 246.1);
const HASH3_DOT3: vec3f = vec3f(113.5, 271.9, 124.6);

fn hash2d_to_1d(p: vec2f) -> f32 {
    let src = bitcast<vec2u>(p);
    var m = src * 1664525u + 1013904223u;
    m.x += m.y * 1664525u;
    m.y += m.x * 1664525u;
    m ^= m >> vec2u(16u);
    m.x += m.y * 1664525u;
    return f32(m.x) * (1.0 / f32(0xffffffffu));
}

fn _hash2d(p: vec2f) -> vec2f {
    let p2 = vec2f(dot(p, HASH2_DOT1), dot(p, HASH2_DOT2));
    return -1.0 + 2.0 * fract(sin(p2) * HASH_SINE_SCALE);
}

fn _hash3d(p: vec3f) -> vec3f {
    let p2 = vec3f(
        dot(p, HASH3_DOT1),
        dot(p, HASH3_DOT2),
        dot(p, HASH3_DOT3)
    );
    return -1.0 + 2.0 * fract(sin(p2) * HASH_SINE_SCALE);
}

fn _simplex2d(p: vec2f) -> f32 {
    let K1 = 0.366025404;
    let K2 = 0.211324865;
    let i = floor(p + (p.x + p.y) * K1);
    let a = p - i + (i.x + i.y) * K2;
    let o = select(vec2f(1.0, 0.0), vec2f(0.0, 1.0), a.x < a.y);
    let b = a - o + K2;
    let c = a - 1.0 + 2.0 * K2;
    let h = max(0.5 - vec3f(dot(a, a), dot(b, b), dot(c, c)), vec3f(0.0));
    let n = h * h * h * h * vec3f(dot(a, _hash2d(i + 0.0)), dot(b, _hash2d(i + o)), dot(c, _hash2d(i + 1.0)));
    return dot(n, vec3f(70.0));
}

fn _simplex3d(p: vec3f) -> f32 {
    let F3 = 1.0 / 3.0;
    let G3 = 1.0 / 6.0;

    let s = (p.x + p.y + p.z) * F3;
    let i = floor(p + s);
    let t = (i.x + i.y + i.z) * G3;
    let a = p - i + t;

    var i1: vec3f;
    var i2: vec3f;
    if a.x >= a.y {
        if a.y >= a.z { i1 = vec3f(1.0, 0.0, 0.0); i2 = vec3f(1.0, 1.0, 0.0); }
        else if a.x >= a.z { i1 = vec3f(1.0, 0.0, 0.0); i2 = vec3f(1.0, 0.0, 1.0); }
        else { i1 = vec3f(0.0, 0.0, 1.0); i2 = vec3f(1.0, 0.0, 1.0); }
    } else {
        if a.y < a.z { i1 = vec3f(0.0, 0.0, 1.0); i2 = vec3f(0.0, 1.0, 1.0); }
        else if a.x < a.z { i1 = vec3f(0.0, 1.0, 0.0); i2 = vec3f(0.0, 1.0, 1.0); }
        else { i1 = vec3f(0.0, 1.0, 0.0); i2 = vec3f(1.0, 1.0, 0.0); }
    }

    let b = a - i1 + G3;
    let c = a - i2 + 2.0 * G3;
    let d = a - 1.0 + 3.0 * G3;

    let h = max(0.6 - vec4f(dot(a, a), dot(b, b), dot(c, c), dot(d, d)), vec4f(0.0));
    let n = h * h * h * h * vec4f(
        dot(a, _hash3d(i)),
        dot(b, _hash3d(i + i1)),
        dot(c, _hash3d(i + i2)),
        dot(d, _hash3d(i + 1.0))
    );
    return dot(n, vec4f(32.0));
}

fn _fbm2d(pos: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += _simplex2d(pos * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value / max_value;
}

fn _fbm3d(pos: vec3f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += _simplex3d(pos * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value / max_value;
}

fn _bias(y: f32, base: f32) -> f32 {
    let t = (y - base) / TERRAIN_SPREAD;
    return -clamp(t, -TERRAIN_BIAS_CLAMP_RANGE, TERRAIN_BIAS_CLAMP_RANGE);
}

fn terrain_height(pos_xz: vec2f) -> f32 {
    let warp = vec2f(
        _fbm2d(pos_xz * WARP_FREQ, WARP_OCTAVES),
        _fbm2d(pos_xz * WARP_FREQ + WARP_OFFSET_Y, WARP_OCTAVES)
    ) * WARP_STRENGTH;

    let continent = _fbm2d((pos_xz + warp) * CONTINENT_FREQ, CONTINENT_OCTAVES);
    return TERRAIN_BASE_Y + continent * TERRAIN_HEIGHT_RANGE;
}

fn terrain_density(pos: vec3f, base: f32) -> f32 {
    let noise_val = _fbm3d(pos * CAVE_FREQ, CAVE_OCTAVES);
    return noise_val + _bias(pos.y, base);
}

fn cave_density(pos: vec3f, base: f32) -> f32 {
    let cheese = _fbm3d((pos + CHEESE_OFFSET) * CAVE_FREQ, CAVE_OCTAVES);

    let spaghetti_base = _fbm3d((pos + SPAGHETTI_OFFSET) * (CAVE_FREQ * SPAGHETTI_FREQ_SCALE), CAVE_OCTAVES);
    let spaghetti = 1.0 - (abs(spaghetti_base) * SPAGHETTI_TUNNEL_WIDTH_MULT);

    let raw_caves = max(cheese, spaghetti);
    let surface_penalty = clamp(CAVE_CRUST_THICKNESS - _bias(pos.y, base), 0.0, CAVE_CRUST_THICKNESS);

    return raw_caves - surface_penalty;
}

fn is_definitely_empty(chunk_min_y: f32) -> bool {
    let max_surface_height = TERRAIN_BASE_Y + TERRAIN_HEIGHT_RANGE + (TERRAIN_BIAS_CLAMP_RANGE * TERRAIN_SPREAD);
    return chunk_min_y > max_surface_height;
}