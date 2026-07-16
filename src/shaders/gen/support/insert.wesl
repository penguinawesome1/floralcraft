import constants::{MIP_CAPACITY_L1, MIP_CAPACITY_L2, MIP_CAPACITY_L3, MIP_CAPACITY_L4, MIP_CAPACITY_L5};
import package::core::{pos, flags};

struct Camera {
    pos: vec3f,
    rotation: mat4x4f,
}

struct SkipMips {
    l1: array<atomic<u32>, MIP_CAPACITY_L1>,
    l2: array<atomic<u32>, MIP_CAPACITY_L2>,
    l3: array<atomic<u32>, MIP_CAPACITY_L3>,
    l4: array<atomic<u32>, MIP_CAPACITY_L4>,
    l5: array<atomic<u32>, MIP_CAPACITY_L5>,
}

@group(0) @binding(1) var chunk_index_map: texture_storage_3d<r32uint, write>; 
@group(0) @binding(3) var<uniform> cam: Camera;
@group(0) @binding(5) var<storage, read_write> skip_mips: SkipMips;

fn insert(chunk_pos: vec3i, value: u32) {
    let player_chunk = pos::chunk(vec3i(cam.pos));
    let wrapped = pos::wrapped_chunk(chunk_pos, player_chunk);
    textureStore(chunk_index_map, wrapped, vec4u(value, 0u, 0u, 0u));
}
