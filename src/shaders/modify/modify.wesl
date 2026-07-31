import constants::{CHUNK_SIDE, BITS_PER_ID};
import package::config::Config;
import package::core::{pos, flags, blocks, block_target::BlockTarget};
import package::gen::core::{FreeList, OUT_OF_MEMORY};
import package::modify::core::ChunkAllocResult;

const CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const CHUNK_LEN = CHUNK_VOLUME * BITS_PER_ID / 32u;
const BLOCK_MASK = (1u << BITS_PER_ID) - 1u;

struct _ChunkAddr {
    word_idx: u32,
    bit_idx: u32,
}

@group(0) @binding(0) var<uniform> config: Config;
@group(0) @binding(1) var<storage, read> block_target: BlockTarget;
@group(0) @binding(2) var chunk_index_map: texture_3d<u32>;
@group(0) @binding(3) var<storage, read_write> chunk_pool: array<u32>;
@group(0) @binding(4) var<storage, read_write> free_list: FreeList;
@group(0) @binding(5) var<storage, read_write> alloc_result: ChunkAllocResult;

fn _alloc() -> u32 {
    let slot = atomicAdd(&free_list.len, 1u);
    if slot >= arrayLength(&free_list.indices) {
        atomicSub(&free_list.len, 1u);
        return OUT_OF_MEMORY;
    }
    return free_list.indices[slot];
}

fn _chunk_addr(local_pos: vec3u) -> _ChunkAddr {
    let linear_idx = local_pos.z * (CHUNK_SIDE * CHUNK_SIDE) + local_pos.x * CHUNK_SIDE + local_pos.y;
    let offset = linear_idx * BITS_PER_ID;
    return _ChunkAddr(offset / 32u, offset % 32u);
}

@compute @workgroup_size(1, 1, 1)
fn modify_target_block() {
    alloc_result.is_store_needed = 0u;
    if config.pending_action == 0u || block_target.is_block_hit == 0u { return; }

    let chunk_pos = pos::chunk(block_target.pos);
    let wrapped = pos::wrapped_chunk(chunk_pos);
    var chunk_num = textureLoad(chunk_index_map, wrapped, 0u).x;

    if chunk_num == flags::ALL_AIR {
        chunk_num = _alloc();
        if chunk_num == OUT_OF_MEMORY { return; }
        let chunk_base_idx = chunk_num * CHUNK_LEN;
        for (var i = 0u; i < CHUNK_LEN; i++) {
            chunk_pool[chunk_base_idx + i] = 0u;
        }
        alloc_result.chunk_num = chunk_num;
        alloc_result.wrapped = wrapped;
        alloc_result.is_store_needed = 1u;
    }

    let local = pos::local(block_target.pos);
    let addr = _chunk_addr(local);
    let chunk_base_idx = chunk_num * CHUNK_LEN;

    if config.pending_action == 1u {
        chunk_pool[chunk_base_idx + addr.word_idx] &= ~(BLOCK_MASK << addr.bit_idx);
    } else {
        chunk_pool[chunk_base_idx + addr.word_idx] |= (BLOCK_MASK << addr.bit_idx);
    }
}
