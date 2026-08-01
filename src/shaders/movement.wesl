import constants::CHUNK_SIDE;
import package::config::Config;
import package::core::{pos, blocks, player::Player, flags::ALL_AIR};
import package::render::traversal::read_pool;

const _ARROW_UP_SHIFT = 2u;
const _ARROW_DOWN_SHIFT = 3u;
const _ARROW_LEFT_SHIFT = 4u;
const _ARROW_RIGHT_SHIFT = 5u;
const _SPACE_SHIFT = 6u;
const _LEFT_SHIFT_SHIFT = 7u;

const _WALK_SPEED = 4.317;
const _SPRINT_SPEED = 7.5; 
const _FLY_SPEED = 25.0;
const _PLAYER_RADIUS = 0.3;
const _PLAYER_HEIGHT = 1.8;
const _PLAYER_EYE_OFFSET = 1.6;
const _GRAVITY_PER_TICK = -0.08;
const _DRAG_PER_TICK = 0.98;
const _TICKS_PER_SECOND = 20.0;
const _JUMP_IMPULSE = 0.46 * _TICKS_PER_SECOND;

const IS_GROUNDED_MASK = 1u;
const IS_FLYING_MASK = 2u;
const WAS_SPACE_HELD_MASK = 4u;

@group(0) @binding(0) var<uniform> config: Config;
@group(0) @binding(1) var<storage, read_write> player: Player;
@group(0) @binding(3) var chunk_index_map: texture_3d<u32>;

fn _is_pressed(shift: u32) -> bool {
    return ((config.pending_action >> shift) & 1u) == 1u;
}

fn _axis(neg_shift: u32, pos_shift: u32) -> f32 {
    return select(0.0, 1.0, _is_pressed(pos_shift)) - select(0.0, 1.0, _is_pressed(neg_shift));
}

fn _is_solid(world_pos: vec3f) -> bool {
    let voxel_pos = vec3i(floor(world_pos));
    let chunk_pos = vec3i(floor(vec3f(voxel_pos) / f32(CHUNK_SIDE)));

    let value = textureLoad(chunk_index_map, pos::wrapped_chunk(chunk_pos), 0u).x;
    if value == 0u { return true; }
    if value == ALL_AIR { return false; }

    let bid = read_pool::block_id(value, pos::local(voxel_pos));
    return bid != blocks::AIR;
}

fn _is_solid_at_height(center: vec3f, y_offset: f32) -> bool {
    let p = vec3f(center.x, center.y + y_offset, center.z);
    let r = _PLAYER_RADIUS;

    return _is_solid(p) ||
           _is_solid(p + vec3f(r, 0.0, 0.0)) || _is_solid(p + vec3f(-r, 0.0, 0.0)) ||
           _is_solid(p + vec3f(0.0, 0.0, r)) || _is_solid(p + vec3f(0.0, 0.0, -r)) ||
           _is_solid(p + vec3f(r, 0.0, r)) || _is_solid(p + vec3f(r, 0.0, -r)) ||
           _is_solid(p + vec3f(-r, 0.0, r)) || _is_solid(p + vec3f(-r, 0.0, -r));
}

fn _is_solid_player_box(center: vec3f) -> bool {
    let feet_y = -_PLAYER_EYE_OFFSET;
    let mid_y = feet_y + (_PLAYER_HEIGHT * 0.5);
    let head_y = _PLAYER_HEIGHT - _PLAYER_EYE_OFFSET;

    return _is_solid_at_height(center, feet_y) || 
           _is_solid_at_height(center, mid_y) || 
           _is_solid_at_height(center, head_y);
}

fn _snap_to_surface(open_y: f32, blocked_y: f32, center: vec3f) -> f32 {
    var lo = open_y;
    var hi = blocked_y;
    for (var i = 0u; i < 8u; i++) {
        let mid = (lo + hi) * 0.5;
        if _is_solid_player_box(vec3f(center.x, mid, center.z)) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    return lo;
}

fn _resolve_collision(pos: vec3f, delta: vec3f, is_grounded: ptr<function, bool>) -> vec3f {
    var res = pos;
    *is_grounded = false;

    if !_is_solid_player_box(res + vec3f(delta.x, 0.0, 0.0)) { res.x += delta.x; }
    if !_is_solid_player_box(res + vec3f(0.0, 0.0, delta.z)) { res.z += delta.z; }

    let next_y = res + vec3f(0.0, delta.y, 0.0);
    if _is_solid_player_box(next_y) {
        res.y = _snap_to_surface(res.y, next_y.y, res);
        player.velocity.y = 0.0;
        *is_grounded = delta.y <= 0.0;
    } else {
        res.y += delta.y;
    }

    return res;
}

fn _apply_gravity(velocity_y: f32, dt: f32) -> f32 {
    let ticks = dt * _TICKS_PER_SECOND;
    let drag_factor = pow(_DRAG_PER_TICK, ticks);
    let gravity = _GRAVITY_PER_TICK * _TICKS_PER_SECOND * (1.0 - drag_factor) / (1.0 - _DRAG_PER_TICK);
    return velocity_y * drag_factor + gravity;
}

@compute @workgroup_size(1, 1, 1)
fn move_player() {
    let forward_input = _axis(_ARROW_UP_SHIFT, _ARROW_DOWN_SHIFT);
    let right_input = _axis(_ARROW_LEFT_SHIFT, _ARROW_RIGHT_SHIFT);
    let space_held = _is_pressed(_SPACE_SHIFT);
    let shift_held = _is_pressed(_LEFT_SHIFT_SHIFT);

    var is_grounded = (player.state_flags & IS_GROUNDED_MASK) != 0u;
    var is_flying = (player.state_flags & IS_FLYING_MASK) != 0u;
    let was_space_held = (player.state_flags & WAS_SPACE_HELD_MASK) != 0u;
    var timer_ms = player.fly_timer_ms;

    if is_grounded { is_flying = false; }

    let dt_ms = u32(max(0.0, config.delta_time * 1000.0));
    timer_ms = select(0u, timer_ms - dt_ms, timer_ms > dt_ms);

    let space_just_pressed = space_held && !was_space_held;

    if space_just_pressed && timer_ms > 0u {
        is_flying = !is_flying;
        if is_flying { player.velocity = vec3f(0.0); }
        timer_ms = 0u;
    } else if space_just_pressed {
        timer_ms = 300u;
    }

    if space_held && !is_flying && is_grounded {
        player.velocity.y = _JUMP_IMPULSE;
        is_grounded = false;
    }

    let forward = vec3f(sin(config.cam_yaw), 0.0, cos(config.cam_yaw));
    let right = vec3f(cos(config.cam_yaw), 0.0, -sin(config.cam_yaw));

    var move_dir = forward * forward_input + right * right_input;
    if length(move_dir) > 0.0 { move_dir = normalize(move_dir); }

    if is_flying {
        let vert = select(0.0, 1.0, space_held) - select(0.0, 1.0, shift_held);
        var fly_dir = move_dir + vec3f(0.0, vert, 0.0);
        if length(fly_dir) > 0.0 { fly_dir = normalize(fly_dir); }

        let delta = fly_dir * _FLY_SPEED * config.delta_time;
        player.pos = _resolve_collision(player.pos, delta, &is_grounded);
        player.velocity = vec3f(0.0);
    } else {
        player.velocity.y = _apply_gravity(player.velocity.y, config.delta_time);

        let speed = select(_WALK_SPEED, _SPRINT_SPEED, shift_held && forward_input < 0.0);
        let vel = move_dir * speed;
        let delta = vec3f(vel.x, player.velocity.y, vel.z) * config.delta_time;

        player.pos = _resolve_collision(player.pos, delta, &is_grounded);
    }

    player.fly_timer_ms = timer_ms;
    player.state_flags = select(0u, IS_GROUNDED_MASK, is_grounded)
                       | select(0u, IS_FLYING_MASK, is_flying)
                       | select(0u, WAS_SPACE_HELD_MASK, space_held);
}