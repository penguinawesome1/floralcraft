const WORLD_IDX_NONE = 0xFFFFFFFFu;
const ZERO_IDX = 0xFFFFFFFFu;

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

fn _world_child_idx(pos: vec3u, depth: u32) -> u32 {
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

fn world_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_idx = _world_child_idx(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_idx + 23u);
        if (node & child_bit) == 0u { return WORLD_IDX_NONE; }
        node_idx = extractBits(node, 0u, 23u) + child_idx;
    }
    return atomicLoad(&world.svo_nodes[node_idx]);
}

fn world_insert(pos: vec3u, chunk_idx: u32) {
    var node_idx = 0u;
    var i = 0u;
    while i < SVO_DEPTH {
        let node = atomicLoad(&world.svo_nodes[node_idx]);
        let child_idx = _world_child_idx(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_idx + 23u);
        if (node & child_bit) != 0u {
            node_idx = extractBits(node, 0u, 23u) + child_idx;
            i++;
            continue;
        }
        let existing_base = extractBits(node, 0u, 23u);
        if existing_base != 0u {
            let new_val = node | child_bit;
            let result = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_val);
            if result.exchanged {
                node_idx = existing_base + child_idx;
                i++;
            }
            continue;
        }
        let new_node_idx = _world_free_pop();
        let new_val = insertBits(node, new_node_idx, 0u, 23u) | child_bit;
        let result = atomicCompareExchangeWeak(&world.svo_nodes[node_idx], node, new_val);
        if !result.exchanged {
            _world_free_push(new_node_idx);
            continue;
        }
        node_idx = new_node_idx + child_idx;
        i++;
    }
    atomicStore(&world.svo_nodes[node_idx], chunk_idx);
}

// fn world_remove(pos: vec3u) {}
