import package::config::SVO_DEPTH;
import package::data::{read::{chunk, world::world}, blocks, pos};

const UNIFORM_MASK = 1u << 23u;
const VALUE_MASK = 0x7FFFFFu;
const CHILD_SHIFT = 24u;

struct _ChildInfo {
    num: u32,
    mask: u32,
}

struct LookupResult {
    height: u32, // heights greater than 0 are uniform
    value: u32, // chunk index if height is 0, otherwise collapsed block id
}

fn _child_info(pos: vec3u, height: u32) -> _ChildInfo {
    let bits = (pos >> vec3u(height - 2u)) & vec3u(1u);
    let num = bits.x | (bits.y << 1u) | (bits.z << 2u);
    let mask = 1u << (num + CHILD_SHIFT);
    return _ChildInfo(num, mask);
}

fn lookup(pos: vec3u) -> LookupResult {
    var node_idx = _child_info(pos, SVO_DEPTH).num + 1u; // skip null idx
    for (var height = SVO_DEPTH - 1u; height > 1u; height--) {
        let node = world.svo_nodes[node_idx];
        if (node & UNIFORM_MASK) != 0u { return LookupResult(height, node & VALUE_MASK); }
        let child_info = _child_info(pos, height);
        if (node & child_info.mask) == 0u { return LookupResult(height, blocks::AIR); }
        node_idx = (node & VALUE_MASK) + child_info.num;
    }
    let node = world.svo_nodes[node_idx];
    if (node & UNIFORM_MASK) != 0u { return LookupResult(1u, node & VALUE_MASK); }
    return LookupResult(0u, node & VALUE_MASK);
}