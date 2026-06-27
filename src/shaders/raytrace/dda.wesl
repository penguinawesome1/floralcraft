import package::data::read::chunk::{block_id, CHUNK_SIDE};
import package::data::read::world::{lookup, LookupResult, IDX_NONE, UNIFORM_MASK, DEPTH_MASK, VALUE_MASK, DEPTH_SHIFT};
import package::raytrace::conversions::block_to_chunk;
import package::config::{config, SVO_DEPTH};
import package::data::blocks;

const NO_AXIS = 0u;
const X_AXIS = 1u;
const Y_AXIS = 2u;
const Z_AXIS = 3u;
const INF = 1e30;
const EPS = 1e-6;

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

struct Ray {
    origin: vec3f,
    dir: vec3f,
}

struct BlockResult {
    is_uniform: bool,
    id: u32,
    depth: u32,
}

fn _umod3(a: vec3i, b: vec3i) -> vec3i {
    return ((a % b) + b) % b;
}

// Finds the grid boundary that first occurs along the given ray.
fn step(t_max: vec3f, pos: vec3i, delta_t: vec3f, grid_step: vec3i) -> StepResult {
    var s: StepResult;
    s.t_max = t_max;
    s.snapped_pos = pos;
    if t_max.x < t_max.y {
        if t_max.x < t_max.z {
            s.t = t_max.x;
            s.t_max.x += delta_t.x;
            s.snapped_pos.x += grid_step.x;
            s.last_hit_axis = X_AXIS;
        } else {
            s.t = t_max.z;
            s.t_max.z += delta_t.z;
            s.snapped_pos.z += grid_step.z;
            s.last_hit_axis = Z_AXIS;
        }
    } else {
        if t_max.y < t_max.z {
            s.t = t_max.y;
            s.t_max.y += delta_t.y;
            s.snapped_pos.y += grid_step.y;
            s.last_hit_axis = Y_AXIS;
        } else {
            s.t = t_max.z;
            s.t_max.z += delta_t.z;
            s.snapped_pos.z += grid_step.z;
            s.last_hit_axis = Z_AXIS;
        }
    }
    return s;
}

fn block(pos: vec3i) -> LookupResult {
    let chunk_pos = vec3u(block_to_chunk(pos));
    let res = lookup(chunk_pos);
    if res.height != 0u { return res; }
    let local_pos = vec3u(_umod3(pos, vec3i(i32(CHUNK_SIDE))));
    return LookupResult(0u, block_id(res.value, local_pos));
}

// Finds the first non-air block the ray hits.
fn trace(ray: Ray) -> TraceResult {
    var snapped_pos = vec3i(floor(ray.origin));

    let block_res = block(snapped_pos);
    if block_res.value != 0u {
        return TraceResult(ray.origin, block_res.value, NO_AXIS);
    }

    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);

    let next_boundary = vec3f(snapped_pos) + vec3f(grid_step > vec3i(0));
    let raw_t_max = (next_boundary - ray.origin) / ray.dir;

    var t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
    var last_hit_axis: u32;

    for (var i = 0u; i < 1000u; i++) {
        let step_res = step(t_max, snapped_pos, delta_t, grid_step);
        t_max = step_res.t_max;
        snapped_pos = step_res.snapped_pos;
        last_hit_axis = step_res.last_hit_axis;

        if step_res.t > config.max_trace_dist { break; }

        let block_res = block(snapped_pos);
        if block_res.value != blocks::AIR {
            return TraceResult(ray.origin + ray.dir * step_res.t, block_res.value, step_res.last_hit_axis);
        }
        // if block_res.height == blocks::AIR { continue; }

        // let chunk_local = _umod3(snapped_pos, vec3i(i32(CHUNK_SIDE)));
        // let to_edge = select(
        //     chunk_local + vec3i(1),
        //     vec3i(i32(CHUNK_SIDE)) - chunk_local,
        //     grid_step > vec3i(0)
        // );
        // let exit_t = t_max + (vec3f(to_edge) - vec3f(1.0)) * delta_t;
        // let min_t = min(exit_t.x, min(exit_t.y, exit_t.z));
        // let steps = select(vec3i(0), to_edge - vec3i(1), exit_t <= vec3f(min_t));

        // snapped_pos += steps * grid_step;
        // t_max += vec3f(steps) * delta_t;
    }

    return TraceResult(vec3f(0.0), blocks::AIR, NO_AXIS);
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
