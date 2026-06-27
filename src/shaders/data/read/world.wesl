import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read::chunk::Chunk;
import package::data::blocks;

const UNIFORM_MASK = 1u << 23u;
const CHILD_SHIFT = 24u;
const CHILD_MASK = 0xFFu << CHILD_SHIFT;
const ADDRESS_MASK = 0x7FFFFFu;
const VALUE_MASK = 0x3FFFFu;

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
    let bits = (pos >> vec3u(height - 1u)) & vec3u(1u);
    let num = bits.x | (bits.y << 1u) | (bits.z << 2u);
    let mask = 1u << (num + CHILD_SHIFT);
    return _ChildInfo(num, mask);
}

fn lookup(pos: vec3u) -> LookupResult {
    if any(pos >= vec3u(1u << SVO_DEPTH)) {
        return LookupResult(1u, blocks::AIR); // height 1 ensures the minimum of one chunk is skipped in dda
    }
    var node_idx = _child_info(pos, SVO_DEPTH - 1u).num;
    for (var height = SVO_DEPTH - 1u; height > 0u; height--) {
        let node = world.svo_nodes[node_idx];
        if (node & UNIFORM_MASK) != 0u { return LookupResult(height, node & VALUE_MASK); }
        let child_info = _child_info(pos, height);
        if (node & child_info.mask) == 0u { return LookupResult(height, blocks::AIR); }
        node_idx = (node & ADDRESS_MASK) + child_info.num;
    }
    return LookupResult(0u, world.svo_nodes[node_idx] & VALUE_MASK);
}
