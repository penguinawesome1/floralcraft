import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read::chunk::Chunk;

const UNIFORM_MASK = 1u << 23u;
const DEPTH_SHIFT = 18u;
const DEPTH_MASK = 0x1Fu << DEPTH_SHIFT;
const CHILD_SHIFT = 24u;
const CHILD_MASK = 0xFFu << CHILD_SHIFT;
const ADDRESS_MASK = 0x7FFFFFu;
const VALUE_MASK = 0x3FFFFu;

struct World {
    // Marks the start index of the 8 contiguous free slots and the head of other groups.
    // Initially 1, 0 is reserved for null.
    svo_nodes_free: u32,
    // Head of chunk free list.
    // Initially 1, 0 is reserved for null.
    chunks_free: u32,
    // Each node is like this: AAAA AAAA BCCC CCDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The depth into the tree if a uniform value, otherwise is D.
    // D: The address of children or the uniform value if present.
    svo_nodes: array<u32, SVO_NODES_CAPACITY>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

@group(0) @binding(0) var<storage, read> world: World;

fn _child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

// Finds the index of the chunk given a position.
// Returns 0 when chunk is not present.
fn chunk_idx(pos: vec3u) -> u32 {
    if any(pos >= vec3u(1u << SVO_DEPTH)) { return 0u; }
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = world.svo_nodes[node_idx];
        if (node & UNIFORM_MASK) != 0u { return node; }
        let child_num = _child_num(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_num + 24u);
        if (node & child_bit) == 0u { return 0u; }
        node_idx = (node & ADDRESS_MASK) + child_num;
    }
    return world.svo_nodes[node_idx] & VALUE_MASK;
}
