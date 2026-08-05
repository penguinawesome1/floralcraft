import package::render::render::config;
import package::core::noise;

const PI = 3.1415;
const NIGHT_ZENITH = vec3f(0.001, 0.002, 0.005);
const NIGHT_HORIZON = vec3f(0.004, 0.005, 0.012);
const DAWN_ZENITH = vec3f(0.15, 0.20, 0.45);
const DAWN_HORIZON = vec3f(0.80, 0.40, 0.20);
const DAY_ZENITH = vec3f(0.20, 0.45, 0.90);
const DAY_HORIZON = vec3f(0.65, 0.78, 0.95);

fn sun_dir() -> vec3f {
    let angle = (config.time_of_day - 0.25) * 2.0 * PI;
    let elevation = sin(angle);
    let horizontal = cos(angle);
    return normalize(vec3f(horizontal, elevation, 0.0));
}

fn _cirrus_density(dir: vec3f) -> f32 {
    let shell_height = 8000.0;
    let t = shell_height / max(dir.y, 0.05);
    let sample_pos = dir * t;
    let scaled_xz = sample_pos.xz * 0.0001;

    let motion = config.time_of_day * 8.0;

    let wind_dir = vec2f(0.9, 0.436);
    let scroll = wind_dir * motion;
    let base_uv = scaled_xz + scroll;

    let warp_time_offset = vec2f(motion * 0.6, motion * -0.4);
    let warp_x = noise::fbm3d(vec3f(base_uv * 0.15 + warp_time_offset, 0.0), 2u);
    let warp_y = noise::fbm3d(vec3f(base_uv * 0.15 + warp_time_offset + vec2f(11.3, 4.7), 0.0), 2u);
    let warped_uv = base_uv + vec2f(warp_x, warp_y) * 0.15;

    let stretched = vec2f(warped_uv.x * 4.0, warped_uv.y * 1.2);
    let density = noise::fbm3d(vec3f(stretched.x, stretched.y, 0.0), 4u) * 0.5 + 0.5;
    return smoothstep(0.45, 0.75, density);
}

fn color(ray_dir: vec3f) -> vec3f {
    let sun = sun_dir();
    let night_t = smoothstep(-0.3, 0.0, sun.y);
    let day_t = smoothstep(0.0, 0.3, sun.y);
    let zenith = mix(mix(NIGHT_ZENITH, DAWN_ZENITH, night_t), DAY_ZENITH, day_t);
    let horizon = mix(mix(NIGHT_HORIZON, DAWN_HORIZON, night_t), DAY_HORIZON, day_t);
    let sky_t = smoothstep(0.0, 0.5, ray_dir.y);
    var final_color = mix(horizon, zenith, sky_t);

    if ray_dir.y > 0.05 {
        let cirrus = _cirrus_density(ray_dir);
        let horizon_mask = smoothstep(0.05, 0.35, ray_dir.y);
        let night_fade = smoothstep(-0.15, 0.05, sun.y);

        let final_cloud_alpha = cirrus * horizon_mask * night_fade * 0.5;

        let sun_facing = pow(clamp(dot(ray_dir, sun), 0.0, 1.0), 4.0);
        let base_cloud_color = mix(vec3f(0.15), vec3f(0.85), day_t);
        let cloud_color = mix(base_cloud_color, vec3f(1.0, 0.9, 0.75), sun_facing * day_t);

        final_color = mix(final_color, cloud_color, final_cloud_alpha);
    }

    return final_color;
}