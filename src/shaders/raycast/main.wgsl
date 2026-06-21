#include "../config.wgsl"
#include "../data/read/World.wgsl"
#include "./conversions.wgsl"
#include "./dda.wgsl"
#include "./pbr.wgsl"

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

@group(0) @binding(0) var<storage, read> g_world: world__World;
@group(1) @binding(0) var g_t_output: texture_storage_2d<rgba8unorm, write>;
@group(1) @binding(1) var<uniform> g_cam: Camera;
@group(1) @binding(2) var<uniform> g_config: config__Config;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) px: vec3u) {
    let canvas_size = textureDimensions(g_t_output);
    if any(px.xy >= canvas_size) { return; }

    let dir = conversions__px_to_dir(px.xy, canvas_size);
    let ray = dda__Ray(g_cam.pos, dir);
    let res = dda__trace(ray);

    if res.block_id == 0 {
        textureStore(g_t_output, px.xy, vec4f(0.7, 0.75, 0.95, 1.0));
        return;
    }

    if config__IS_DEBUG_MODE {
        switch res.last_hit_axis {
            case dda__X_AXIS: { textureStore(g_t_output, px.xy, vec4f(1.0, 0.0, 0.0, 1.0)); }
            case dda__Y_AXIS: { textureStore(g_t_output, px.xy, vec4f(0.0, 1.0, 0.0, 1.0)); }
            default: { textureStore(g_t_output, px.xy, vec4f(0.0, 0.0, 1.0, 1.0)); }
        }
        return;
    }

    let color = pbr__get(dir, res);
    textureStore(g_t_output, px.xy, color);
}
