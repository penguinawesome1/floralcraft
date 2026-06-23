import package::data::read::chunk::{get_block_id, CHUNK_SIDE};
import package::data::read::world::{get_chunk_idx, IDX_NONE, UNIFORM_MASK, DEPTH_MASK, VALUE_MASK};
import package::raytrace::conversions::block_to_chunk;
import package::raytrace::config;

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

// Finds the block id and keeps the uniform bit, if it exists.
fn raw_block_id(pos: vec3i) -> u32 {
    if any(pos < vec3i(0)) { return 0u; }
    let chunk_pos = block_to_chunk(pos);
    let val = get_chunk_idx(vec3u(chunk_pos));
    if val == IDX_NONE { return 0u; }
    if (val & UNIFORM_MASK) != 0 { return (val & DEPTH_MASK) >> 18u; }
    let local_pos = vec3u(_umod3(pos, vec3i(i32(CHUNK_SIDE))));
    return get_block_id(val, local_pos);
}

// Finds the first non-air block the ray hits.
fn trace(ray: Ray) -> TraceResult {
    var snapped_pos = vec3i(floor(ray.origin));

    let raw_bid = raw_block_id(snapped_pos);
    let bid = raw_bid & VALUE_MASK;
    if bid != 0u {
        return TraceResult(ray.origin, bid, NO_AXIS);
    }

    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);

    let next_boundary = vec3f(snapped_pos) + vec3f(grid_step > vec3i(0));
    let raw_t_max = (next_boundary - ray.origin) / ray.dir;

    var t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
    var last_hit_axis: u32;

    var iterations = 0u;
    loop {
        iterations++;
        if iterations > 1000u { break; }

        let res = step(t_max, snapped_pos, delta_t, grid_step);
        t_max = res.t_max;
        snapped_pos = res.snapped_pos;
        last_hit_axis = res.last_hit_axis;

        if res.t > config.max_trace_dist { break; }

        let raw_bid = raw_block_id(snapped_pos);
        let bid = raw_bid & VALUE_MASK;
        if bid != 0u { return TraceResult(ray.origin + ray.dir * res.t, bid, res.last_hit_axis); }
        if (raw_bid & UNIFORM_MASK) == 0u { continue; }

        let chunk_local = _umod3(snapped_pos, vec3i(i32(CHUNK_SIDE)));
        let to_edge = select(
            chunk_local + vec3i(1),
            vec3i(i32(CHUNK_SIDE)) - chunk_local,
            grid_step > vec3i(0)
        );
        let exit_t = t_max + (vec3f(to_edge) - vec3f(1.0)) * delta_t;
        let min_t = min(exit_t.x, min(exit_t.y, exit_t.z));
        let steps = select(vec3i(0), to_edge - vec3i(1), exit_t <= vec3f(min_t));

        snapped_pos += steps * grid_step;
        t_max += vec3f(steps) * delta_t;
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
