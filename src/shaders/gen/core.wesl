const OUT_OF_MEMORY = 0xFFFFFFFFu;

struct FreeList {
    len: atomic<u32>,
    indices: array<u32>,
}

struct LoadList {
    len: atomic<u32>,
    indices: array<u32>,
}

struct LoadListView {
    len: u32,
    indices: array<u32>,
}