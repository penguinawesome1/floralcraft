import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY, CHUNK_SIDE_SHIFT};
import package::data::read::chunk::{Chunk, CHUNK_SIDE};
import package::data::read::chunk;
import package::data::blocks;
import package::raytrace::conversions::block_to_chunk;

const UNIFORM_MASK = 1u << 23u;
const CHILD_SHIFT = 24u;
const CHILD_MASK = 0xFFu << CHILD_SHIFT;
const VALUE_MASK = 0x7FFFFFu;

struct _ChildInfo {
    num: u32,
    mask: u32,
}

struct LookupResult {
    height: u32,
    value: u32, // chunk index if at height 0, otherwise collapsed block ID
}

struct World {
    // Marks the start index of the 8 contiguous free slots and the head of other groups.
    // Initially 1, 0 is reserved for null.
    svo_nodes_free: u32,
    // Head of chunk free list.
    // Initially 1, 0 is reserved for null.
    chunks_free: u32,
    // Each node is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    // There is no root, begin at first child.
    svo_nodes: array<u32, SVO_NODES_CAPACITY>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

@group(0) @binding(0) var<storage, read> world: World;

fn _child_info(pos: vec3u, height: u32) -> _ChildInfo {
    let bits = (pos >> vec3u(height - 2u)) & vec3u(1u);
    let num = bits.x | (bits.y << 1u) | (bits.z << 2u);
    let mask = 1u << (num + CHILD_SHIFT);
    return _ChildInfo(num, mask);
}

fn _block_to_chunk(pos: vec3i) -> vec3i {
    return pos >> vec3u(CHUNK_SIDE_SHIFT);
}

fn _umod3(a: vec3i, b: vec3i) -> vec3i {
    return ((a % b) + b) % b;
}

fn block(pos: vec3i) -> LookupResult {
    let chunk_pos = vec3u(_block_to_chunk(pos));
    if any(chunk_pos >= vec3u(1u << SVO_DEPTH)) { return LookupResult(1u, blocks::AIR); }
    var node_idx = _child_info(chunk_pos, SVO_DEPTH - 1u).num;
    for (var height = SVO_DEPTH - 1u; height >= 2u; height--) {
        let node = world.svo_nodes[node_idx];
        if (node & UNIFORM_MASK) != 0u { return LookupResult(height, node & VALUE_MASK); }
        let child_info = _child_info(chunk_pos, height);
        if (node & child_info.mask) == 0u { return LookupResult(height, blocks::AIR); }
        node_idx = (node & VALUE_MASK) + child_info.num;
    }
    let leaf = world.svo_nodes[node_idx];
    if (leaf & UNIFORM_MASK) != 0u { return LookupResult(1u, leaf & VALUE_MASK); }
    let local_pos = vec3u(_umod3(pos, vec3i(i32(CHUNK_SIDE))));
    return LookupResult(0u, chunk::block_id(leaf, local_pos));
}