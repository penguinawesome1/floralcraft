import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read_write::chunk::{Chunk, free as chunk_free, uniform_val, IS_NOT_UNIFORM};

const UNIFORM_MASK = 1u << 23u;
const DEPTH_SHIFT = 18u;
const DEPTH_MASK = 0x1Fu << DEPTH_SHIFT;
const CHILD_SHIFT = 24u;
const CHILD_MASK = 0xFFu << CHILD_SHIFT;
const VALUE_MASK = 0x3FFFFu;
const ADDRESS_MASK = 0x7FFFFFu;
const _OOM = 0xFFFFFFFEu;
const _FAILED_CAS = 0xFFFFFFFDu;
const _ADDED_UNIFORM = 0xFFFFFFFCu;

struct ChildInfo {
    num: u32,
    mask: u32,
}

struct World {
    // Marks the start index of the 8 contiguous free slots and the head of other groups.
    // Initially 1, 0 is reserved for null.
    svo_nodes_free: atomic<u32>,
    // Head of chunk free list.
    // Initially 1, 0 is reserved for null.
    chunks_free: atomic<u32>,
    // Each node is like this: AAAA AAAA BCCC CCDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The depth into the tree if a uniform value, otherwise is D.
    // D: The address of children or the uniform value if present.
    svo_nodes: array<atomic<u32>, SVO_NODES_CAPACITY>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

@group(0) @binding(0) var<storage, read_write> world: World;

fn _child_info(pos: vec3u, depth: u32) -> ChildInfo {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    let num = bits.x | (bits.y << 1u) | (bits.z << 2u);
    let mask = 1u << (num + CHILD_SHIFT);
    return ChildInfo(num, mask);
}

// Finds the index of a specific child given a node.
// Returns 0 if child is not present.
fn _child_idx(node: u32, child_info: ChildInfo) -> u32 {
    let is_child_present = (node & child_info.mask) != 0u;
    let child_idx = (node & VALUE_MASK) + child_info.num;
    return select(0u, child_idx, is_child_present);
}

// Removes an svo group of 8 contiguous items from the free list and returns the starting index.
// The indices are zeroed.
// Returns 0 when out of memory. 0 is reserved for the root and means null.
fn _alloc() -> u32 {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        let next_ptr = atomicLoad(&world.svo_nodes[old_free]);
        let new_free = select(next_ptr, old_free + 8u, next_ptr == 0u); // never pushed slot, bump to next group
        if new_free + 7 >= SVO_NODES_CAPACITY { return 0u; }
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    for (var i = 0u; i < 8u; i++) {
        atomicStore(&world.svo_nodes[old_free + i], 0u);
    }
    return old_free;
}

fn _free(idx: u32) {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        atomicStore(&world.svo_nodes[idx], old_free);
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

// Combines children if they are similarly uniform.
// Unallocates chunks when it is a leaf level.
// fn _try_collapse(
//     uval: u32,
//     node_idx: u32,
//     base_idx: u32,
//     child_num: u32,
//     depth: u32
// ) -> bool {
//     if uval == IS_NOT_UNIFORM { return false; }
//     let expected = uval | UNIFORM_MASK;

//     var acc = 0u;
//     for (var i = 0u; i < 8u; i++) {
//         if i == child_num { continue; }
//         acc |= atomicLoad(&world.svo_nodes[base_idx + i]) ^ expected;
//     }
//     if acc != 0u { return false; }

//     if depth == SVO_DEPTH - 1u {
//         for (var i = 0u; i < 8u; i++) {
//             if i == child_num { continue; }
//             chunk_free(atomicLoad(&world.svo_nodes[base_idx + i]));
//         }
//     }
//     _free(base_idx);
//     atomicStore(&world.svo_nodes[node_idx], uval | (depth << 18u) | UNIFORM_MASK);
//     return true;
// }

fn _add_or_alloc_child(node: u32, node_idx: u32, child_info: ChildInfo) -> u32 {
    var base_idx = node & VALUE_MASK;
    if base_idx != 0u {
        let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, node | child_info.mask);
        if res.exchanged { return base_idx + child_info.num; }
        return _FAILED_CAS;
    }
    base_idx = _alloc();
    if base_idx == 0u { return _OOM; }
    let new_val = insertBits(node, base_idx, 0u, 23u) | child_info.mask;
    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_val);
    if res.exchanged { return base_idx + child_info.num; }
    _free(base_idx);
    return _FAILED_CAS;
}

fn _is_in_bounds(pos: vec3u) -> bool {
    return all(pos < vec3u(1u << SVO_DEPTH));
}

fn _chunk_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        if (node & UNIFORM_MASK) != 0u { return node; }
        let child_info = _child_info(pos, SVO_DEPTH - 1u - i);
        if (node & child_info.mask) == 0u { return 0u; }
        node_idx = (node & ADDRESS_MASK) + child_info.num;
    }
    return atomicLoad(&world.svo_nodes[node_idx]) & VALUE_MASK;
}

fn insert(pos: vec3u, chunk_idx: u32) {
    if !_is_in_bounds(pos) { return; }
    // let uval = uniform_val(chunk_idx);
    // if uval != IS_NOT_UNIFORM { chunk_free(chunk_idx); }
    var node_idx = 0u;
    var i = 0u;
    while i < SVO_DEPTH {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_info = _child_info(pos, SVO_DEPTH - 1u - i);

        var child_idx = _child_idx(node, child_info);
        if child_idx == 0u { child_idx = _add_or_alloc_child(node, node_idx, child_info); }
        if child_idx == _OOM { return; }
        if child_idx == _FAILED_CAS { continue; }

        node_idx = child_idx;
        i++;
    }
    // let val = select(chunk_idx, uval | UNIFORM_MASK | (SVO_DEPTH << 18u), uval != IS_NOT_UNIFORM);
    atomicStore(&world.svo_nodes[node_idx], chunk_idx);
}

fn remove(pos: vec3u) {
    if !_is_in_bounds(pos) { return; }
    chunk_free(_chunk_idx(pos));
}

fn contains(pos: vec3u) -> bool {
    if !_is_in_bounds(pos) { return false; }
    return _chunk_idx(pos) != 0u;
}