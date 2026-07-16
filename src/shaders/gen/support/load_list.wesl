import constants::MAX_CHUNK_BATCH_SIZE;

struct LoadList {
    len: atomic<u32>,
    indices: array<u32, MAX_CHUNK_BATCH_SIZE>,
}

struct LoadListView {
    len: u32,
    indices: array<u32>,
}