import package::render::render::config;
import package::core::noise;
import package::render::shading::spectral;

const _PI = 3.1415;

const _NIGHT_ZENITH_COEFFS = vec3f(-17.0348, -2.1341, -5.7680);
const _NIGHT_HORIZON_COEFFS = vec3f(-0.1602, -7.7266, -2.9198);
const _DAWN_ZENITH_COEFFS = vec3f(1.0457, -2.9337, 0.4805);
const _DAWN_HORIZON_COEFFS = vec3f(0.9659, 0.8016, -0.7687);
const _DAY_ZENITH_COEFFS = vec3f(11.9822, -16.4373, 5.0276);
const _DAY_HORIZON_COEFFS = vec3f(22.5299, -30.0, 10.1200);

const _CLOUD_NIGHT_COEFFS = vec3f(-1.0150, 0.4904, -0.9443);
const _CLOUD_DAY_COEFFS = vec3f(-0.6561, -2.4013, 2.5500);
const _CLOUD_SUNLIT_COEFFS = vec3f(-9.9939, 8.9120, -0.2311);

fn sun_dir() -> vec3f {
    let angle = (config.time_of_day - 0.25) * 2.0 * _PI;
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
    let warp_x = noise::simplex3d(vec3f(base_uv * 0.15 + warp_time_offset, 0.0));
    let warp_y = noise::simplex3d(vec3f(base_uv * 0.15 + warp_time_offset + vec2f(11.3, 4.7), 0.0));
    let warped_uv = base_uv + vec2f(warp_x, warp_y) * 0.15;

    let stretched = vec2f(warped_uv.x * 4.0, warped_uv.y * 1.2);
    let density = noise::fbm3d(vec3f(stretched.x, stretched.y, 0.0), 4u) * 0.5 + 0.5;
    return smoothstep(0.45, 0.75, density);
}

fn spectral_coeffs(ray_dir: vec3f) -> vec3f {
    let sun = sun_dir();
    let night_t = smoothstep(-0.3, 0.0, sun.y);
    let day_t = smoothstep(0.0, 0.3, sun.y);
    let zenith = mix(mix(_NIGHT_ZENITH_COEFFS, _DAWN_ZENITH_COEFFS, night_t), _DAY_ZENITH_COEFFS, day_t);
    let horizon = mix(mix(_NIGHT_HORIZON_COEFFS, _DAWN_HORIZON_COEFFS, night_t), _DAY_HORIZON_COEFFS, day_t);
    let sky_t = smoothstep(0.0, 0.5, ray_dir.y);
    var final_coeffs = mix(horizon, zenith, sky_t);

    if ray_dir.y > 0.05 {
        let cirrus = _cirrus_density(ray_dir);
        let horizon_mask = smoothstep(0.05, 0.35, ray_dir.y);
        let night_fade = smoothstep(-0.15, 0.05, sun.y);

        let final_cloud_alpha = cirrus * horizon_mask * night_fade * 0.5;

        let sun_facing = pow(clamp(dot(ray_dir, sun), 0.0, 1.0), 4.0);
        let base_cloud_coeffs = mix(_CLOUD_NIGHT_COEFFS, _CLOUD_DAY_COEFFS, day_t);
        let cloud_coeffs = mix(base_cloud_coeffs, _CLOUD_SUNLIT_COEFFS, sun_facing * day_t);

        final_coeffs = mix(final_coeffs, cloud_coeffs, final_cloud_alpha);
    }

    return final_coeffs;
}

fn color(ray_dir: vec3f) -> vec3f {
    let wavelengths = spectral::sample_hero_wavelengths(ray_dir.xz + vec2f(ray_dir.y));
    let coeffs = spectral_coeffs(ray_dir);
    let spectral_color = spectral::coeffs_to_spectrum(coeffs, wavelengths);
    let xyz = spectral::cie_xyz(wavelengths, spectral_color);
    return spectral::xyz_to_linear_srgb(xyz);
}