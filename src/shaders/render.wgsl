struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
}

@group(0) @binding(0) var g_t_canvas: texture_2d<f32>;
@group(0) @binding(1) var g_s_canvas: sampler;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let x = f32((in_vertex_index & 1u) * 4u) - 1.0;
    let y = f32((in_vertex_index >> 1u) * 4u) - 1.0;

    return VertexOutput(
        vec4f(x, y, 0.0, 1.0),
        vec2f(
            f32((in_vertex_index & 1u) * 2u),
            1.0 - f32((in_vertex_index >> 1u) * 2u)
        )
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(g_t_canvas, g_s_canvas, in.uv);
}
