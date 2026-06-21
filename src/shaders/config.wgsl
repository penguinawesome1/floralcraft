struct config__Config {
    max_trace_dist: f32,
}

override config__IS_DEBUG_MODE = false;
const config__BITS_PER_ID = 8u;
const config__CHUNK_SIDE_SHIFT = 3u;
const config__SVO_DEPTH = 8u;
const config__SVO_NODES_CAPACITY = 195456u;
const config__CHUNKS_CAPACITY = 65152u;

const_assert 32u % config__BITS_PER_ID == 0u;