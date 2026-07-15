import package::data::pos;

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

@group(0) @binding(1) var chunk_index_map: texture_storage_3d<r32uint, write>; 
@group(0) @binding(3) var<uniform> cam: Camera;

fn insert(chunk_pos: vec3i, value: u32) {
    let player_chunk = pos::chunk(vec3i(cam.pos));
    let wrapped = pos::wrapped_chunk(chunk_pos, player_chunk);
    textureStore(chunk_index_map, wrapped, vec4u(value, 0u, 0u, 0u));
}
