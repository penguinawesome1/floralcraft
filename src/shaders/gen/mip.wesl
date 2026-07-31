import constants::GEN_SIDE;
import package::core::{flags, flags::ALL_AIR};
import package::gen::core::LoadListView;

@group(0) @binding(0) var<storage, read> load_list: LoadListView;
@group(0) @binding(1) var chunk_index_map: texture_3d<u32>;
@group(0) @binding(2) var<storage, read_write> skip_mip: array<atomic<u32>>;

@compute @workgroup_size(64, 1, 1)
fn expand_skip_mip(@builtin(global_invocation_id) id: vec3u) {
    if id.x >= load_list.len { return; }
    let wrapped = flags::wrapped_chunk(load_list.indices[id.x]);
    if textureLoad(chunk_index_map, wrapped, 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(1u, 0u, 0u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(0u, 1u, 0u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(0u, 0u, 1u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(1u, 1u, 0u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(1u, 0u, 1u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(0u, 1u, 1u), 0u).x != ALL_AIR { return; }
    if textureLoad(chunk_index_map, wrapped ^ vec3u(1u, 1u, 1u), 0u).x != ALL_AIR { return; }
    let addr = flags::addr(wrapped >> vec3u(1u), GEN_SIDE >> 1u);
    atomicOr(&skip_mip[addr.word], 1u << addr.bit);
}