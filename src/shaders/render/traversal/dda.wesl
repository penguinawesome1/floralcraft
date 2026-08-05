import constants::CHUNK_SIDE;
import package::core::{pos, blocks};
import package::render::traversal::{lookup, read_pool};

const NO_AXIS = 0u;
const X_AXIS = 1u;
const Y_AXIS = 2u;
const Z_AXIS = 3u;
const INF = 1e30;
const EPS = 1e-4;

struct StepResult {
    t_max: vec3f,
    hit_pos: vec3f,
    snapped_pos: vec3i,
    t: f32,
    last_hit_axis: u32,
}

struct TraceResult {
    hit_pos: vec3f,
    snapped_pos: vec3i,
    distance: f32,
    block_id: u32,
    last_hit_axis: u32,
}

struct Ray {
    origin: vec3f,
    dir: vec3f,
}

// Finds the grid boundary that first occurs along the given ray.
fn _step(
    t_max: vec3f,
    hit_pos: vec3f,
    snapped_pos: vec3i,
    delta_t: vec3f,
    grid_step: vec3i,
    scale: u32,
    ray: Ray
) -> StepResult {
    let t = min(t_max.x, min(t_max.y, t_max.z));
    let step_mask = t_max <= vec3f(t);

    var s: StepResult;
    s.t = t;
    s.t_max = t_max;
    s.last_hit_axis = select(select(select(NO_AXIS, Z_AXIS, step_mask.z), Y_AXIS, step_mask.y), X_AXIS, step_mask.x);
    s.hit_pos = ray.origin + ray.dir * t;

    if scale == 1u {
        s.t_max = t_max + select(vec3f(0.0), delta_t, step_mask);
        s.snapped_pos = snapped_pos + select(vec3i(0), grid_step, step_mask);
    } else {
        s.t_max = t_max + select(vec3f(0.0), delta_t * f32(scale), step_mask);

        let safe_hit = ray.origin + ray.dir * (t + EPS);
        let next_snapped = vec3i(floor(safe_hit));

        let step_scale = i32(scale);
        let chunk_base = snapped_pos & ~vec3i(step_scale - 1i);
        let exact_boundary = chunk_base + select(vec3i(-1), vec3i(step_scale), grid_step > vec3i(0));

        s.snapped_pos = select(next_snapped, exact_boundary, step_mask);
    }

    return s;
}

// Finds the first non-air block the ray hits.
fn trace(ray: Ray, max_trace_dist: f32) -> TraceResult {
    let origin = vec3i(floor(ray.origin));
    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);
    let next_boundary = vec3f(origin) + vec3f(grid_step > vec3i(0));
    let inv_dir = 1.0 / ray.dir;
    let raw_t_max = (next_boundary - ray.origin) * inv_dir;

    var snapped_pos = origin;
    var t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
    var t = 0.0;
    var hit_pos = ray.origin;
    var last_hit_axis = NO_AXIS;

    var is_chunk_found_once = false;
    var chunk_pos = vec3i(0);
    var lookup_res: lookup::LookupResult;
    var prev_mip_level = 0u;
    var scale = 0u;

    for (var i = 0u; i < 2000u; i++) {
        let new_chunk_pos = pos::chunk(snapped_pos);
        if !is_chunk_found_once || any(new_chunk_pos != chunk_pos) {
            chunk_pos = new_chunk_pos;
            lookup_res = lookup::lookup(chunk_pos);
            if !lookup_res.is_loaded {
                return TraceResult(ray.origin, snapped_pos, 0.0, blocks::AIR, NO_AXIS);
            }
            if !is_chunk_found_once || lookup_res.mip_level != prev_mip_level {
                scale = select(1u, u32(CHUNK_SIDE) << (lookup_res.mip_level - 1u), lookup_res.mip_level > 0u);
                var next_boundary = vec3i(0);
                if lookup_res.mip_level == 0u {
                    next_boundary = snapped_pos + vec3i(grid_step > vec3i(0));
                } else {
                    let mip_cell_pos = snapped_pos & ~vec3i(i32(scale) - 1i);
                    next_boundary = mip_cell_pos + i32(scale) * vec3i(grid_step > vec3i(0));
                }

                let raw_t_max = (vec3f(next_boundary) - ray.origin) * inv_dir;
                t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
                prev_mip_level = lookup_res.mip_level;
            }
            is_chunk_found_once = true;
        }

        if lookup_res.mip_level == 0u {
            let local_pos = pos::local(snapped_pos);
            let bid = read_pool::block_id(lookup_res.chunk_idx, local_pos);
            if bid != blocks::AIR {
                return TraceResult(hit_pos, snapped_pos, t, bid, last_hit_axis);
            }
        }

        let step_res = _step(t_max, hit_pos, snapped_pos, delta_t, grid_step, scale, ray);
        t_max = step_res.t_max;
        snapped_pos = step_res.snapped_pos;
        hit_pos = step_res.hit_pos;
        last_hit_axis = step_res.last_hit_axis;
        t = step_res.t;
        if t > max_trace_dist { break; }
    }

    return TraceResult(vec3f(0.0), snapped_pos, max_trace_dist, blocks::AIR, NO_AXIS);
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
