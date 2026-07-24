import package::config::config;
import package::core::{pos, blocks};
import package::raytrace::support::{lookup, read_pool};

const NO_AXIS = 0u;
const X_AXIS = 1u;
const Y_AXIS = 2u;
const Z_AXIS = 3u;
const INF = 1e30;
const EPS = 1e-4;

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
    distance: f32,
}

struct Ray {
    origin: vec3f,
    dir: vec3f,
}

// Finds the grid boundary that first occurs along the given ray.
fn _step(t_max: vec3f, pos: vec3i, delta_t: vec3f, grid_step: vec3i, scale: u32, ray: Ray) -> StepResult {
    let t = min(t_max.x, min(t_max.y, t_max.z));
    let step_x = t_max.x <= t;
    let step_y = t_max.y <= t;
    let step_z = t_max.z <= t;

    var s: StepResult;
    s.t = t;
    s.t_max = t_max;
    s.last_hit_axis = select(select(select(NO_AXIS, Z_AXIS, step_z), Y_AXIS, step_y), X_AXIS, step_x);

    if scale == 0u {
        s.snapped_pos = pos;
        if step_x {
            s.t_max.x += delta_t.x;
            s.snapped_pos.x += grid_step.x;
        }
        if step_y {
            s.t_max.y += delta_t.y;
            s.snapped_pos.y += grid_step.y;
        }
        if step_z {
            s.t_max.z += delta_t.z;
            s.snapped_pos.z += grid_step.z;
        }
    } else {
        let hit_pos = ray.origin + ray.dir * t;
        s.snapped_pos = vec3i(floor(hit_pos + ray.dir * EPS));
        let d_t = delta_t * f32(scale);
        if step_x { s.t_max.x += d_t.x; }
        if step_y { s.t_max.y += d_t.y; }
        if step_z { s.t_max.z += d_t.z; }
    }

    return s;
}

// Finds the first non-air block the ray hits.
fn trace(ray: Ray, player_chunk: vec3i) -> TraceResult {
    let origin = vec3i(floor(ray.origin));
    let grid_step = vec3i(sign(ray.dir));
    let delta_t = 1.0 / abs(ray.dir);
    let next_boundary = vec3f(origin) + vec3f(grid_step > vec3i(0));
    let raw_t_max = (next_boundary - ray.origin) / ray.dir;

    var snapped_pos = origin;
    var t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
    var t = 0.0;
    var last_hit_axis = NO_AXIS;

    var is_chunk_found_once = false;
    var chunk_pos = vec3i(0);
    var lookup_res: lookup::LookupResult;
    var prev_mip_level = 0u;
    var scale = 0u;

    for (var i = 0u; i < 3000u; i++) {
        let new_chunk_pos = pos::chunk(snapped_pos);
        if !is_chunk_found_once || any(new_chunk_pos != chunk_pos) {
            chunk_pos = new_chunk_pos;
            lookup_res = lookup::lookup(chunk_pos, player_chunk);
            if !lookup_res.is_loaded {
                return TraceResult(ray.origin, blocks::AIR, NO_AXIS, 0.0);
            }
            if !is_chunk_found_once || lookup_res.mip_level != prev_mip_level {
                scale = select(1u, 8u << (lookup_res.mip_level - 1u), lookup_res.mip_level > 0u);
                var next_boundary = vec3f(0);
                if lookup_res.mip_level == 0u {
                    next_boundary = vec3f(snapped_pos) + vec3f(grid_step > vec3i(0));
                } else {
                    let mip_cell_pos = snapped_pos & ~vec3i(i32(scale) - 1i);
                    next_boundary = vec3f(mip_cell_pos) + f32(scale) * vec3f(grid_step > vec3i(0));
                }
                let raw_t_max = (next_boundary - ray.origin) / ray.dir;
                t_max = select(raw_t_max, vec3f(INF), abs(ray.dir) < vec3f(EPS));
                prev_mip_level = lookup_res.mip_level;
            }
            is_chunk_found_once = true;
        }

        if lookup_res.mip_level == 0u {
            let local_pos = pos::local(snapped_pos);
            let bid = read_pool::block_id(lookup_res.chunk_idx, local_pos);
            if bid != blocks::AIR {
                return TraceResult(ray.origin + ray.dir * t, bid, last_hit_axis, t);
            }
        }

        let step_res = _step(t_max, snapped_pos, delta_t, grid_step, scale, ray);
        t_max = step_res.t_max;
        snapped_pos = step_res.snapped_pos;
        last_hit_axis = step_res.last_hit_axis;
        t = step_res.t;
        if t > config.max_trace_dist { break; }
    }

    return TraceResult(vec3f(0.0), blocks::AIR, NO_AXIS, config.max_trace_dist);
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
