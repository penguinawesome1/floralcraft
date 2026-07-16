import constants::{CHUNK_SIDE_SHIFT, BITS_PER_ID};

const CHUNK_SIDE = 1u << CHUNK_SIDE_SHIFT;
const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = CHUNK_VOLUME * BITS_PER_ID / 32u;

struct _ChunkAddr {
    word_idx: u32,
    bit_idx: u32,
}

@group(0) @binding(0) var<storage, read> chunk_pool: array<u32>;

fn _chunk_addr(local_pos: vec3u) -> _ChunkAddr {
    let linear_idx = local_pos.z * (CHUNK_SIDE * CHUNK_SIDE) + local_pos.x * CHUNK_SIDE + local_pos.y;
    let offset = linear_idx * BITS_PER_ID;
    return _ChunkAddr(offset / 32u, offset % 32u);
}

fn block_id(chunk_num: u32, local_pos: vec3u) -> u32 {
    let addr = _chunk_addr(local_pos);
    let chunk_base_idx = chunk_num * CHUNK_LEN;
    return extractBits(chunk_pool[chunk_base_idx + addr.word_idx], addr.bit_idx, BITS_PER_ID);
}