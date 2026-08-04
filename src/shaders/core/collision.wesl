import constants::CHUNK_SIDE;
import package::core::{pos, blocks, flags::ALL_AIR};
import package::render::traversal::read_pool;

const PLAYER_RADIUS = 0.3;
const PLAYER_HEIGHT = 1.8;
const PLAYER_EYE_OFFSET = 1.6;

struct CollisionResult {
    pos: vec3f,
    vel_y: f32,
    is_grounded: bool,
}

fn _is_solid(world_pos: vec3f, chunk_map: texture_3d<u32>) -> bool {
    let voxel_pos = vec3i(floor(world_pos));
    let chunk_pos = vec3i(floor(vec3f(voxel_pos) / f32(CHUNK_SIDE)));

    let value = textureLoad(chunk_map, pos::wrapped_chunk(chunk_pos), 0u).x;
    if value == 0u { return true; }
    if value == ALL_AIR { return false; }

    let bid = read_pool::block_id(value, pos::local(voxel_pos));
    return bid != blocks::AIR;
}

fn _is_solid_at_height(center: vec3f, y_offset: f32, chunk_map: texture_3d<u32>) -> bool {
    let p = vec3f(center.x, center.y + y_offset, center.z);
    let r = PLAYER_RADIUS;

    return _is_solid(p, chunk_map) ||
           _is_solid(p + vec3f(r, 0.0, 0.0), chunk_map) || _is_solid(p + vec3f(-r, 0.0, 0.0), chunk_map) ||
           _is_solid(p + vec3f(0.0, 0.0, r), chunk_map) || _is_solid(p + vec3f(0.0, 0.0, -r), chunk_map) ||
           _is_solid(p + vec3f(r, 0.0, r), chunk_map) || _is_solid(p + vec3f(r, 0.0, -r), chunk_map) ||
           _is_solid(p + vec3f(-r, 0.0, r), chunk_map) || _is_solid(p + vec3f(-r, 0.0, -r), chunk_map);
}

fn _is_solid_player_box(center: vec3f, chunk_map: texture_3d<u32>) -> bool {
    let feet_y = -PLAYER_EYE_OFFSET;
    let mid_y = feet_y + (PLAYER_HEIGHT * 0.5);
    let head_y = PLAYER_HEIGHT - PLAYER_EYE_OFFSET;

    return _is_solid_at_height(center, feet_y, chunk_map) || 
           _is_solid_at_height(center, mid_y, chunk_map) || 
           _is_solid_at_height(center, head_y, chunk_map);
}

fn _snap_to_surface(open_y: f32, blocked_y: f32, center: vec3f, chunk_map: texture_3d<u32>) -> f32 {
    var lo = open_y;
    var hi = blocked_y;
    for (var i = 0u; i < 8u; i++) {
        let mid = (lo + hi) * 0.5;
        if _is_solid_player_box(vec3f(center.x, mid, center.z), chunk_map) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    return lo;
}

fn resolve_collision(pos: vec3f, delta: vec3f, vel_y: f32, chunk_map: texture_3d<u32>) -> CollisionResult {
    var res = pos;
    var out_vel = vel_y;
    var grounded = false;

    if !_is_solid_player_box(res + vec3f(delta.x, 0.0, 0.0), chunk_map) { res.x += delta.x; }
    if !_is_solid_player_box(res + vec3f(0.0, 0.0, delta.z), chunk_map) { res.z += delta.z; }

    let next_y = res + vec3f(0.0, delta.y, 0.0);

    let feet_y = -PLAYER_EYE_OFFSET;
    let mid_y = feet_y + (PLAYER_HEIGHT * 0.5);
    let head_y = PLAYER_HEIGHT - PLAYER_EYE_OFFSET;

    let feet_blocked = _is_solid_at_height(next_y, feet_y, chunk_map);
    let mid_blocked = _is_solid_at_height(next_y, mid_y, chunk_map);
    let head_blocked = _is_solid_at_height(next_y, head_y, chunk_map);

    if feet_blocked || mid_blocked {
        res.y = _snap_to_surface(res.y, next_y.y, res, chunk_map);
        out_vel = 0.0;
        grounded = delta.y <= 0.0;
    } else if head_blocked && delta.y > 0.0 {
        res.y = res.y;
        out_vel = 0.0;
        grounded = false;
    } else {
        res.y += delta.y;
        grounded = false;
    }

    return CollisionResult(res, out_vel, grounded);
}

fn is_overlap(player_pos: vec3f, block_pos: vec3i) -> bool {
    let p_min_x = player_pos.x - PLAYER_RADIUS;
    let p_max_x = player_pos.x + PLAYER_RADIUS;

    let p_min_y = player_pos.y - PLAYER_EYE_OFFSET;
    let p_max_y = player_pos.y + (PLAYER_HEIGHT - PLAYER_EYE_OFFSET);

    let p_min_z = player_pos.z - PLAYER_RADIUS;
    let p_max_z = player_pos.z + PLAYER_RADIUS;

    let b_min_x = f32(block_pos.x);
    let b_max_x = b_min_x + 1.0;

    let b_min_y = f32(block_pos.y);
    let b_max_y = b_min_y + 1.0;

    let b_min_z = f32(block_pos.z);
    let b_max_z = b_min_z + 1.0;

    let intersect_x = p_min_x < b_max_x && p_max_x > b_min_x;
    let intersect_y = p_min_y < b_max_y && p_max_y > b_min_y;
    let intersect_z = p_min_z < b_max_z && p_max_z > b_min_z;

    return intersect_x && intersect_y && intersect_z;
}