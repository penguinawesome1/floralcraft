import constants::MAX_CHUNKS_LOADED;

const OUT_OF_MEMORY = 0xFFFFFFFFu;

struct _FreeList {
    len: atomic<u32>,
    indices: array<u32, MAX_CHUNKS_LOADED>,
}

@group(0) @binding(2) var<storage, read_write> _free_list: _FreeList;

fn has_free_slot() -> bool {
    return atomicLoad(&_free_list.len) < MAX_CHUNKS_LOADED;
}

fn alloc() -> u32 {
    let slot = atomicAdd(&_free_list.len, 1u);
    if slot >= MAX_CHUNKS_LOADED {
        atomicSub(&_free_list.len, 1u);
        return OUT_OF_MEMORY;
    }
    return _free_list.indices[slot];
}

fn free(idx: u32) {
    let slot = atomicSub(&_free_list.len, 1u);
    _free_list.indices[slot] = idx;
}