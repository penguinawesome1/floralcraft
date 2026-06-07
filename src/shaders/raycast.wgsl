// ╔══════════════════════════════════════════════════════╗
// ║                    CONSTANTS                         ║
// ╚══════════════════════════════════════════════════════╝

const EPS = 1e-6;
const INF = 1e30;
const MAX_STEPS = 50u;
const NO_AXIS = 0u;
const X_AXIS = 1u;
const Y_AXIS = 2u;
const Z_AXIS = 3u;
const PI = 3.14159;

// ╔══════════════════════════════════════════════════════╗
// ║                    STRUCTS                           ║
// ╚══════════════════════════════════════════════════════╝

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

struct Ray {
    origin: vec3f,
    dir: vec3f,
}

struct StepResult {
    t: f32,
    t_max: vec3f,
    snapped_pos: vec3i,
    last_hit_axis: u32,
}

struct TraceResult {
    hit_pos: vec3f,
    block_id: u32,
    last_hit_axis: u32,
}

struct BlockMaterial {
    albedo: vec3f,
    roughness: f32,
    metallic: f32,
}

// ╔══════════════════════════════════════════════════════╗
// ║                    BINDINGS                          ║
// ╚══════════════════════════════════════════════════════╝

@group(0) @binding(0) var t_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> u_cam: Camera;

// ╔══════════════════════════════════════════════════════╗
// ║                    CONVERSIONS                       ║
// ╚══════════════════════════════════════════════════════╝

// Converts pixel coordinates to a normalized world-space ray direction.
// Used as the primary vector for grid traversal.
fn px_to_dir(px: vec2u) -> vec3f {
    let canvas_size = textureDimensions(t_output);
    let aspect = f32(canvas_size.x) / f32(canvas_size.y);

    let uv = (vec2f(px) / vec2f(canvas_size)) * 2.0 - 1.0;
    let scaled = vec2f(uv.x * aspect, -uv.y);
    let view_dir = vec3f(scaled, -1.0);
    let world_dir = (u_cam.rotation * vec4f(view_dir, 0.0)).xyz;

    return normalize(world_dir);
}

// ╔══════════════════════════════════════════════════════╗
// ║                    BLOCK ACCESS                      ║
// ╚══════════════════════════════════════════════════════╝

fn block_id(pos: vec3i) -> u32 {
    if pos.y < 0 {
        return 2;
    }

    if pos.x == 5 && pos.y == 5 && pos.z == 5 {
        return 1;
    }

    return 0;
}

fn block_material(id: u32) -> BlockMaterial {
    switch id {
        case 1: { return BlockMaterial(vec3f(1.0, 0.08, 0.58), 0.5, 0.5); }
        case 2: { return BlockMaterial(vec3f(0.18, 1.0, 1.0), 0.5, 0.5); }
        case 3: { return BlockMaterial(vec3f(0.96, 0.96, 0.45), 0.5, 0.5); }
        case 4: { return BlockMaterial(vec3f(0.96, 0.30, 0.16), 0.5, 0.5); }
        case 5: { return BlockMaterial(vec3f(1.0, 0.5, 0.31), 0.5, 0.5); }
        default: { return BlockMaterial(vec3f(0.5, 0.0, 1.0), 0.5, 0.5); }
    }
}

// ╔══════════════════════════════════════════════════════╗
// ║                    RAYCASTING                        ║
// ╚══════════════════════════════════════════════════════╝

// Finds the grid boundary that first occurs along the given ray.
fn step(curr_t_max: vec3f, current_pos: vec3i, delta_t: vec3f, grid_step: vec3i) -> StepResult {
    var step: StepResult;
    step.t_max = curr_t_max;
    step.snapped_pos = current_pos;

    if curr_t_max.x < curr_t_max.y {
        if curr_t_max.x < curr_t_max.z {
            step.t = curr_t_max.x;
            step.t_max.x += delta_t.x;
            step.snapped_pos.x += grid_step.x;
            step.last_hit_axis = X_AXIS;
        } else {
            step.t = curr_t_max.z;
            step.t_max.z += delta_t.z;
            step.snapped_pos.z += grid_step.z;
            step.last_hit_axis = Z_AXIS;
        }
    } else {
        if curr_t_max.y < curr_t_max.z {
            step.t = curr_t_max.y;
            step.t_max.y += delta_t.y;
            step.snapped_pos.y += grid_step.y;
            step.last_hit_axis = Y_AXIS;
        } else {
            step.t = curr_t_max.z;
            step.t_max.z += delta_t.z;
            step.snapped_pos.z += grid_step.z;
            step.last_hit_axis = Z_AXIS;
        }
    }

    return step;
}

// Finds the first block the ray hits.
fn trace(ray: Ray) -> TraceResult {
    var snapped_pos = vec3i(floor(ray.origin));

    let bid = block_id(snapped_pos);
    if bid != 0u {
        return TraceResult(ray.origin, bid, NO_AXIS);
    }

    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);

    let next_boundary = vec3f(snapped_pos) + vec3f(grid_step > vec3i(0));
    let raw_t_max = (next_boundary - ray.origin) / ray.dir;

    var t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
    var last_hit_axis: u32;

    for (var i = 0u; i < MAX_STEPS; i++) {
        let res = step(t_max, snapped_pos, delta_t, grid_step);
        t_max = res.t_max;
        snapped_pos = res.snapped_pos;
        last_hit_axis = res.last_hit_axis;

        let bid = block_id(snapped_pos);
        if bid == 0u {
            continue;
        }

        return TraceResult(ray.origin + ray.dir * res.t, bid, res.last_hit_axis);
    }

    return TraceResult(vec3f(0.0), 0u, NO_AXIS);
}

fn axis_normal(axis: u32, dir: vec3f) -> vec3f {
    let s = -sign(dir);
    switch axis {
        case X_AXIS: { return vec3f(s.x, 0.0, 0.0); }
        case Y_AXIS: { return vec3f(0.0, s.y, 0.0); }
        case Z_AXIS: { return vec3f(0.0, 0.0, s.z); }
        default: { return -normalize(dir); }
    }
}

// ╔══════════════════════════════════════════════════════╗
// ║                        PBR                           ║
// ╚══════════════════════════════════════════════════════╝

fn distribution_GGX(N: vec3f, H: vec3f, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn geometry_smith(N: vec3f, V: vec3f, L: vec3f, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let gv = NdotV / (NdotV * (1.0 - k) + k);
    let gl = NdotL / (NdotL * (1.0 - k) + k);
    return gv * gl;
}

fn fresnel_schlick(V: vec3f, H: vec3f, F0: vec3f) -> vec3f {
    let VdotH = max(dot(V, H), 0.0);
    return F0 + (1.0 - F0) * pow(1.0 - VdotH, 5.0);
}

fn pbr(dir: vec3f, res: TraceResult) -> vec4f {
    let material = block_material(res.block_id);

    let N = axis_normal(res.last_hit_axis, dir);
    let V = -dir;
    let L = normalize(vec3f(1.0, 2.0, 1.0));
    let H = normalize(V + L);

    let shadow_origin = res.hit_pos + N * EPS * 10.0;
    let shadow_ray = Ray(shadow_origin, L);
    let shadow_res = trace(shadow_ray);
    let in_shadow = shadow_res.block_id != 0u;

    let F0 = mix(vec3f(0.04), material.albedo, material.metallic);
    let F = fresnel_schlick(V, H, F0);
    let D = distribution_GGX(N, H, material.roughness);
    let G = geometry_smith(N, V, L, material.roughness);

    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);

    let specular = (D * G * F) / max(4.0 * NdotV * NdotL, EPS);
    let kD = (1.0 - F) * (1.0 - material.metallic);
    let diffuse = kD * material.albedo / PI;

    let ambient = vec3f(0.03) * material.albedo;
    let direct = select((diffuse + specular) * NdotL, vec3f(0.0), in_shadow);
    let Lo = ambient + direct;

    let mapped = Lo / (Lo + vec3f(1.0));
    let out = pow(mapped, vec3f(1.0 / 2.2));

    return vec4f(out, 1.0);
}

// ╔══════════════════════════════════════════════════════╗
// ║                    ENTRY POINT                       ║
// ╚══════════════════════════════════════════════════════╝

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) px: vec3u) {
    let canvas_size = textureDimensions(t_output);
    if any(px.xy >= canvas_size) {
        return;
    }

    let dir = px_to_dir(px.xy);
    let ray = Ray(u_cam.pos, dir);
    let res = trace(ray);

    let is_debug_mode = false;

    if res.block_id == 0 {
        textureStore(t_output, px.xy, vec4f(0.7, 0.75, 0.95, 1.0));
        return;
    }

    if is_debug_mode {
        var debug_color = vec4f(0.0, 0.0, 1.0, 1.0);

        if res.last_hit_axis == X_AXIS {
            debug_color = vec4f(1.0, 0.0, 0.0, 1.0);
        } else if res.last_hit_axis == Y_AXIS {
            debug_color = vec4f(0.0, 1.0, 0.0, 1.0);
        }

        textureStore(t_output, px.xy, debug_color);
        return;
    }

    let color = pbr(dir, res);
    textureStore(t_output, px.xy, color);
}
