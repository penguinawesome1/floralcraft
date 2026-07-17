import package::core::{flags, flags::ALL_AIR};
import package::gen::core::{LoadListView, FreeList};

@group(0) @binding(0) var<storage, read> load_list: LoadListView;
@group(0) @binding(1) var chunk_index_map: texture_3d<u32>; 
@group(0) @binding(2) var<storage, read_write> free_list: FreeList;

fn _free(idx: u32) {
    let slot = atomicSub(&free_list.len, 1u) - 1u;
    free_list.indices[slot] = idx;
    // Note: chunk_index_map still points at `idx` until gen_chunk's insert()
    // overwrites it. Safe only because nothing reads chunk_index_map between
    // this evict pass and the gen pass in the same frame.
}

@compute @workgroup_size(64, 1, 1)
fn free_chunk(@builtin(global_invocation_id) id: vec3u) {
    let wrapped = flags::wrapped_chunk(load_list.indices[id.x]);
    let idx = textureLoad(chunk_index_map, wrapped, 0u).x;
    if idx != 0u && idx != ALL_AIR { _free(idx); }
}