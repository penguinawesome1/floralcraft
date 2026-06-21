#include "../../config.wgsl"

const chunk__CHUNK_SIDE = 1u << config__CHUNK_SIDE_SHIFT;
const chunk__CHUNK_VOLUME = chunk__CHUNK_SIDE * chunk__CHUNK_SIDE * chunk__CHUNK_SIDE;
const chunk__CHUNK_LEN = (chunk__CHUNK_VOLUME * config__BITS_PER_ID + 31u) / 32u;
const chunk__IDS_PER_GROUP = 32u / config__BITS_PER_ID;
const chunk__ID_MASK = (1u << config__BITS_PER_ID) - 1u;
const _chunk__BROADCAST_MULT = 0xFFFFFFFFu / chunk__ID_MASK;

alias chunk__Chunk = array<u32, chunk__CHUNK_LEN>;

// Finds the total bit offset of the pos within the chunk.
fn _chunk__pos_to_offset(pos: vec3u) -> u32 {
    let linear_idx = pos.z * (chunk__CHUNK_SIDE * chunk__CHUNK_SIDE) + pos.y * chunk__CHUNK_SIDE + pos.x;
    return linear_idx * config__BITS_PER_ID;
}

fn chunk__get(chunk_idx: u32, pos: vec3u) -> u32 {
    let offset = _chunk__pos_to_offset(pos);
    let word_idx = offset >> 5u;
    let bit_idx = offset & 31u;
    return extractBits(g_world.chunks[chunk_idx][word_idx], bit_idx, config__BITS_PER_ID);
}

fn chunk__is_empty(chunk_idx: u32) -> bool {
    var acc = 0u;
    for (var i = 0u; i < chunk__CHUNK_LEN; i++) {
        acc |= g_world.chunks[chunk_idx][i];
    }
    return acc == 0u;
}

fn chunk__is_uniform(chunk_idx: u32) -> bool {
    let first_id = g_world.chunks[chunk_idx][0] & chunk__ID_MASK;
    let expected_pattern = first_id * _chunk__BROADCAST_MULT;
    var acc = 0u;
    for (var i = 0u; i < chunk__CHUNK_LEN; i++) {
        acc |= g_world.chunks[chunk_idx][i] ^ expected_pattern;
    }
    return acc == 0u;
}
