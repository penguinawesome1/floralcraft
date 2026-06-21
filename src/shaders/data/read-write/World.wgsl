#include "../../config.wgsl"
#include "./Chunk.wgsl"

const world__IDX_NONE = 0xFFFFFFFFu;
const world__UNIFORM_BIT = 1u << 23u;
const world__IS_NOT_UNIFORM = 0xFFFFFFFFu;
const _world__NO_BASE = 0xFFFFFFFEu;
const _world__FAILED_CAS = 0xFFFFFFFDu;
const _world__ADDED_UNIFORM = 0xFFFFFFFCu;
const _world__ZERO_IDX = 0xFFFFFFFFu;

struct world__World {
    // Marks the start index of the 8 contigious free slots.
    svo_nodes_free: atomic<u32>,
    // Each node is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_nodes: array<atomic<u32>, config__SVO_NODES_CAPACITY>,
    chunks_free: atomic<u32>,
    chunks: array<chunk__Chunk, config__CHUNKS_CAPACITY>,
}

fn _world__child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn _world__free_pop() -> u32 {
    var old_free = atomicLoad(&g_world.svo_nodes_free);
    loop {
        var new_free = atomicLoad(&g_world.svo_nodes[old_free]);
        new_free = select(new_free, old_free + 8u, new_free == 0u);
        new_free = select(new_free, 0u, new_free == _world__ZERO_IDX);
        let res = atomicCompareExchangeWeak(&g_world.svo_nodes_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    atomicStore(&g_world.svo_nodes[old_free], 0u);
    return old_free;
}

fn _world__free_push(idx: u32) {
    var old_free = atomicLoad(&g_world.svo_nodes_free);
    loop {
        let true_old_free = select(old_free, _world__ZERO_IDX, old_free == 0u);
        atomicStore(&g_world.svo_nodes[idx], true_old_free);
        let res = atomicCompareExchangeWeak(&g_world.svo_nodes_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

fn _world__child_bit(child_num: u32) -> u32 {
    return 1u << (child_num + 24u);
}

fn _world__child_idx(node: u32, child_num: u32) -> u32 {
    let is_child_present = (node & _world__child_bit(child_num)) != 0u;
    let child_idx = extractBits(node, 0u, 23u) + child_num;
    return select(world__IDX_NONE, child_idx, is_child_present);
}

fn _world__try_collapse(uniform_opt: u32, base_idx: u32, child_num: u32, parent_node_idx: u32) -> bool {
    if uniform_opt == world__IS_NOT_UNIFORM { return false; }
    let expected = uniform_opt | world__UNIFORM_BIT;
    var acc = 0u;
    for (var i = 0u; i < 8u; i++) {
        if i == child_num { continue; }
        acc |= atomicLoad(&g_world.svo_nodes[base_idx + i]) ^ expected;
    }
    if acc != 0u { return false; }
    for (var i = 0u; i < 8u; i++) {
        if i == child_num { continue; }
        _world__free_push(base_idx + i);
    }
    atomicStore(&g_world.svo_nodes[parent_node_idx], uniform_opt | world__UNIFORM_BIT);
    return true;
}

fn _world__add_child(node: u32, node_idx: u32, child_num: u32, uniform_opt: u32, parent_node_idx: u32) -> u32 {
    let child_bit = _world__child_bit(child_num);
    let base_idx = extractBits(node, 0u, 23u);
    if base_idx == 0u { return _world__NO_BASE; }
    if _world__try_collapse(uniform_opt, base_idx, child_num, parent_node_idx) { return _world__ADDED_UNIFORM; }
    let res = atomicCompareExchangeWeak(&g_world.svo_nodes[node_idx], node, node | child_bit);
    if res.exchanged { return base_idx + child_num; }
    return _world__FAILED_CAS;
}

fn _world__alloc_children(node: u32, node_idx: u32, child_num: u32) -> u32 {
    let child_bit = _world__child_bit(child_num);
    let base_idx = _world__free_pop();
    let new_val = insertBits(node, base_idx, 0u, 23u) | child_bit;
    let res = atomicCompareExchangeWeak(&g_world.svo_nodes[node_idx], node, new_val);
    if res.exchanged { return base_idx + child_num; }
    _world__free_push(base_idx);
    return _world__FAILED_CAS;
}

fn world__idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < config__SVO_DEPTH; i++) {
        let node = atomicLoad(&g_world.svo_nodes[node_idx]);
        if (node & world__UNIFORM_BIT) != 0 { return node; }
        let child_num = _world__child_num(pos, config__SVO_DEPTH - 1u - i);
        let child_idx = _world__child_idx(node, child_num);
        if child_idx == world__IDX_NONE { return world__IDX_NONE; }
        node_idx = child_idx;
    }
    return atomicLoad(&g_world.svo_nodes[node_idx]);
}

fn world__insert(pos: vec3u, chunk_idx: u32) {
    let is_uniform = chunk__is_uniform(chunk_idx);
    let uniform_opt = select(world__IS_NOT_UNIFORM, chunk__get(chunk_idx, vec3u(0, 0, 0)), is_uniform);
    var parent_node_idx = 0u;
    var node_idx = 0u;
    var i = 0u;
    while i < config__SVO_DEPTH {
        let node = atomicLoad(&g_world.svo_nodes[node_idx]);
        let child_num = _world__child_num(pos, config__SVO_DEPTH - 1u - i);
        var child_idx = _world__child_idx(node, child_num);
        if child_idx == world__IDX_NONE { child_idx = _world__add_child(node, node_idx, child_num, uniform_opt, parent_node_idx); }
        if child_idx == _world__ADDED_UNIFORM {
            chunk__free_push(chunk_idx);
            return;
        }
        if child_idx == _world__NO_BASE { child_idx = _world__alloc_children(node, node_idx, child_num); }
        if child_idx == _world__FAILED_CAS { continue; }
        parent_node_idx = node_idx;
        node_idx = child_idx;
        i++;
    }
    let val = select(chunk_idx, uniform_opt | world__UNIFORM_BIT, is_uniform);
    atomicStore(&g_world.svo_nodes[node_idx], val);
}

// fn world_remove(pos: vec3u) {}
