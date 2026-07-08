import package::data::{pos, blocks, chunk::read_pool};

struct LookupResult {
    height: u32, // heights greater than 0 are uniform
    block_id: u32,
}

@group(0) @binding(1) var chunk_index_map: texture_3d<u32>; 

fn lookup(global_pos: vec3i) -> LookupResult {
    if !pos::is_in_bounds(global_pos) { return LookupResult(0u, blocks::AIR); }
    let chunk_pos = pos::chunk(global_pos);
    let wrapped_pos = pos::wrapped_chunk(chunk_pos);
    let value = textureLoad(chunk_index_map, wrapped_pos, 0u).x;

    let local_pos = pos::local(global_pos);
    let block_id = read_pool::block_id(value, local_pos);
    return LookupResult(0u, block_id);
}
