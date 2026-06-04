const EPS: f32 = 1e-6;
const INF: f32 = 1e30;
const MAX_STEPS: u32 = 50;

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

@group(0) @binding(0) var t_output: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> u_cam: Camera;

fn block_id(pos: vec3i) -> u32 {
    if pos.y < 0 {
        return 1;
    }

    return 0;
}

fn px_to_dir(px: vec2u) -> vec3f {
    let canvas_size = textureDimensions(t_output);
    let aspect = f32(canvas_size.x) / f32(canvas_size.y);
    let ndc = vec2f(
        ((f32(px.x) / f32(canvas_size.x)) * 2.0 - 1.0) * aspect,
        1.0 - (f32(px.y) / f32(canvas_size.y)) * 2.0,
    );

    return normalize((u_cam.rotation * vec4f(ndc, 2.0, 0.0)).xyz);
}

fn init_t_max(pos: vec3f, dir: vec3f) -> vec3f {
    let frac = fract(pos);
    let to_next = select(frac, vec3f(1.0) - frac, dir > vec3f(0.0));
    let safe = select(to_next, vec3f(1.0), to_next < vec3f(EPS));
    return select(safe / abs(dir), vec3f(INF), abs(dir) < vec3f(EPS));
}

fn march(pos: vec3f, dir: vec3f) -> vec3i {
    var block = vec3i(floor(pos));
    let step = vec3i(sign(dir));
    let t_delta = select(abs(1.0 / dir), vec3f(INF), abs(dir) < vec3f(EPS));
    var t_max = init_t_max(pos, dir);

    for (var i = 0u; i < MAX_STEPS; i++) {
        if block_id(block) != 0u { return block; }

        if t_max.x < t_max.y && t_max.x < t_max.z {
            block.x += step.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            block.y += step.y;
            t_max.y += t_delta.y;
        } else {
            block.z += step.z;
            t_max.z += t_delta.z;
        }
    }

    return vec3i(-1);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) px: vec3u) {
    let canvas_size = textureDimensions(t_output);
    if any(px.xy >= canvas_size) {
        return;
    }

    let dir = px_to_dir(px.xy);

    var color = vec4f(0.5, 0.8, 0.9, 1.0);
    var curr_pos = u_cam.pos;

    for (var i = 0u; i < 50u; i++) {
        curr_pos += dir * 0.5;

        let block_pos = vec3i(floor(curr_pos));
        let block_id = block_id(block_pos);

        if block_id == 0 {
            continue;
        }

        color = vec4f(0.6, 0.9, 0.0, 1.0);
        break;
    }

    textureStore(t_output, px.xy, color);
}