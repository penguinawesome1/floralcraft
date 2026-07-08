const OUT_OF_MEMORY = 0xFFFFFFFFu;

struct _FreeList {
    len: atomic<u32>,
    indices: array<u32>,
}

@group(0) @binding(2) var<storage, read_write> _free_list: _FreeList;

fn alloc() -> u32 {
    let old_len = atomicSub(&_free_list.len, 1u) - 1u;
    if old_len == 0u {
        atomicSub(&_free_list.len, 1u);
        return OUT_OF_MEMORY;
    }
    return _free_list.indices[old_len];
}

fn free(idx: u32) {
    let old_len = atomicAdd(&_free_list.len, 1u);
    _free_list.indices[old_len] = idx;
}
