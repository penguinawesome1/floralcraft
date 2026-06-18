const WORLD_IDX_NONE = 0xFFFFFFFFu;
const ERR_NO_BASE = 0xFFFFFFFEu;
const ERR_FAILED_CAS = 0xFFFFFFFDu;
const ZERO_IDX = 0xFFFFFFFFu;
const UNIFORM_BIT = 1u << 23u;

struct World {
    // Marks the start index of the 8 contigious free slots.
    svo_nodes_free: atomic<u32>,
    // Each node is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_nodes: array<atomic<u32>, SVO_NODES_CAPACITY>,
    chunks_free: atomic<u32>,
    chunks: array<Chunk>,
}

fn _world_child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn _world_free_pop() -> u32 {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        var new_free = atomicLoad(&world.svo_nodes[old_free]);
        new_free = select(new_free, old_free + 8u, new_free == 0u);
        new_free = select(new_free, 0u, new_free == ZERO_IDX);
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, new_free);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
    atomicStore(&world.svo_nodes[old_free], 0u);
    return old_free;
}

fn _world_free_push(idx: u32) {
    var old_free = atomicLoad(&world.svo_nodes_free);
    loop {
        let true_old_free = select(old_free, ZERO_IDX, old_free == 0u);
        atomicStore(&world.svo_nodes[idx], true_old_free);
        let res = atomicCompareExchangeWeak(&world.svo_nodes_free, old_free, idx);
        if res.exchanged { break; }
        old_free = res.old_value;
    }
}

fn _world_child_bit(child_num: u32) -> u32 {
    return 1u << (child_num + 23u);
}

fn _world_child_idx(node: u32, child_num: u32) -> u32 {
    let is_child_present = (node & _world_child_bit(child_num)) != 0u;
    let child_idx = extractBits(node, 0u, 23u) + child_num;
    return select(WORLD_IDX_NONE, child_idx, is_child_present);
}

fn _world_add_child(node: u32, node_idx: u32, child_num: u32) -> u32 {
    let child_bit = _world_child_bit(child_num);
    let base_idx = extractBits(node, 0u, 23u);
    if base_idx == 0u { return ERR_NO_BASE; }
    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, node | child_bit);
    if res.exchanged { return base_idx + child_num; }
    return ERR_FAILED_CAS;
}

fn _world_alloc_children(node: u32, node_idx: u32, child_num: u32) -> u32 {
    let child_bit = _world_child_bit(child_num);
    let base_idx = _world_free_pop();
    let new_val = insertBits(node, base_idx, 0u, 23u) | child_bit;
    let res = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_val);
    if res.exchanged { return base_idx + child_num; }
    _world_free_push(base_idx);
    return ERR_FAILED_CAS;
}

fn world_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_num = _world_child_num(pos, SVO_DEPTH - 1u - i);
        let child_idx = _world_child_idx(node, child_num);
        if child_idx == WORLD_IDX_NONE { return WORLD_IDX_NONE; }
        node_idx = child_idx;
    }
    return atomicLoad(&world.svo_nodes[node_idx]);
}

fn world_insert(pos: vec3u, chunk_idx: u32) {
    var node_idx = 0u;
    var i = 0u;
    while i < SVO_DEPTH {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_num = _world_child_num(pos, SVO_DEPTH - 1u - i);
        var child_idx = _world_child_idx(node, child_num);
        if child_idx == WORLD_IDX_NONE { child_idx = _world_add_child(node, node_idx, child_num); }
        if child_idx == ERR_NO_BASE { child_idx = _world_alloc_children(node, node_idx, child_num); }
        if child_idx == ERR_FAILED_CAS { continue; }
        node_idx = child_idx;
        i++;
    }
    atomicStore(&world.svo_nodes[node_idx], chunk_idx);
}

// fn world_remove(pos: vec3u) {}
