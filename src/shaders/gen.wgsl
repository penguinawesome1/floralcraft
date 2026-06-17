@group(0) @binding(0) var<storage, read_write> world: World;

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) chunk_pos: vec3u) {
    if world_idx(vec3i(chunk_pos)) != WORLD_IDX_NONE { return; }
    let chunk_idx = chunk_free_pop();
    for (var i = 0u; i < 8u; i++) {
        for (var j = 0u; j < 8u; j++) {
            for (var k = 0u; k < 8u; k++) {
                chunk_set(chunk_idx, vec3u(i, j, k), 1);
            }
        }
    }
    world_insert(vec3i(chunk_pos), chunk_idx);
}
