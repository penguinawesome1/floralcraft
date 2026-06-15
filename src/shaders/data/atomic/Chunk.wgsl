const CHUNK_SIDE = 8u;
const BITS_PER_ID = 8u;
const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = (CHUNK_VOLUME * BITS_PER_ID + 31u) / 32u;
const IDS_PER_GROUP = 32u / BITS_PER_ID;
const ID_MASK = (1u << BITS_PER_ID) - 1u;
const BROADCAST_MULT = 0xFFFFFFFFu / ID_MASK;

const_assert CHUNK_SIDE > 0u;
const_assert 32u % BITS_PER_ID == 0u;

alias Chunk = array<atomic<u32>, CHUNK_LEN>;

struct ChunkAddr {
    word_idx: u32,
    bit_idx: u32,
}

fn _chunk_pos_to_addr(pos: vec3u) -> ChunkAddr {
    let offset = (pos.z * (CHUNK_SIDE * CHUNK_SIDE) + pos.y * CHUNK_SIDE + pos.x) * BITS_PER_ID;
    return ChunkAddr(offset >> 5u, offset & 31u);
}

fn chunk_clear(chunk_idx: u32) {
    for (var i = 0u; i < CHUNK_LEN; i++) {
        atomicStore(&world.chunks[chunk_idx][i], 0u);
    }
}

fn chunk_get(chunk_idx: u32, pos: vec3u) -> u32 {
    let addr = _chunk_pos_to_addr(pos);
    let word = atomicLoad(&world.chunks[chunk_idx][addr.word_idx]);
    return extractBits(word, addr.bit_idx, BITS_PER_ID);
}

fn chunk_set(chunk_idx: u32, pos: vec3u, id: u32) {
    let addr = _chunk_pos_to_addr(pos);
    var old_word = atomicLoad(&world.chunks[chunk_idx][addr.word_idx]);
    loop {
        let new_word = insertBits(old_word, id, addr.bit_idx, BITS_PER_ID);
        let res = atomicCompareExchangeWeak(&world.chunks[chunk_idx][addr.word_idx], old_word, new_word);
        if res.exchanged { break; }
        old_word = res.old_value;
    }
}

fn chunk_is_empty(chunk_idx: u32) -> bool {
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= atomicLoad(&world.chunks[chunk_idx][i]);
    }
    return acc == 0u;
}

fn chunk_is_uniform(chunk_idx: u32) -> bool {
    let first_id = atomicLoad(&world.chunks[chunk_idx][0]) & ID_MASK;
    let expected_pattern = first_id * BROADCAST_MULT;
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= (atomicLoad(&world.chunks[chunk_idx][i]) ^ expected_pattern);
    }
    return acc == 0u;
}
