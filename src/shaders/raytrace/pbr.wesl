import package::raytrace::dda::{TraceResult, Ray, axis_normal, trace};
import package::raytrace::cam;
import package::data::blocks;

const PI = 3.1415;
const NUDGE = 0.001;

struct BlockMaterial {
    albedo: vec3f,
    roughness: f32,
    metallic: f32,
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

fn _fresnel_schlick(V: vec3f, H: vec3f, F0: vec3f) -> vec3f {
    let VdotH = max(dot(V, H), 0.0);
    return F0 + (1.0 - F0) * pow(1.0 - VdotH, 5.0);
}

fn _block_material(id: u32) -> BlockMaterial {
    switch id {
        case blocks::GRASS: { return BlockMaterial(vec3f(0.18, 0.38, 0.15), 0.85, 0.0); }
        case blocks::DIRT: { return BlockMaterial(vec3f(0.25, 0.16, 0.11), 0.92, 0.0); }
        case blocks::STONE: { return BlockMaterial(vec3f(0.32, 0.33, 0.35), 0.65, 0.0); }
        case blocks::BEDROCK: { return BlockMaterial(vec3f(0.25, 0.23, 0.22), 0.95, 0.0); }
        case 8: { return BlockMaterial(vec3f(1.0, 0.0, 0.0), 0.5, 0.5); }
        default: { return BlockMaterial(vec3f(0.5, 0.0, 1.0), 0.5, 0.5); }
    }
}

// NOTE: AI-generated prototype. To be thoroughly reviewed and refactored in a future development stage.
fn get_color(initial_dir: vec3f, initial_res: TraceResult) -> vec4f {
    // return vec4f(_block_material(initial_res.block_id).albedo, 1.0);

    var current_dir = initial_dir;
    var current_res = initial_res;

    var accumulated_color = vec3f(0.0);
    var throughput = vec3f(1.0); // Keeps track of how much light can survive the bounces

    let MAX_BOUNCES = 10u;

    for (var bounce = 0u; bounce < MAX_BOUNCES; bounce = bounce + 1u) {
        if current_res.block_id == 0u {
            // Hit the sky: Add sky/ambient light modified by current throughput and break
            let sky_color = vec3f(0.5, 0.7, 1.0) * 0.5;
            accumulated_color += sky_color * throughput;
            break;
        }

        let material = _block_material(current_res.block_id);
        let N = axis_normal(current_res.last_hit_axis, current_dir);
        let V = -current_dir;
        let L = normalize(vec3f(1.0, 2.0, 1.0)); // Direction to primary sun light
        let H = normalize(V + L);

        // --- 1. Calculate Direct Contribution (Sun Light & Shadows) ---
        let dist = distance(cam.pos, current_res.hit_pos);
        let bias = max(1e-4 * dist, NUDGE);
        let shadow_origin = current_res.hit_pos + N * bias;

        let shadow_ray = Ray(shadow_origin, L);
        let shadow_res = trace(shadow_ray);
        let in_shadow = shadow_res.block_id != 0u;

        let F0 = mix(vec3f(0.04), material.albedo, material.metallic);
        let F = _fresnel_schlick(V, H, F0);
        let D = _distribution_GGX(N, H, material.roughness);
        let G = _geometry_smith(N, V, L, material.roughness);

        let NdotL = clamp(dot(N, L), 0.0, 1.0);
        let NdotV = clamp(dot(N, V), 0.0, 1.0);

        let specular = (D * G * F) / max(4.0 * NdotV * NdotL, NUDGE);
        let kD = (1.0 - F) * (1.0 - material.metallic);
        let diffuse = kD * material.albedo / PI;

        // Add direct sun light scaled by the current throughput
        let direct_light = select((diffuse + specular) * NdotL, vec3f(0.0), in_shadow);
        accumulated_color += direct_light * throughput;

        // --- 2. Prepare Next Ray Bounce (Indirect Global Illumination) ---
        // For simple diffuse indirect light bounce, reflect perfectly or pick an angle.
        // For a full path tracer, 'next_dir' should be a random vector on the hemisphere.
        let next_dir = reflect(current_dir, N);
        let next_ray = Ray(shadow_origin, next_dir);

        // Update throughput: The next bounce carries less energy depending on material color
        throughput *= material.albedo * max(dot(N, next_dir), 0.0);

        // Trace the secondary ray for the next loop iteration
        current_dir = next_dir;
        current_res = trace(next_ray);

        // Optimization: Stop bouncing if the light energy has completely petered out
        if length(throughput) < 0.01 {
            break;
        }
    }

    // --- 3. Post-Processing ---
    let mapped = accumulated_color / (accumulated_color + vec3f(1.0));
    let out = pow(mapped, vec3f(1.0 / 2.2));

    return vec4f(out, 1.0);
}
