import package::config::config;

const PI = 3.1415;
const NIGHT_ZENITH = vec3f(0.02, 0.02, 0.05);
const NIGHT_HORIZON = vec3f(0.05, 0.05, 0.10);
const DAWN_ZENITH = vec3f(0.25, 0.30, 0.55);
const DAWN_HORIZON = vec3f(0.90, 0.55, 0.35);
const DAY_ZENITH = vec3f(0.30, 0.55, 0.95);
const DAY_HORIZON = vec3f(0.70, 0.80, 0.95);

fn sun_dir() -> vec3f {
    let angle = (config.time_of_day - 0.25) * 2.0 * PI;
    let elevation = sin(angle);
    let horizontal = cos(angle);
    return normalize(vec3f(horizontal, elevation, 0.0));
}

fn color(ray_dir: vec3f) -> vec3f {
    let sun = sun_dir();
    let night_t = smoothstep(-0.3, 0.0, sun.y); // night to dawn
    let day_t = smoothstep(0.0, 0.3, sun.y); // dawn to day
    let zenith = mix(mix(NIGHT_ZENITH, DAWN_ZENITH, night_t), DAY_ZENITH, day_t);
    let horizon = mix(mix(NIGHT_HORIZON, DAWN_HORIZON, night_t), DAY_HORIZON, day_t);
    let sky_t = smoothstep(0.0, 0.5, ray_dir.y);
    return mix(horizon, zenith, sky_t);
}
