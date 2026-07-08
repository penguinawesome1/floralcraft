import constants::CHUNK_SIDE_SHIFT;
import package::data::pos;

const CHUNK_SIDE = 1u << CHUNK_SIDE_SHIFT;

@group(0) @binding(1) var chunk_index_map: texture_storage_3d<r32uint, write>; 

fn insert(chunk_pos: vec3i, chunk_num: u32) {
    if !pos::is_in_bounds(chunk_pos * i32(CHUNK_SIDE)) { return; }
    let wrapped_pos = pos::wrapped_chunk(chunk_pos);
    textureStore(chunk_index_map, wrapped_pos, vec4u(chunk_num, 0u, 0u, 0u));
}