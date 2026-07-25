import constants::GEN_SIDE;
import package::modify::core::ChunkAllocResult;
import package::core::flags;

@group(0) @binding(0) var<storage, read> alloc_result: ChunkAllocResult;
@group(0) @binding(1) var chunk_index_map: texture_storage_3d<r32uint, write>;
@group(0) @binding(2) var<storage, read_write> skip_mip: array<u32>;

@compute @workgroup_size(1, 1, 1)
fn store_chunk_num() {
    if alloc_result.is_store_needed == 0u { return; }
    textureStore(chunk_index_map, alloc_result.wrapped, vec4u(alloc_result.chunk_num, 0u, 0u, 0u));
    let addr = flags::addr(alloc_result.wrapped >> vec3u(1u), GEN_SIDE >> 1u);
    skip_mip[addr.word] &= ~(1u << addr.bit);
}