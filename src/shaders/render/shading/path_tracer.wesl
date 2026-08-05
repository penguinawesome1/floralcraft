import package::render::render::config;
import package::core::blocks;
import package::render::traversal::dda::{TraceResult, Ray};
import package::render::traversal::dda;
import package::render::shading::{sky, pbr, spectral};

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
        case blocks::GRASS: {
            return BlockMaterial(vec3f(-12.0, 12.0, -2.2), 0.5, 0.0, 0.04, 0.0);
        }
        case blocks::DIRT: {
            return BlockMaterial(vec3f(5.8, 1.2, -4.3), 0.92, 0.0, 0.04, 0.0);
        }
        case blocks::STONE: {
            return BlockMaterial(vec3f(0.5, -0.1, -1.0), 0.65, 0.0, 0.04, 0.0);
        }
        case blocks::BEDROCK: {
            return BlockMaterial(vec3f(0.4, -0.1, -1.7), 0.95, 0.0, 0.04, 0.0);
        }
        case blocks::SAND: {
            return BlockMaterial(vec3f(3.5, 0.6, -0.3), 0.85, 0.0, 0.04, 0.0);
        }
        case blocks::MIRROR: {
            return BlockMaterial(vec3f(1.0, 0.0, 0.0), 0.0, 1.0, 1.0, 0.0);
        }
        default: {
            return BlockMaterial(vec3f(1.0, 1.0, 1.0), 0.0, 0.0, 0.04, 15.0);
        }
    }
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

    for (var bounce = 0u; bounce < 3u; bounce++) {
        let material = _block_material(curr_res.block_id);
        let spectral_albedo = spectral::coeffs_to_spectrum(material.spectral_coeffs, wavelengths);

        let N = dda::axis_normal(curr_res.last_hit_axis, curr_dir);
        let L = sky::sun_dir();
        let V = -curr_dir;
        let H = normalize(V + L);

        let eps = max(0.005, length(curr_res.hit_pos) * 1e-5);
        let shadow_origin = curr_res.hit_pos + N * eps;

        let F0 = mix(vec4f(material.f0), spectral_albedo, material.metallic);
        let F = pbr::fresnel_schlick_spectral(V, H, F0);
        let alpha = material.roughness * material.roughness;
        let D = pbr::distribution_GGX(N, H, alpha);
        let G = pbr::geometry_smith(N, V, L, alpha);

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
            let emitter_spectrum = spectral::coeffs_to_spectrum(material.spectral_coeffs, wavelengths);
            let emission_spectral = emitter_spectrum * material.emission;
            radiance += emission_spectral * throughput;
            break;
        }

        let specular = (vec4f(D * G) * F) / max(4.0 * NdotV * NdotL, 0.001);
        let kD = (vec4f(1.0) - F) * (1.0 - material.metallic);
        let diffuse = kD * spectral_albedo / PI;
        let is_specular = material.metallic > 0.9 && material.roughness < 0.05;

        let bounce_seed = pixel_seed + vec2f(f32(bounce) * 17.13, f32(bounce) * 31.41);
        let next_dir = pbr::sample_rough_bounce(curr_dir, N, material.roughness, bounce_seed);
        let NdotNext = max(dot(N, next_dir), 0.0);

        if is_specular {
            let F_mirror = pbr::fresnel_schlick_spectral(V, N, F0);
            throughput *= F_mirror * NdotNext;
        } else {
            let direct_light = (diffuse + specular) * NdotL * visibility;
            radiance += direct_light * throughput;
            throughput *= spectral_albedo * NdotNext;
        }

        curr_dir = next_dir;
        curr_res = dda::trace(Ray(shadow_origin, next_dir), 32.0);

        if curr_res.block_id == blocks::AIR {
            let sky_coeffs = sky::spectral_coeffs(curr_dir);
            let sky_spectral = spectral::coeffs_to_spectrum(sky_coeffs, wavelengths);
            radiance += sky_spectral * throughput;
            break;
        }

        if length(throughput) < 0.03 { break; }
    }

    let xyz = spectral::cie_xyz(wavelengths, radiance);
    let srgb = spectral::xyz_to_linear_srgb(xyz);
    let fogged = _apply_fog(srgb, initial_dir, initial_res);
    let mapped = fogged / (fogged + vec3f(1.0));
    let gamma_corrected = pow(mapped, vec3f(1.0 / 2.2));

    return vec4f(gamma_corrected, 1.0);
}
