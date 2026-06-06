// ╔══════════════════════════════════════════════════════╗
// ║                    CONSTANTS                         ║
// ╚══════════════════════════════════════════════════════╝

const EPS = 1e-6;
const INF = 1e30;
const MAX_STEPS = 50u;
const X_AXIS = 0u;
const Y_AXIS = 1u;
const Z_AXIS = 2u;

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
    // if pos.y < 0 {
    //     return 1;
    // }

    if pos.x == 0 && pos.y == 0 && pos.z == 0 {
        return 1;
    }

    return 0;
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
        return TraceResult(ray.origin, bid, X_AXIS);
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

    return TraceResult(vec3f(0.0), 0u, X_AXIS);
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

    var color = vec4f(0.5, 0.8, 0.9, 1.0);

    if res.block_id != 0 {
        if res.last_hit_axis == X_AXIS {
            color = vec4f(1.0, 0.0, 0.0, 1.0);
        } else if res.last_hit_axis == Y_AXIS {
            color = vec4f(0.0, 1.0, 0.0, 1.0);
        } else {
            color = vec4f(0.0, 0.0, 1.0, 1.0);
        }
    }

    textureStore(t_output, px.xy, color);
}
