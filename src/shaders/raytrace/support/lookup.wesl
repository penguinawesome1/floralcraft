import constants::{MIP_CAPACITY_L1, MIP_CAPACITY_L2, MIP_CAPACITY_L3, MIP_CAPACITY_L4, MIP_CAPACITY_L5};
import package::core::flags::{ALL_AIR, GEN_FLAGS_CAPACITY};
import package::core::{pos, blocks, flags};
import package::raytrace::renderer::cam;

struct LookupResult {
    is_loaded: bool,
    mip_level: u32,
    chunk_idx: u32,
}

struct SkipMips {
    l1: array<u32, MIP_CAPACITY_L1>,
    l2: array<u32, MIP_CAPACITY_L2>,
    l3: array<u32, MIP_CAPACITY_L3>,
    l4: array<u32, MIP_CAPACITY_L4>,
    l5: array<u32, MIP_CAPACITY_L5>,
}

@group(0) @binding(1) var chunk_index_map: texture_3d<u32>;
@group(0) @binding(2) var<storage, read_write> gen_flags: array<u32, GEN_FLAGS_CAPACITY>;
@group(0) @binding(3) var<storage, read> skip_mips: SkipMips;

fn _flag(wrapped: vec3u) {
    let addr = flags::addr(wrapped);
    gen_flags[addr.word] |= 1u << addr.bit;
}

fn lookup(chunk_pos: vec3i) -> LookupResult {
    let player_chunk = pos::chunk(vec3i(cam.pos));
    if !pos::is_in_range(chunk_pos, player_chunk) { return LookupResult(false, 1u, blocks::AIR); }
    let wrapped = pos::wrapped_chunk(chunk_pos, player_chunk);
    let value = textureLoad(chunk_index_map, wrapped, 0u).x;
    if value == 0u {
        _flag(wrapped);
        return LookupResult(false, 0u, blocks::AIR);
    }
    let is_air = value == ALL_AIR;
    return LookupResult(true, select(0u, 1u, is_air), select(value, blocks::AIR, is_air));
}
