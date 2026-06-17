const WORLD_IDX_NONE = 0xFFFFFFFFu;

struct World {
    // Marks the start index of the 8 contigious free slots.
    svo_branches_free: u32,
    // Each branch is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_branches: array<u32, SVO_BRANCHES_CAPACITY>,
    chunks_free: u32,
    chunks: array<Chunk>,
}

fn _world_child_idx(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn world_idx(pos: vec3u) -> u32 {
    var branch_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let branch = world.svo_branches[branch_idx];
        let child_idx = _world_child_idx(pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_idx + 23u);
        if (branch & child_bit) == 0u { return WORLD_IDX_NONE; }
        branch_idx = extractBits(branch, 0u, 23u) + child_idx;
    }
    return world.svo_branches[branch_idx];
}
