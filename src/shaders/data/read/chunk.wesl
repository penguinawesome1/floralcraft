import package::config::{CHUNK_SIDE_SHIFT, BITS_PER_ID, CHUNKS_CAPACITY};
import package::raytrace::world;

const CHUNK_SIDE = 1u << CHUNK_SIDE_SHIFT;
const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = (CHUNK_VOLUME * BITS_PER_ID + 31u) / 32u;
const IDS_PER_GROUP = 32u / BITS_PER_ID;
const ID_MASK = (1u << BITS_PER_ID) - 1u;

alias Chunk = array<u32, CHUNK_LEN>;

// Finds the total bit offset of the pos within the chunk.
fn _pos_to_offset(pos: vec3u) -> u32 {
    let linear_idx = pos.z * (CHUNK_SIDE * CHUNK_SIDE) + pos.y * CHUNK_SIDE + pos.x;
    return linear_idx * BITS_PER_ID;
}

fn get_block_id(chunk_idx: u32, pos: vec3u) -> u32 {
    let offset = _pos_to_offset(pos);
    let word_idx = offset >> 5u;
    let bit_idx = offset & 31u;
    return extractBits(world.chunks[chunk_idx][word_idx], bit_idx, BITS_PER_ID);
}
