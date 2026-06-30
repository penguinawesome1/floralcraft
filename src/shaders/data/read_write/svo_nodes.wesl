import package::config::{SVO_DEPTH, SVO_NODES_CAPACITY};
import package::data::read_write::{chunk, world::world};

const UNIFORM_MASK = 1u << 23u;
const VALUE_MASK = 0x7FFFFFu;
const CHILD_SHIFT = 24u;
const _OOM = 0xFFFFFFFEu;
const _FAILED_CAS = 0xFFFFFFFDu;
const _ADDED_UNIFORM = 0xFFFFFFFCu;

struct _ChildInfo {
    num: u32,
    mask: u32,
}

struct _PathInfo {
    node_idx: u32,
    node: u32,
    child_info: _ChildInfo,
}

struct _AddChildResult {
    child_idx: u32,
    new_node: u32,
}

fn _child_info(pos: vec3u, height: u32) -> _ChildInfo {
    let bits = (pos >> vec3u(height - 2u)) & vec3u(1u);
    let num = bits.x | (bits.y << 1u) | (bits.z << 2u);
    let mask = 1u << (num + CHILD_SHIFT);
    return _ChildInfo(num, mask);
}

// Finds the index of a specific child given a node.
// Returns 0 if child is not present.
fn _child_idx(node: u32, child_info: _ChildInfo) -> u32 {
    let is_child_present = (node & child_info.mask) != 0u;
    let child_idx = (node & VALUE_MASK) + child_info.num;
    return select(0u, child_idx, is_child_present);
}

// Removes an svo group of 8 contiguous items from the free list and returns the starting index.
// The indices are zeroed.
// Returns 0 when out of memory. 0 is reserved null.
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

// Requires uval to be the current uniform value at the passed height for the current child.
fn _try_collapse(path_info: _PathInfo, uval: u32) -> bool {
    if uval == chunk::IS_NOT_UNIFORM { return false; }
    let base_idx = path_info.node & VALUE_MASK;
    let goal = uval | UNIFORM_MASK;
    var acc = true;
    for (var i = 0u; i < 8u; i++) {
        if i == path_info.child_info.num { continue; }
        let child = atomicLoad(&world.svo_nodes[base_idx + i]);
        acc &= child == goal;
    }
    if !acc { return false; }

    let res = atomicCompareExchangeWeak(&world.svo_nodes[path_info.node_idx], path_info.node, goal);
    if !res.exchanged { return res.old_value == goal; }
    _free(base_idx);
    return true;
}

fn _add_or_alloc_child(node_idx: u32, node: u32, child_info: _ChildInfo) -> _AddChildResult {
    var base_idx = node & VALUE_MASK;
    if base_idx != 0u {
        let new_node = node | child_info.mask;
        let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_node);
        if res.exchanged { return _AddChildResult(base_idx + child_info.num, new_node); }
        return _AddChildResult(_FAILED_CAS, 0u);
    }
    base_idx = _alloc();
    if base_idx == 0u { return _AddChildResult(_OOM, 0u); }
    let new_node = insertBits(node, base_idx, 0u, 23u) | child_info.mask;
    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_node);
    if res.exchanged { return _AddChildResult(base_idx + child_info.num, new_node); }
    _free(base_idx);
    return _AddChildResult(_FAILED_CAS, 0u);
}

fn insert(pos: vec3u, chunk_idx: u32) {
    let uval = chunk::uniform_val(chunk_idx);
    if uval != chunk::IS_NOT_UNIFORM { chunk::free(chunk_idx); }

    var path: array<_PathInfo, SVO_DEPTH>;
    var node_idx = _child_info(pos, SVO_DEPTH).num + 1u; // skip null idx
    var height = SVO_DEPTH - 1u;

    while height > 1u {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_info = _child_info(pos, height);
        var child_idx = _child_idx(node, child_info);
        var new_node = node;
        if child_idx == 0u {
            let res = _add_or_alloc_child(node_idx, node, child_info);
            child_idx = res.child_idx;
            if child_idx == _OOM { return; }
            if child_idx == _FAILED_CAS { continue; }
            new_node = res.new_node;
        }
        path[height] = _PathInfo(node_idx, new_node, child_info);
        node_idx = child_idx;
        height--;
    }

    if uval == chunk::IS_NOT_UNIFORM {
        atomicStore(&world.svo_nodes[node_idx], chunk_idx);
        return;
    }

    atomicStore(&world.svo_nodes[node_idx], uval | UNIFORM_MASK);
    for (var height = 2u; height < SVO_DEPTH; height++) {
        if !_try_collapse(path[height], uval) { return; }
    }
}
