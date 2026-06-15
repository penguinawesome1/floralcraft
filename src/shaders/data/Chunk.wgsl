const CHUNK_SIDE = 8u;
const BITS_PER_ID = 8u;
const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = (CHUNK_VOLUME * BITS_PER_ID + 31u) / 32u;
const IDS_PER_GROUP = 32u / BITS_PER_ID;
const ID_MASK = (1u << BITS_PER_ID) - 1u;
const BROADCAST_MULT = 0xFFFFFFFFu / ID_MASK;

const_assert CHUNK_SIDE > 0u;
const_assert 32u % BITS_PER_ID == 0u;

alias Chunk = array<u32, CHUNK_LEN>;

// Finds the total bit offset of the pos within the chunk.
fn _chunk_pos_to_offset(pos: vec3u) -> u32 {
    return (pos.z * (CHUNK_SIDE * CHUNK_SIDE) + pos.y * CHUNK_SIDE + pos.x) * BITS_PER_ID;
}

fn chunk_get(chunk_idx: u32, pos: vec3u) -> u32 {
    let offset = _chunk_pos_to_offset(pos);
    return extractBits(world.chunks[chunk_idx][offset / 32u], offset % 32u, BITS_PER_ID);
}

fn chunk_is_empty(chunk_idx: u32) -> bool {
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= world.chunks[chunk_idx][i];
    }
    return acc == 0u;
}

fn chunk_is_uniform(chunk_idx: u32) -> bool {
    let first_id = world.chunks[chunk_idx][0] & ID_MASK;
    let expected_pattern = first_id * BROADCAST_MULT;
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= world.chunks[chunk_idx][i] ^ expected_pattern;
    }
    return acc == 0u;
}
