import constants::GEN_SIDE;
import package::core::{pos, flags, flags::ALL_AIR};
import package::gen::core::{UnloadParams, FreeList, in_wrapped_range};

@group(0) @binding(0) var<storage, read> unload_params: UnloadParams;
@group(0) @binding(1) var chunk_index_map: texture_3d<u32>; 
@group(0) @binding(2) var<storage, read_write> free_list: FreeList;

fn _free(chunk_num: u32) {
    let slot = atomicSub(&free_list.len, 1u) - 1u;
    free_list.indices[slot] = chunk_num;
}

@compute @workgroup_size(16, 16, 1)
fn free_slab(@builtin(global_invocation_id) id: vec3u) {
    if id.x >= GEN_SIDE || id.y >= GEN_SIDE { return; }
    let wrapped_origin = pos::wrapped_chunk(unload_params.origin);

    let dx = unload_params.depths.x;
    let dy = unload_params.depths.y;
    let dz = unload_params.depths.z;

    for (var d = 0u; d < dx; d++) {
        let wrapped_x = (wrapped_origin.x + d) % GEN_SIDE;
        let chunk_num = textureLoad(chunk_index_map, vec3u(wrapped_x, id.x, id.y), 0u).x;
        if chunk_num != 0u && chunk_num != ALL_AIR { _free(chunk_num); }
    }

    if !in_wrapped_range(id.x, wrapped_origin.x, dx) {
        for (var d = 0u; d < dy; d++) {
            let wrapped_y = (wrapped_origin.y + d) % GEN_SIDE;
            let chunk_num = textureLoad(chunk_index_map, vec3u(id.x, wrapped_y, id.y), 0u).x;
            if chunk_num != 0u && chunk_num != ALL_AIR { _free(chunk_num); }
        }
    }

    if !in_wrapped_range(id.x, wrapped_origin.x, dx) &&
       !in_wrapped_range(id.y, wrapped_origin.y, dy) {
        for (var d = 0u; d < dz; d++) {
            let wrapped_z = (wrapped_origin.z + d) % GEN_SIDE;
            let chunk_num = textureLoad(chunk_index_map, vec3u(id.x, id.y, wrapped_z), 0u).x;
            if chunk_num != 0u && chunk_num != ALL_AIR { _free(chunk_num); }
        }
    }
}