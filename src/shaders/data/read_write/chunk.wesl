import package::config::{CHUNK_SIDE_SHIFT, BITS_PER_ID, CHUNKS_CAPACITY};
import package::data::read_write::world::world;

const CHUNK_SIDE = 1u << CHUNK_SIDE_SHIFT;
const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = (CHUNK_VOLUME * BITS_PER_ID + 31u) / 32u;
const IDS_PER_GROUP = 32u / BITS_PER_ID;
const ID_MASK = (1u << BITS_PER_ID) - 1u;
const IS_NOT_UNIFORM = 0xFFFFFFFFu;
const _BROADCAST_MULT = 0xFFFFFFFFu / ID_MASK;

alias Chunk = array<atomic<u32>, CHUNK_LEN>;

struct _ChunkAddr {
    word_idx: u32,
    bit_idx: u32,
}

fn _pos_to_addr(pos: vec3u) -> _ChunkAddr {
    let linear_idx = pos.z * (CHUNK_SIDE * CHUNK_SIDE) + pos.y * CHUNK_SIDE + pos.x;
    let offset = linear_idx * BITS_PER_ID;
    return _ChunkAddr(offset >> 5u, offset & 31u);
}

// Removes a chunk from the free list and returns its index.
// The chunk is not zeroed.
// Returns 0 when out of memory. 0 is reserved for null.
fn alloc() -> u32 {
    var old_free = atomicLoad(&world.chunks_free);
    loop {
        let next_ptr = atomicLoad(&world.chunks[old_free][0]);
        let new_free = select(next_ptr, old_free + 1u, next_ptr == 0u); // never pushed slot, bump to next
        if new_free >= CHUNKS_CAPACITY { return 0u; }
        let res = atomicCompareExchangeWeak(&world.chunks_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    return old_free;
}

fn free(idx: u32) {
    var old_free = atomicLoad(&world.chunks_free);
    loop {
        atomicStore(&world.chunks[idx][0], old_free);
        let res = atomicCompareExchangeWeak(&world.chunks_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

fn block_id(chunk_idx: u32, pos: vec3u) -> u32 {
    let addr = _pos_to_addr(pos);
    let word = atomicLoad(&world.chunks[chunk_idx][addr.word_idx]);
    return extractBits(word, addr.bit_idx, BITS_PER_ID);
}

fn set_block_id(chunk_idx: u32, pos: vec3u, id: u32) {
    let addr = _pos_to_addr(pos);
    let clear_mask = ~(ID_MASK << addr.bit_idx);
    let id_bits = id << addr.bit_idx;
    atomicAnd(&world.chunks[chunk_idx][addr.word_idx], clear_mask);
    atomicOr(&world.chunks[chunk_idx][addr.word_idx], id_bits);
}

fn fill(chunk_idx: u32, id: u32) {
    var filled_word = 0u;
    for (var slot = 0u; slot < IDS_PER_GROUP; slot++) {
        filled_word = insertBits(filled_word, id, slot * BITS_PER_ID, BITS_PER_ID);
    }
    for (var i = 0u; i < CHUNK_LEN; i++) {
        atomicStore(&world.chunks[chunk_idx][i], filled_word);
    }
}

fn is_empty(chunk_idx: u32) -> bool {
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= atomicLoad(&world.chunks[chunk_idx][i]);
    }
    return acc == 0u;
}

fn uniform_val(chunk_idx: u32) -> u32 {
    let first_id = atomicLoad(&world.chunks[chunk_idx][0]) & ID_MASK;
    let expected_pattern = first_id * _BROADCAST_MULT;
    var acc = 0u;
    for (var i = 0u; i < CHUNK_LEN; i++) {
        acc |= (atomicLoad(&world.chunks[chunk_idx][i]) ^ expected_pattern);
    }
    if acc != 0u { return IS_NOT_UNIFORM; }
    return block_id(chunk_idx, vec3u(0, 0, 0));
}
