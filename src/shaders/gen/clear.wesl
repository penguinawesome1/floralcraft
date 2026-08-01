import constants::GEN_SIDE;
import package::core::{pos, flags};
import package::gen::core::{UnloadParams, in_wrapped_range};

@group(0) @binding(0) var<storage, read> unload_params: UnloadParams;
@group(0) @binding(1) var chunk_index_map: texture_storage_3d<r32uint, write>; 
@group(0) @binding(2) var<storage, read_write> skip_mip: array<atomic<u32>>;

@compute @workgroup_size(16, 16, 1)
fn clear_slab(@builtin(global_invocation_id) id: vec3u) {
    if id.x >= GEN_SIDE || id.y >= GEN_SIDE { return; }
    let wrapped_origin = pos::wrapped_chunk(unload_params.origin);

    let dx = unload_params.depths.x;
    let dy = unload_params.depths.y;
    let dz = unload_params.depths.z;

    for (var d = 0u; d < dx; d++) {
        let wrapped_x = (wrapped_origin.x + d) % GEN_SIDE;
        let wrapped = vec3u(wrapped_x, id.x, id.y);
        textureStore(chunk_index_map, wrapped, vec4u(0));
        let addr = flags::addr(wrapped >> vec3u(1u), GEN_SIDE >> 1u);
        atomicAnd(&skip_mip[addr.word], ~(1u << addr.bit));
    }

    if !in_wrapped_range(id.x, wrapped_origin.x, dx) {
        for (var d = 0u; d < dy; d++) {
            let wrapped_y = (wrapped_origin.y + d) % GEN_SIDE;
            let wrapped = vec3u(id.x, wrapped_y, id.y);
            textureStore(chunk_index_map, wrapped, vec4u(0));
            let addr = flags::addr(wrapped >> vec3u(1u), GEN_SIDE >> 1u);
            atomicAnd(&skip_mip[addr.word], ~(1u << addr.bit));
        }
    }

    if !in_wrapped_range(id.x, wrapped_origin.x, dx) &&
       !in_wrapped_range(id.y, wrapped_origin.y, dy) {
        for (var d = 0u; d < dz; d++) {
            let wrapped_z = (wrapped_origin.z + d) % GEN_SIDE;
            let wrapped = vec3u(id.x, id.y, wrapped_z);
            textureStore(chunk_index_map, wrapped, vec4u(0));
            let addr = flags::addr(wrapped >> vec3u(1u), GEN_SIDE >> 1u);
            atomicAnd(&skip_mip[addr.word], ~(1u << addr.bit));
        }
    }
}