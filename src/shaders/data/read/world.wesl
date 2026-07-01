import package::config::{SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::{read::{chunk, chunk::Chunk, svo_nodes}, pos, blocks};

struct BlockResult {
    height: u32,
    id: u32,
}

struct World {
    svo_nodes_free: u32,
    chunks_free: u32,
    // Each node is like this: AAAA AAAA BCDD DDDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: A mask to mark if the node is in the process of collapsing on a thread.
    // D: The address of children or the uniform value if present.
    // There is no root, begin at first child.
    svo_nodes: array<u32, SVO_NODES_CAPACITY>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

@group(0) @binding(0) var<storage, read> world: World;

fn block(global_pos: vec3i) -> BlockResult {
    if !pos::is_in_bounds(global_pos) { return BlockResult(0u, blocks::AIR); }
    let chunk_pos = pos::chunk(global_pos);
    let wrapped_pos = pos::wrapped_chunk(chunk_pos);

    let res = svo_nodes::lookup(wrapped_pos);
    if res.height != 0u { return BlockResult(res.height, res.value); }

    let local_pos = pos::local(global_pos);
    let block_id = chunk::block_id(res.value, local_pos);
    return BlockResult(0u, block_id);
}
