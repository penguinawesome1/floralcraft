#include "../data/read/Chunk.wgsl"
#include "../data/read/World.wgsl"
#include "./conversions.wgsl"

const dda__NO_AXIS = 0u;
const dda__X_AXIS = 1u;
const dda__Y_AXIS = 2u;
const dda__Z_AXIS = 3u;
const dda__INF = 1e30;
const dda__EPS = 1e-6;

struct dda__StepResult {
    t: f32,
    t_max: vec3f,
    snapped_pos: vec3i,
    last_hit_axis: u32,
}

struct dda__TraceResult {
    hit_pos: vec3f,
    block_id: u32,
    last_hit_axis: u32,
}

struct dda__Ray {
    origin: vec3f,
    dir: vec3f,
}

fn _dda__umod3(a: vec3i, b: vec3i) -> vec3i {
    return ((a % b) + b) % b;
}

// Finds the grid boundary that first occurs along the given ray.
fn dda__step(t_max: vec3f, pos: vec3i, delta_t: vec3f, grid_step: vec3i) -> dda__StepResult {
    var step: dda__StepResult;
    step.t_max = t_max;
    step.snapped_pos = pos;
    if t_max.x < t_max.y {
        if t_max.x < t_max.z {
            step.t = t_max.x;
            step.t_max.x += delta_t.x;
            step.snapped_pos.x += grid_step.x;
            step.last_hit_axis = dda__X_AXIS;
        } else {
            step.t = t_max.z;
            step.t_max.z += delta_t.z;
            step.snapped_pos.z += grid_step.z;
            step.last_hit_axis = dda__Z_AXIS;
        }
    } else {
        if t_max.y < t_max.z {
            step.t = t_max.y;
            step.t_max.y += delta_t.y;
            step.snapped_pos.y += grid_step.y;
            step.last_hit_axis = dda__Y_AXIS;
        } else {
            step.t = t_max.z;
            step.t_max.z += delta_t.z;
            step.snapped_pos.z += grid_step.z;
            step.last_hit_axis = dda__Z_AXIS;
        }
    }
    return step;
}

// Finds the block id and keeps the uniform bit, if it exists.
fn dda__raw_block_id(pos: vec3i) -> u32 {
    if any(pos < vec3i(0)) { return 0u; }
    let chunk_pos = conversions__block_to_chunk(pos);
    let idx = world__idx(vec3u(chunk_pos));
    if idx == world__IDX_NONE { return 0u; }
    if (idx & world__UNIFORM_BIT) != 0 { return idx; }
    let local_pos = vec3u(_dda__umod3(pos, vec3i(i32(chunk__CHUNK_SIDE))));
    return chunk__get(idx, local_pos);
}

// Finds the first non-air block the ray hits.
fn dda__trace(ray: dda__Ray) -> dda__TraceResult {
    var snapped_pos = vec3i(floor(ray.origin));

    let raw_bid = dda__raw_block_id(snapped_pos);
    let bid = extractBits(raw_bid, 0u, 23u);
    if bid != 0u {
        return dda__TraceResult(ray.origin, bid, dda__NO_AXIS);
    }

    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);

    let next_boundary = vec3f(snapped_pos) + vec3f(grid_step > vec3i(0));
    let raw_t_max = (next_boundary - ray.origin) / ray.dir;

    var t_max = select(raw_t_max, vec3f(dda__INF), abs(ray.dir) < vec3f(dda__EPS));
    var last_hit_axis: u32;

    var iterations = 0u;
    loop {
        iterations++;
        if iterations > 1000u { break; }

        let res = dda__step(t_max, snapped_pos, delta_t, grid_step);
        t_max = res.t_max;
        snapped_pos = res.snapped_pos;
        last_hit_axis = res.last_hit_axis;

        if res.t > g_config.max_trace_dist { break; }

        let raw_bid = dda__raw_block_id(snapped_pos);
        let bid = extractBits(raw_bid, 0u, 23u);
        if bid != 0u { return dda__TraceResult(ray.origin + ray.dir * res.t, bid, res.last_hit_axis); }
        if (raw_bid & world__UNIFORM_BIT) == 0u { continue; }

        let chunk_local = _dda__umod3(snapped_pos, vec3i(i32(chunk__CHUNK_SIDE)));
        let to_edge = select(
            chunk_local + vec3i(1),
            vec3i(i32(chunk__CHUNK_SIDE)) - chunk_local,
            grid_step > vec3i(0)
        );
        let exit_t = t_max + (vec3f(to_edge) - vec3f(1.0)) * delta_t;
        let min_t = min(exit_t.x, min(exit_t.y, exit_t.z));
        let steps = select(vec3i(0), to_edge - vec3i(1), exit_t <= vec3f(min_t));

        snapped_pos += steps * grid_step;
        t_max += vec3f(steps) * delta_t;
    }

    return dda__TraceResult(vec3f(0.0), 0u, dda__NO_AXIS);
}

fn dda__axis_normal(axis: u32, dir: vec3f) -> vec3f {
    let s = -sign(dir);
    switch axis {
        case dda__X_AXIS: { return vec3f(s.x, 0.0, 0.0); }
        case dda__Y_AXIS: { return vec3f(0.0, s.y, 0.0); }
        case dda__Z_AXIS: { return vec3f(0.0, 0.0, s.z); }
        default: { return -normalize(dir); }
    }
}