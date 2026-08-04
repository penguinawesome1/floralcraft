import package::core::noise;

const _DOMAIN_WARP_FREQ = 1.0 / 2500.0;
const _DOMAIN_WARP_OCTAVES = 4u;
const _DOMAIN_WARP_OFFSET_Y = vec2f(42.5, 11.8);
const _DOMAIN_WARP_AMPLITUDE = 40.0;
fn _domain_warp(pos: vec2f) -> vec2f {
    return pos + vec2f(
        noise::fbm2d(pos * _DOMAIN_WARP_FREQ, _DOMAIN_WARP_OCTAVES),
        noise::fbm2d(pos * _DOMAIN_WARP_FREQ + _DOMAIN_WARP_OFFSET_Y, _DOMAIN_WARP_OCTAVES)
    ) * _DOMAIN_WARP_AMPLITUDE;
}

const _GROUND_DRIFT_OFFSET = vec2f(-9173.2, 4408.6);
const _GROUND_DRIFT_FREQ = 1.0 / 10000.0;
const _GROUND_DRIFT_AMPLITUDE = 500.0;
fn _ground_drift(pos: vec2f) -> f32 {
    return noise::simplex2d((pos + _GROUND_DRIFT_OFFSET) * _GROUND_DRIFT_FREQ) * _GROUND_DRIFT_AMPLITUDE;
}

const _HILLS_OFFSET = vec2f(-9173.2, 4408.6);
const _HILLS_FREQ = 1.0 / 700.0;
const _HILLS_STEEPNESS = 3.0;
const _HILLS_AMPLITUDE = 100.0;
fn _hills(pos: vec2f) -> f32 {
    let noise = noise::simplex2d((pos + _HILLS_OFFSET) * _HILLS_FREQ);
    let hill_curve = 1.0 / (1.0 + exp(-_HILLS_STEEPNESS * noise));
    return hill_curve * _HILLS_AMPLITUDE;
}

fn surface_height(pos: vec2i) -> f32 {
    let warped = _domain_warp(vec2f(pos));
    let ground = _ground_drift(warped);
    let hills = _hills(warped);
    return ground + hills;
}

fn surface_density(pos: vec3i, terrain_height: f32) -> f32 {
    let base_density = terrain_height - f32(pos.y);
    return base_density;
}

fn cave_density(pos: vec3i, depth: f32) -> f32 {
    return 0.0;
}

fn is_definitely_empty(y: i32) -> bool {
    return y > i32(_GROUND_DRIFT_AMPLITUDE + _HILLS_AMPLITUDE);
}