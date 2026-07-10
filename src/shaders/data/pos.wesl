import constants::{GEN_SIDE, CHUNK_SIDE_SHIFT};

fn chunk(global_pos: vec3i) -> vec3i {
    return global_pos >> vec3u(CHUNK_SIDE_SHIFT);
}

fn wrapped_chunk(chunk_pos: vec3i) -> vec3u {
    return vec3u(chunk_pos); // TODO
}

fn local(global_pos: vec3i) -> vec3u {
    let mask = i32((1u << CHUNK_SIDE_SHIFT) - 1u);
    return vec3u(global_pos & vec3i(mask));
}

fn is_in_bounds(global_pos: vec3i) -> bool {
    let world_extent = i32(GEN_SIDE << CHUNK_SIDE_SHIFT);
    return all(global_pos >= vec3i(0)) && all(global_pos < vec3i(world_extent));
}