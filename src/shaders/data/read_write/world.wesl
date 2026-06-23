import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY, CHUNKS_CAPACITY};
import package::data::read_write::chunk::{Chunk, free_push as chunk_free_push, uniform_val, IS_NOT_UNIFORM};
import package::gen::world;

const IDX_NONE = 0xFFFFFFFFu;
const UNIFORM_MASK = 1u << 23u;
const DEPTH_MASK = 0x1Fu << 18u;
const CHILD_MASK = 0xFFu << 24u;
const VALUE_MASK = 0x3FFFFu;
const _NO_BASE = 0xFFFFFFFEu;
const _FAILED_CAS = 0xFFFFFFFDu;
const _ADDED_UNIFORM = 0xFFFFFFFCu;
const _IDX_ZERO = 0xFFFFFFFFu;

struct World {
    // Marks the start index of the 8 contiguous free slots.
    svo_nodes_free: atomic<u32>,
    // Each node is like this: AAAA AAAA BCCC CCDD DDDD DDDD DDDD DDDD.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The depth into the tree if a uniform value, otherwise is D.
    // D: The address of children or the uniform value if present.
    svo_nodes: array<atomic<u32>, SVO_NODES_CAPACITY>,
    chunks_free: atomic<u32>,
    chunks: array<Chunk, CHUNKS_CAPACITY>,
}

fn _child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn _free_pop() -> u32 {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        var new_free = atomicLoad(&world.svo_nodes[old_free]);
        new_free = select(new_free, old_free + 8u, new_free == 0u);
        new_free = select(new_free, 0u, new_free == _IDX_ZERO);
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    atomicStore(&world.svo_nodes[old_free], 0u);
    return old_free;
}

fn _free_push(idx: u32) {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        let true_old_free = select(old_free, _IDX_ZERO, old_free == 0u);
        atomicStore(&world.svo_nodes[idx], true_old_free);
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

fn _child_bit(child_num: u32) -> u32 {
    return 1u << (child_num + 24u);
}

fn _child_idx(node: u32, child_num: u32) -> u32 {
    let is_child_present = (node & _child_bit(child_num)) != 0u;
    let child_idx = (node & VALUE_MASK) + child_num;
    return select(IDX_NONE, child_idx, is_child_present);
}

// Combines children if they are similarly uniform.
// Unallocates chunks when it is a leaf level.
fn _try_collapse(
    uval: u32,
    node_idx: u32,
    base_idx: u32,
    child_num: u32,
    depth: u32
) -> bool {
    if uval == IS_NOT_UNIFORM { return false; }
    let expected = uval | UNIFORM_MASK;

    var acc = 0u;
    for (var i = 0u; i < 8u; i++) {
        if i == child_num { continue; }
        acc |= atomicLoad(&world.svo_nodes[base_idx + i]) ^ expected;
    }
    if acc != 0u { return false; }

    if depth == SVO_DEPTH - 1u {
        let chunk_idx = atomicLoad(&world.svo_nodes[base_idx]);
        chunk_free_push(chunk_idx);
    }
    _free_push(base_idx);
    atomicStore(&world.svo_nodes[node_idx], 0xFFFFFFF0);
    return true;
}

// Attempts to add a child to the SVO, adding a uniform value instead if possible.
fn _add_child(
    node: u32,
    node_idx: u32,
    child_num: u32,
    uval: u32,
    depth: u32
) -> u32 {
    let child_bit = _child_bit(child_num);
    let base_idx = node & VALUE_MASK;
    if base_idx == 0u { return _NO_BASE; }

    if _try_collapse(uval, node_idx, base_idx, child_num, depth) {
        return _ADDED_UNIFORM;
    }

    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, node | child_bit);
    if res.exchanged { return base_idx + child_num; }
    return _FAILED_CAS;
}

fn _alloc_children(node: u32, node_idx: u32, child_num: u32) -> u32 {
    let child_bit = _child_bit(child_num);
    let base_idx = _free_pop();

    for (var i = 0u; i < 8u; i++) {
        atomicStore(&world.svo_nodes[base_idx + i], 0u);
    }

    let new_val = insertBits(node, base_idx, 0u, 23u) | child_bit;
    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_val);
    if res.exchanged { return base_idx + child_num; }
    _free_push(base_idx);
    return _FAILED_CAS;
}

fn get_chunk_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        if (node & UNIFORM_MASK) != 0 { return node; }
        let child_num = _child_num(pos, SVO_DEPTH - 1u - i);
        let child_idx = _child_idx(node, child_num);
        if child_idx == IDX_NONE { return IDX_NONE; }
        node_idx = child_idx;
    }
    return atomicLoad(&world.svo_nodes[node_idx]);
}

fn insert(pos: vec3u, chunk_idx: u32) {
    let uval = uniform_val(chunk_idx);
    var node_idx = 0u;
    var i = 0u;
    while i < SVO_DEPTH {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_num = _child_num(pos, SVO_DEPTH - 1u - i);
        var child_idx = _child_idx(node, child_num);
        if child_idx == IDX_NONE {
            child_idx = _add_child(node, node_idx, child_num, uval, i);
        }
        if child_idx == _ADDED_UNIFORM {
            chunk_free_push(chunk_idx);
            return;
        }
        if child_idx == _NO_BASE {
            child_idx = _alloc_children(node, node_idx, child_num);
        }
        if child_idx == _FAILED_CAS { continue; }
        node_idx = child_idx;
        i++;
    }
    let val = select(chunk_idx, uval | UNIFORM_MASK, uval != IS_NOT_UNIFORM);
    atomicStore(&world.svo_nodes[node_idx], val);
}

// fn remove(pos: vec3u) {}
