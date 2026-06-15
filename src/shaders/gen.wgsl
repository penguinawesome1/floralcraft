@group(0) @binding(0) var<storage, read_write> world: World;

@compute @workgroup_size(1, 1, 1)
fn cs_main(@builtin(global_invocation_id) chunk_pos: vec3u) {
    // if chunk pos already in world: return;
    // for block in chunk: block = noise_val(pos);

    if chunk_get(0, vec3u(0, 0, 0)) == 0 {
        chunk_set(0, vec3u(0, 0, 0), 1);
    }
}
