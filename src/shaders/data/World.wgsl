const SVO_DEPTH = 6u;
const SVO_BRANCHES_MAX_LEN = 37449u; // (8 ^ SVO_DEPTH - 1) / 7
const SVO_MIDDLE_OFFSET = vec3i(i32(1u << (SVO_DEPTH - 1u)));

struct World {
    // Marks the start index of the 8 contigious free slots.
    svo_branches_free: atomic<u32>,
    // Each branch is like this: AAAA AAAA BCCC CCCC CCCC CCCC CCCC CCCC.
    // A: The child bit mask.
    // B: Whether the children are all uniform, relevant only if every child is present.
    // C: The address of children or the uniform value if present.
    svo_branches: array<u32, SVO_BRANCHES_MAX_LEN>,
    chunks_free: atomic<u32>,
    chunks: array<Chunk>,
}

// fn _world_child_idx(pos: vec3u, depth: u32) -> u32 {
//     let bits = (pos >> vec3u(depth)) & vec3u(1u);
//     return bits.x | (bits.y << 1u) | (bits.z << 2u);
// }

fn world_insert(pos: vec3i) {}

fn world_remove(pos: vec3u) {}
