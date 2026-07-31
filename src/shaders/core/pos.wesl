import constants::{GEN_SIDE, CHUNK_SIDE};

fn chunk(global_pos: vec3i) -> vec3i {
    let shift = u32(firstTrailingBit(CHUNK_SIDE));
    return global_pos >> vec3u(shift);
}

fn wrapped_chunk(chunk_pos: vec3i) -> vec3u {
    let mask = vec3i(i32(GEN_SIDE) - 1i);
    return vec3u(chunk_pos & mask);
}

fn unwrapped_chunk(wrapped: vec3u, player_chunk: vec3i) -> vec3i {
    let side = i32(GEN_SIDE);
    let half = side / 2;
    let p_wrap = ((player_chunk % side) + side) % side;
    let offset = ((vec3i(wrapped) - p_wrap + half + side) % side) - half;
    return player_chunk + offset;
}

fn is_in_range(chunk_pos: vec3i, player_chunk: vec3i) -> bool {
    let half = i32(GEN_SIDE) / 2i;
    let diff = chunk_pos - player_chunk;
    return diff.x >= -half && diff.x <= half - 1i &&
           diff.y >= -half && diff.y <= half - 1i &&
           diff.z >= -half && diff.z <= half - 1i;
}

fn local(global_pos: vec3i) -> vec3u {
    let mask = i32(CHUNK_SIDE - 1u);
    return vec3u(global_pos & vec3i(mask));
}