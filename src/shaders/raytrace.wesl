import package::config::{Config, IS_DEBUG_MODE};
import package::data::read::world::World;
import package::raytrace::conversions::px_to_dir;
import package::raytrace::dda::{Ray, trace, X_AXIS, Y_AXIS};
import package::raytrace::pbr::get_color;

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

@group(1) @binding(0) var t_output: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(1) var<uniform> cam: Camera;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) px: vec3u) {
    let canvas_size = textureDimensions(t_output);
    if any(px.xy >= canvas_size) { return; }

    let dir = px_to_dir(px.xy, canvas_size, cam.rotation);
    let ray = Ray(cam.pos, dir);
    let res = trace(ray);

    if res.block_id == 0 {
        textureStore(t_output, px.xy, vec4f(0.7, 0.75, 0.95, 1.0));
        return;
    }

    if IS_DEBUG_MODE {
        switch res.last_hit_axis {
            case X_AXIS: { textureStore(t_output, px.xy, vec4f(1.0, 0.0, 0.0, 1.0)); }
            case Y_AXIS: { textureStore(t_output, px.xy, vec4f(0.0, 1.0, 0.0, 1.0)); }
            default: { textureStore(t_output, px.xy, vec4f(0.0, 0.0, 1.0, 1.0)); }
        }
        return;
    }

    let color = get_color(dir, res);
    textureStore(t_output, px.xy, color);
}
