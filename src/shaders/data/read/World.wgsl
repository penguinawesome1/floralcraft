const WORLD_IDX_NONE = 0xFFFFFFFFu;

struct World {
    // Marks the start index of the 8 contigious free slots.
    svo_nodes_free: u32,
    // Each node is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_nodes: array<u32, SVO_NODES_CAPACITY>,
    chunks_free: u32,
    chunks: array<Chunk>,
}

fn _world_child_num(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn world_idx(pos: vec3u) -> u32 {
    var node_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let node = world.svo_nodes[node_idx];
        let child_num = _world_child_num(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_num + 23u);
        if (node & child_bit) == 0u { return WORLD_IDX_NONE; }
        node_idx = extractBits(node, 0u, 23u) + child_num;
    }
    return world.svo_nodes[node_idx];
}
