import package::raytrace::bindings::config;
import package::core::{blocks, noise};
import package::raytrace::support::dda::{TraceResult, Ray};
import package::raytrace::support::dda;
import package::raytrace::shading::{sky, spectral};

const PI = 3.14159265;
const EPS = 1e-4;

struct BlockMaterial {
    spectral_coeffs: vec3f,
    roughness: f32,
    metallic: f32,
    f0: f32,
    emission: f32,
}

fn _block_material(id: u32) -> BlockMaterial {
    switch id {
        case blocks::GRASS: { return BlockMaterial(vec3f(-28.0, 25.0, -4.2), 0.6, 0.0, 0.04, 0.0); }
        case blocks::DIRT: { return BlockMaterial(vec3f(0.0, 1.2, -1.8), 0.92, 0.0, 0.04, 0.0); }
        case blocks::STONE: { return BlockMaterial(vec3f(0.0, 0.0, -0.73), 0.65, 0.0, 0.04, 0.0); }
        default: { return BlockMaterial(vec3f(15.0, -6.0, -0.2), 0.5, 0.5, 0.04, 15.0); }
    }
}

fn _direction_basis(N: vec3f) -> mat3x3f {
    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(N.y) > 0.99);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);
    return mat3x3f(tangent, bitangent, N);
}

fn _sample_cosine_hemisphere(N: vec3f, seed: vec2f) -> vec3f {
    let u1 = noise::hash2d_to_1d(seed);
    let u2 = noise::hash2d_to_1d(seed + vec2f(13.37, 42.11));

    let r = sqrt(u1);
    let phi = 2.0 * PI * u2;

    let local_dir = vec3f(r * cos(phi), r * sin(phi), sqrt(max(0.0, 1.0 - u1)));
    let basis = _direction_basis(N);
    return normalize(basis * local_dir);
}

fn _sample_rough_bounce(curr_dir: vec3f, N: vec3f, roughness: f32, seed: vec2f) -> vec3f {
    let spec_dir = reflect(curr_dir, N);
    let diff_dir = _sample_cosine_hemisphere(N, seed);
    return normalize(mix(spec_dir, diff_dir, roughness * roughness));
}

fn _distribution_GGX(N: vec3f, H: vec3f, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn _geometry_smith(N: vec3f, V: vec3f, L: vec3f, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let gv = NdotV / (NdotV * (1.0 - k) + k);
    let gl = NdotL / (NdotL * (1.0 - k) + k);
    return gv * gl;
}

fn _fresnel_schlick_spectral(V: vec3f, H: vec3f, F0: vec4f) -> vec4f {
    let VdotH = max(dot(V, H), 0.0);
    return F0 + (vec4f(1.0) - F0) * pow(1.0 - VdotH, 5.0);
}

fn _apply_fog(srgb: vec3f, initial_dir: vec3f, initial_res: TraceResult) -> vec3f {
    let fog_density = 1.8 / config.max_trace_dist;
    let n_dist = initial_res.distance * fog_density;
    let fog_factor = 1.0 - exp(-(n_dist * n_dist));
    let sky_color = sky::color(initial_dir);
    return mix(srgb, sky_color, clamp(fog_factor, 0.0, 1.0));
}

fn color(initial_dir: vec3f, initial_res: TraceResult) -> vec4f {
    var curr_dir = initial_dir;
    var curr_res = initial_res;

    let pixel_seed = initial_res.hit_pos.xz + vec2f(initial_res.hit_pos.y);
    let wavelengths = spectral::sample_hero_wavelengths(pixel_seed);

    var radiance = vec4f(0.0);
    var throughput = vec4f(1.0);

    for (var bounce = 0u; bounce < 2u; bounce++) {
        if curr_res.block_id == blocks::AIR {
            let sky_color = sky::color(curr_dir);
            let sky_spectral = spectral::rgb_to_spectrum(sky_color, wavelengths);
            let cave_depth_factor = clamp((curr_res.hit_pos.y - 10.0) / 40.0, 0.0, 1.0);
            radiance += sky_spectral * throughput * cave_depth_factor;
            break;
        }

        let material = _block_material(curr_res.block_id);
        let spectral_albedo = spectral::rgb_to_spectrum(material.spectral_coeffs, wavelengths);

        let N = dda::axis_normal(curr_res.last_hit_axis, curr_dir);
        let L = sky::sun_dir();
        let V = -curr_dir;
        let H = normalize(V + L);

        let eps = max(0.005, length(curr_res.hit_pos) * 1e-5);
        let shadow_origin = curr_res.hit_pos + N * eps;

        let f0_dielectric = vec4f(material.f0);
        let F0 = mix(f0_dielectric, spectral_albedo, vec4f(material.metallic));
        let F = _fresnel_schlick_spectral(V, H, F0);
        let D = _distribution_GGX(N, H, material.roughness);
        let G = _geometry_smith(N, V, L, material.roughness);

        let NdotL = clamp(dot(N, L), 0.0, 1.0);
        let NdotV = clamp(dot(N, V), 0.0, 1.0);

        var visibility = 0.0;
        if NdotL > 0.0 {
            let max_shadow_dist = 64.0 + 64.0 * clamp((0.5 - L.y) * 2.5, 0.0, 1.0);
            let shadow_res = dda::trace(Ray(curr_res.hit_pos + N * EPS, L), max_shadow_dist);
            if shadow_res.block_id == blocks::AIR {
                visibility = 1.0;
            } else {
                visibility = smoothstep(0.0, max_shadow_dist, shadow_res.distance);
            }
        }

        if material.emission > 0.0 {
            let emitter_spectrum = spectral::rgb_to_spectrum(material.spectral_coeffs, wavelengths);
            let emission_spectral = emitter_spectrum * material.emission;
            radiance += emission_spectral * throughput;
        }

        let specular = (vec4f(D * G) * F) / max(4.0 * NdotV * NdotL, 0.001);
        let kD = (vec4f(1.0) - F) * (1.0 - material.metallic);
        let diffuse = kD * spectral_albedo / PI;
        radiance += (diffuse + specular) * NdotL * throughput * visibility;

        let bounce_seed = pixel_seed + vec2f(f32(bounce) * 17.13, f32(bounce) * 31.41);
        let next_dir = _sample_rough_bounce(curr_dir, N, material.roughness, bounce_seed);
        let next_ray = Ray(shadow_origin, next_dir);

        let NdotNext = max(dot(N, next_dir), 0.0);
        throughput *= spectral_albedo * NdotNext;

        curr_dir = next_dir;
        curr_res = dda::trace(next_ray, 32.0);

        if length(throughput) < 0.01 { break; }
    }

    let xyz = spectral::cie_xyz(wavelengths, radiance);
    let srgb = spectral::xyz_to_linear_srgb(xyz);
    let fogged = _apply_fog(srgb, initial_dir, initial_res);
    let mapped = fogged / (fogged + vec3f(1.0));
    let out = pow(mapped, vec3f(1.0 / 2.2));

    return vec4f(out, 1.0);
}