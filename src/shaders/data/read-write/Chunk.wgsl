#include "../../config.wgsl"

const chunk__CHUNK_SIDE = 1u << config__CHUNK_SIDE_SHIFT;
const chunk__CHUNK_VOLUME = chunk__CHUNK_SIDE * chunk__CHUNK_SIDE * chunk__CHUNK_SIDE;
const chunk__CHUNK_LEN = (chunk__CHUNK_VOLUME * config__BITS_PER_ID + 31u) / 32u;
const chunk__IDS_PER_GROUP = 32u / config__BITS_PER_ID;
const chunk__ID_MASK = (1u << config__BITS_PER_ID) - 1u;
const _chunk__BROADCAST_MULT = 0xFFFFFFFFu / chunk__ID_MASK;
const _chunk__ZERO_IDX = 0xFFFFFFFFu;

alias chunk__Chunk = array<atomic<u32>, chunk__CHUNK_LEN>;

struct _chunk__ChunkAddr {
    word_idx: u32,
    bit_idx: u32,
}

fn _chunk__pos_to_addr(pos: vec3u) -> _chunk__ChunkAddr {
    let linear_idx = (pos.z * (chunk__CHUNK_SIDE * chunk__CHUNK_SIDE) + pos.y * chunk__CHUNK_SIDE + pos.x);
    let offset = linear_idx * config__BITS_PER_ID;
    return _chunk__ChunkAddr(offset >> 5u, offset & 31u);
}

// Removes a chunk from the free list and returns its index.
// The chunk is not cleared.
fn chunk__free_pop() -> u32 {
    var old_free = atomicLoad(&g_world.chunks_free);
    loop {
        var new_free = atomicLoad(&g_world.chunks[old_free][0]);
        new_free = select(new_free, old_free + 1u, new_free == 0u);
        new_free = select(new_free, 0u, new_free == _chunk__ZERO_IDX);
        let res = atomicCompareExchangeWeak(&g_world.chunks_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    return old_free;
}

fn chunk__free_push(idx: u32) {
    var old_free = atomicLoad(&g_world.chunks_free);
    loop {
        let true_old_free = select(old_free, _chunk__ZERO_IDX, old_free == 0u);
        atomicStore(&g_world.chunks[idx][0], true_old_free);
        let res = atomicCompareExchangeWeak(&g_world.chunks_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

fn chunk__get(chunk_idx: u32, pos: vec3u) -> u32 {
    let addr = _chunk__pos_to_addr(pos);
    let word = atomicLoad(&g_world.chunks[chunk_idx][addr.word_idx]);
    return extractBits(word, addr.bit_idx, config__BITS_PER_ID);
}

fn chunk__set(chunk_idx: u32, pos: vec3u, id: u32) {
    let addr = _chunk__pos_to_addr(pos);
    var old_word = atomicLoad(&g_world.chunks[chunk_idx][addr.word_idx]);
    loop {
        let new_word = insertBits(old_word, id, addr.bit_idx, config__BITS_PER_ID);
        let res = atomicCompareExchangeWeak(&g_world.chunks[chunk_idx][addr.word_idx], old_word, new_word);
        if res.exchanged { break; }
        old_word = res.old_value;
    }
}

fn chunk__fill(chunk_idx: u32, id: u32) {
    var filled_word = 0u;
    for (var slot = 0u; slot < chunk__IDS_PER_GROUP; slot++) {
        filled_word = insertBits(filled_word, id, slot * config__BITS_PER_ID, config__BITS_PER_ID);
    }
    for (var i = 0u; i < chunk__CHUNK_LEN; i++) {
        atomicStore(&g_world.chunks[chunk_idx][i], filled_word);
    }
}

fn chunk__is_empty(chunk_idx: u32) -> bool {
    var acc = 0u;
    for (var i = 0u; i < chunk__CHUNK_LEN; i++) {
        acc |= atomicLoad(&g_world.chunks[chunk_idx][i]);
    }
    return acc == 0u;
}

fn chunk__is_uniform(chunk_idx: u32) -> bool {
    let first_id = atomicLoad(&g_world.chunks[chunk_idx][0]) & chunk__ID_MASK;
    let expected_pattern = first_id * _chunk__BROADCAST_MULT;
    var acc = 0u;
    for (var i = 0u; i < chunk__CHUNK_LEN; i++) {
        acc |= (atomicLoad(&g_world.chunks[chunk_idx][i]) ^ expected_pattern);
    }
    return acc == 0u;
}
