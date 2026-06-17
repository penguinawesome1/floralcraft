const SVO_DEPTH = 6u;
const SVO_BRANCHES_MAX_LEN = 37449u; // (8 ^ SVO_DEPTH - 1) / 7
const SVO_MIDDLE_OFFSET = vec3i(i32(1u << (SVO_DEPTH - 1u)));
const WORLD_IDX_NONE = 0xFFFFFFFFu;

struct World {
    // Marks the start index of the 8 contigious free slots.
    svo_branches_free: u32,
    // Each branch is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_branches: array<u32, SVO_BRANCHES_MAX_LEN>,
    chunks_free: u32,
    chunks: array<Chunk>,
}

fn _world_child_idx(pos: vec3u, depth: u32) -> u32 {
    let bits = (pos >> vec3u(depth)) & vec3u(1u);
    return bits.x | (bits.y << 1u) | (bits.z << 2u);
}

fn _world_wrapped_pos(pos: vec3i) -> vec3u {
    return vec3u(pos + vec3i(SVO_MIDDLE_OFFSET)); // TODO: make this change based on player pos
}

fn world_idx(chunk_pos: vec3i) -> u32 {
    let wrapped_pos = _world_wrapped_pos(chunk_pos);
    var branch_idx = 0u;
    for (var i = 0u; i < SVO_DEPTH; i++) {
        let branch = world.svo_branches[branch_idx];
        let child_idx = _world_child_idx(wrapped_pos, SVO_DEPTH - 1u - i);
        let child_bit = 1u << (child_idx + 23u);
        if (branch & child_bit) == 0u { return WORLD_IDX_NONE; }
        branch_idx = extractBits(branch, 0u, 23u) + child_idx;
    }
    let branch = world.svo_branches[branch_idx];
    return extractBits(branch, 0u, 23u);
}
