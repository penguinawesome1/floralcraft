import package::config::IS_DEBUG_MODE;
import package::core::{blocks, pos, camera::Camera, block_target::BlockTarget};
import package::raytrace::bindings::config;
import package::raytrace::support::dda::{X_AXIS, Y_AXIS, NO_AXIS};
import package::raytrace::support::{dda, dda::Ray};
import package::raytrace::materials::{sky, pbr};

@group(0) @binding(4) var<storage, read_write> block_target: BlockTarget;
@group(1) @binding(0) var t_output: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(1) var<uniform> cam: Camera;

// Converts pixel coordinates to a normalized world-space ray direction.
// Used as the primary vector for grid traversal.
fn _px_to_dir(px: vec2u, canvas_size: vec2u, rotation: mat4x4f) -> vec3f {
    let aspect = f32(canvas_size.x) / f32(canvas_size.y);
    let uv = (vec2f(px) / vec2f(canvas_size)) * 2.0 - 1.0;
    let scaled = vec2f(uv.x * aspect, -uv.y);
    let view_dir = vec3f(scaled, -1.0);
    let world_dir = (rotation * vec4f(view_dir, 0.0)).xyz;
    return normalize(world_dir);
}

fn _debug_color(axis: u32) -> vec4f {
    switch axis {
        case X_AXIS: { return vec4f(1.0, 0.0, 0.0, 1.0); }
        case Y_AXIS: { return vec4f(0.0, 1.0, 0.0, 1.0); }
        default: { return vec4f(0.0, 0.0, 1.0, 1.0); }
    }
}

@compute @workgroup_size(8, 8, 1)
fn render(@builtin(global_invocation_id) px: vec3u) {
    let canvas_size = textureDimensions(t_output);
    if any(px.xy >= canvas_size) { return; }

    let dir = _px_to_dir(px.xy, canvas_size, cam.rotation);
    let ray = Ray(cam.pos, dir);
    let player_chunk = pos::chunk(vec3i(floor(cam.pos)));
    let res = dda::trace(ray, player_chunk);

    let crosshair_px = canvas_size / 2u;
    if all(px.xy == crosshair_px) {
        let is_valid_target = config.pending_action != 0u
            && res.block_id != blocks::AIR;

        block_target.is_block_hit = u32(is_valid_target);
        if is_valid_target {
            let axis_offset = vec3i(dda::axis_normal(res.last_hit_axis, ray.dir));
            block_target.pos = select(
                res.snapped_pos + axis_offset,
                res.snapped_pos,
                config.pending_action == 1u
            );
        }
    }

    var color: vec4f;
    if res.block_id == blocks::AIR {
        color = vec4f(sky::color(dir), 1.0);
    } else {
        if IS_DEBUG_MODE {
            color = _debug_color(res.last_hit_axis);
        } else {
            color = pbr::color(dir, res, cam.pos, player_chunk);
        }
    }
    textureStore(t_output, px.xy, color);
}
