import package::data::{pos, blocks};

const EMPTY = 1u << 31u;

struct LookupResult {
    mip_level: u32,
    chunk_idx: u32,
}

@group(0) @binding(1) var chunk_index_map: texture_3d<u32>; 

fn lookup(chunk_pos: vec3i) -> LookupResult {
    let wrapped_pos = pos::wrapped_chunk(chunk_pos);
    let value = textureLoad(chunk_index_map, wrapped_pos, 0u).x;
    let is_empty = value == EMPTY;
    return LookupResult(select(0u, 1u, is_empty), select(value, blocks::AIR, is_empty));
}
