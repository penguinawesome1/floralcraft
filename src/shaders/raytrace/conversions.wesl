import package::config::CHUNK_SIDE_SHIFT;

// Converts pixel coordinates to a normalized world-space ray direction.
// Used as the primary vector for grid traversal.
fn px_to_dir(px: vec2u, canvas_size: vec2u, rotation: mat4x4f) -> vec3f {
    let aspect = f32(canvas_size.x) / f32(canvas_size.y);
    let uv = (vec2f(px) / vec2f(canvas_size)) * 2.0 - 1.0;
    let scaled = vec2f(uv.x * aspect, -uv.y);
    let view_dir = vec3f(scaled, -1.0);
    let world_dir = (rotation * vec4f(view_dir, 0.0)).xyz;
    return normalize(world_dir);
}

fn block_to_chunk(pos: vec3i) -> vec3i {
    return pos >> vec3u(CHUNK_SIDE_SHIFT);
}
