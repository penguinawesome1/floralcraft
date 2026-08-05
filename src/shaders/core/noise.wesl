fn hash2d_to_1d(p: vec2f) -> f32 {
    let src = bitcast<vec2u>(p);
    var m = src * 1664525u + 1013904223u;
    m.x += m.y * 1664525u;
    m.y += m.x * 1664525u;
    m ^= m >> vec2u(16u);
    m.x += m.y * 1664525u;
    return f32(m.x) * (1.0 / 4294967295.0);
}

fn _hash2d(p: vec2f) -> vec2f {
    var v = vec2u(vec2i(p)) * 1664525u + 1013904223u;
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    v.x += v.y * 1664525u;
    v.y += v.x * 1664525u;
    v ^= v >> vec2u(16u);
    return -1.0 + 2.0 * (vec2f(v) * (1.0 / 4294967295.0));
}

fn _hash3d(p: vec3f) -> vec3f {
    var v = vec3u(vec3i(p)) * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3u(16u);
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3u(16u);
    return -1.0 + 2.0 * (vec3f(v) * (1.0 / 4294967295.0));
}

fn simplex2d(p: vec2f) -> f32 {
    let K1 = 0.366025404;
    let K2 = 0.211324865;
    let i = floor(p + (p.x + p.y) * K1);
    let a = p - i + (i.x + i.y) * K2;
    let o = select(vec2f(1.0, 0.0), vec2f(0.0, 1.0), a.x < a.y);
    let b = a - o + K2;
    let c = a - 1.0 + 2.0 * K2;
    let h = max(0.5 - vec3f(dot(a, a), dot(b, b), dot(c, c)), vec3f(0.0));
    let n = h * h * h * h * vec3f(
        dot(a, _hash2d(i + vec2f(0.0))),
        dot(b, _hash2d(i + o)),
        dot(c, _hash2d(i + 1.0))
    );
    return dot(n, vec3f(70.0));
}

fn simplex3d(p: vec3f) -> f32 {
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

fn fbm2d(pos: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += simplex2d(pos * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value / max_value;
}

fn fbm3d(pos: vec3f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        value += simplex3d(pos * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value / max_value;
}

fn ridged2d(pos: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 1.0;
    var max_value = 0.0;
    for (var i = 0u; i < octaves; i++) {
        let noise = simplex2d(pos * frequency);
        let shaped = pow((1.0 - abs(noise)), 2.0);
        value += shaped * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    return value / max_value;
}