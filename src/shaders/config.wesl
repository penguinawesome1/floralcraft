override IS_DEBUG_MODE = false;
const BITS_PER_ID = 8u;
const CHUNK_SIDE_SHIFT = 3u;
const SVO_DEPTH = 8u;
const SVO_NODES_CAPACITY = 195456u;
const CHUNKS_CAPACITY = 65152u;

struct Config {
    max_trace_dist: f32,
}

@group(1) @binding(2) var<uniform> config: Config;

fn _validate_config_bounds() {
    const_assert 32u % BITS_PER_ID == 0u;
    const_assert BITS_PER_ID <= 18u;
    const_assert SVO_DEPTH <= 31u;
}