import constants::GEN_SIDE;

const OUT_OF_MEMORY = 0xFFFFFFFFu;

struct UnloadParams {
    origin: vec3i,
    depths: vec3u,
}

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

fn in_wrapped_range(coord: u32, range_start: u32, range_len: u32) -> bool {
    if range_len == 0u { return false; }
    let offset_from_start = (coord + GEN_SIDE - range_start) % GEN_SIDE;
    return offset_from_start < range_len;
}