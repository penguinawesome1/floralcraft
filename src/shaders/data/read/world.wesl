import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read::chunk::Chunk;
import package::raytrace::world;

const IDX_NONE = 0xFFFFFFFFu;
const UNIFORM_MASK = 1u << 23u;
const DEPTH_MASK = 0x1Fu << 18u;
const CHILD_MASK = 0xFFu << 24u;
const VALUE_MASK = 0x3FFFFu;

struct World {
    // Marks the start index of the 8 contiguous free slots.
    svo_nodes_free: u32,
    // Each node is like this: AAAA AAAA BCCC CCDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The depth into the tree if a uniform value, otherwise is D.
    // D: The address of children or the uniform value if present.
    svo_nodes: array<u32, SVO_NODES_CAPACITY>,
    chunks_free: u32,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

fn _child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn get_chunk_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = world.svo_nodes[node_idx];
        if (node & UNIFORM_MASK) != 0 { return node; }
        let child_num = _child_num(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_num + 24u);
        if (node & child_bit) == 0u { return IDX_NONE; }
        node_idx = (node & VALUE_MASK) + child_num;
    }
    return world.svo_nodes[node_idx];
}
