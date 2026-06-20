#include "./data/read-write/World.wgsl"

@group(0) @binding(0) var<storage, read_write> world: World;

fn hash(p: vec2u) -> f32 {
    let p_f = vec2f(p);
    let dot_product = dot(p_f, vec2f(127.1, 311.7));
    return fract(sin(dot_product) * 43758.5453123);
}

fn noise(p: vec2u) -> f32 {
    let v1 = hash(p);
    let v2 = hash(p + vec2u(1u, 0u));
    let v3 = hash(p + vec2u(0u, 1u));
    let v4 = hash(p - vec2u(1u, 0u));
    let v5 = hash(p - vec2u(0u, 1u));
    return (v1 + v2 + v3 + v4 + v5) / 5.0;
}

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) chunk_pos: vec3u) {
    if world_idx(chunk_pos) != WORLD_IDX_NONE { return; }
    let chunk_idx = chunk_free_pop();
    let origin_pos = chunk_pos * CHUNK_SIDE;
    for (var i = 0u; i < 8u; i++) {
        for (var j = 0u; j < 8u; j++) {
            let noise = noise(origin_pos.xz + vec2u(i, j));
            let scaled = u32(noise * 10.0);
            for (var k = 0u; k < 8u; k++) {
                let block_id = select(0u, 1u, k + origin_pos.y <= scaled);
                chunk_set(chunk_idx, vec3u(i, k, j), block_id);
            }
        }
    }
    world_insert(chunk_pos, chunk_idx);
}
