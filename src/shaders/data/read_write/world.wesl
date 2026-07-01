import package::config::{SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read_write::chunk::Chunk;

struct World {
    // Marks the start index of the 8 contiguous free slots and the head of other groups.
    // Initially 1, 0 is reserved for null.
    svo_nodes_free: atomic<u32>,
    // Head of chunk free list.
    // Initially 1, 0 is reserved for null.
    chunks_free: atomic<u32>,
    // Each node is like this: AAAA AAAA BCDD DDDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: A mask to mark if the node is in the process of collapsing on a thread.
    // D: The address of children or the uniform value if present.
    // There is no root, begin at first child.
    svo_nodes: array<atomic<u32>, SVO_NODES_CAPACITY>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

@group(0) @binding(0) var<storage, read_write> world: World;
