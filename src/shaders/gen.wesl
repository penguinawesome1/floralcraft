import package::data::read_write::chunk::{alloc, set_block_id, CHUNK_SIDE};
import package::data::read_write::world::{World, chunk_idx, insert, contains};
import package::data::blocks;

fn hash(p: vec2f) -> vec2f {
    let p2 = vec2f(dot(p, vec2f(127.1, 311.7)), dot(p, vec2f(269.5, 183.3)));
    return -1.0 + 2.0 * fract(sin(p2) * 43758.5453123);
}

fn simplex2d(p: vec2f) -> f32 {
    let K1 = 0.366025404;
    let K2 = 0.211324865;
    let i = floor(p + (p.x + p.y) * K1);
    let a = p - i + (i.x + i.y) * K2;
    let o = select(vec2f(1.0, 0.0), vec2f(0.0, 1.0), a.x < a.y);
    let b = a - o + K2;
    let c = a - 1.0 + 2.0 * K2;
    let h = max(0.5 - vec3f(dot(a, a), dot(b, b), dot(c, c)), vec3f(0.0));
    let n = h * h * h * h * vec3f(dot(a, hash(i + 0.0)), dot(b, hash(i + o)), dot(c, hash(i + 1.0)));
    return dot(n, vec3f(70.0));
}

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) chunk_pos: vec3u) {
    if contains(chunk_pos) { return; }
    let chunk_idx = alloc();
    if chunk_idx == 0u { return; }
    let origin_pos = chunk_pos * CHUNK_SIDE;
    for (var i = 0u; i < CHUNK_SIDE; i++) {
        for (var j = 0u; j < CHUNK_SIDE; j++) {
            let pos = vec2f(origin_pos.xz + vec2u(i, j)) * 0.04;
            let noise_normalized = simplex2d(pos) * 0.5 + 0.5;
            let scaled = u32(noise_normalized * 50.0);
            for (var k = 0u; k < CHUNK_SIDE; k++) {
                let current_y = k + origin_pos.y;
                var block_id = blocks::AIR;
                if current_y == 0u {
                    block_id = blocks::BEDROCK;
                } else if current_y + 4u < scaled {
                    block_id = blocks::STONE;
                } else if current_y < scaled {
                    block_id = blocks::DIRT;
                } else if current_y == scaled {
                    block_id = blocks::GRASS;
                } else {
                    block_id = blocks::AIR;
                }
                set_block_id(chunk_idx, vec3u(i, k, j), block_id);
            }
        }
    }
    insert(chunk_pos, chunk_idx);
}
// // Shared memory accessible by all 512 threads in the workgroup
// var<workgroup> shared_chunk_idx: u32;

// // Match the workgroup size to your chunk dimensions (8x8x8 = 512 threads)
// @compute @workgroup_size(8, 8, 8)
// fn cs_main(
//     @builtin(workgroup_id) chunk_pos: vec3u,
//     @builtin(local_invocation_id) local_pos: vec3u,
//     @builtin(local_invocation_index) local_idx: u32 
// ) {
//     // 1. Isolate the Atomics: Only Thread 0 checks existence and allocates
//     if (local_idx == 0u) {
//         if (chunk_idx(chunk_pos) == 0u) {
//             shared_chunk_idx = alloc();
//         } else {
//             shared_chunk_idx = 0u; 
//         }
//     }

//     // Wait for Thread 0 to finish allocating
//     workgroupBarrier();

//     // If chunk exists or allocation failed, all 512 threads exit early
//     if (shared_chunk_idx == 0u) { return; } 

//     // 2. Parallel Generation: NO NESTED LOOPS
//     // Every thread calculates exactly ONE block based on its local_pos
//     let origin_pos = chunk_pos * 8u; 
//     let global_pos = origin_pos + local_pos;
    
//     let pos2d = vec2f(f32(global_pos.x), f32(global_pos.z)) * 0.04;
//     let noise_normalized = simplex2d(pos2d) * 0.5 + 0.5;
//     let scaled = u32(noise_normalized * 50.0);

//     var block_id = blocks::AIR;
//     if (global_pos.y == 0u) {
//         block_id = blocks::BEDROCK;
//     } else if (global_pos.y + 4u < scaled) {
//         block_id = blocks::STONE;
//     } else if (global_pos.y < scaled) {
//         block_id = blocks::DIRT;
//     } else if (global_pos.y == scaled) {
//         block_id = blocks::GRASS;
//     }

//     // All 512 threads write their block simultaneously 
//     set_block_id(shared_chunk_idx, local_pos, block_id);

//     // Wait for all 512 threads to finish writing the chunk data
//     workgroupBarrier();

//     // 3. Isolate the Atomics: Only Thread 0 inserts the finished chunk into the SVO
//     if (local_idx == 0u) {
//         insert(chunk_pos, shared_chunk_idx);
//     }
// }
