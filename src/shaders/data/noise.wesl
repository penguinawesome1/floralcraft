fn _hash(p: vec2f) -> vec2f {
    let p2 = vec2f(dot(p, vec2f(127.1, 311.7)), dot(p, vec2f(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p2) * 43758.5453123);
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
    let n = h * h * h * h * vec3f(dot(a, _hash(i + 0.0)), dot(b, _hash(i + o)), dot(c, _hash(i + 1.0)));
    return dot(n, vec3f(70.0));
}