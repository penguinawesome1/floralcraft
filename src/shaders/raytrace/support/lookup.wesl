import constants::GEN_SIDE;
import package::core::flags::ALL_AIR;
import package::core::{pos, blocks, flags};

struct LookupResult {
    is_loaded: bool,
    mip_level: u32,
    chunk_idx: u32,
}

@group(0) @binding(1) var chunk_index_map: texture_3d<u32>;
@group(0) @binding(2) var<storage, read_write> gen_flags: array<u32>;
@group(0) @binding(3) var<storage, read> skip_mip: array<u32>;

fn _flag(wrapped: vec3u) {
    let addr = flags::addr(wrapped, GEN_SIDE);
    gen_flags[addr.word] |= 1u << addr.bit;
}

fn lookup(chunk_pos: vec3i, player_chunk: vec3i) -> LookupResult {
    if !pos::is_in_range(chunk_pos, player_chunk) { return LookupResult(false, 1u, blocks::AIR); }
    let wrapped = pos::wrapped_chunk(chunk_pos, player_chunk);

    let addr = flags::addr(wrapped >> vec3u(1u), GEN_SIDE >> 1u);
    let val = skip_mip[addr.word];
    let mask = 1u << addr.bit;
    if (val & mask) != 0u { return LookupResult(true, 2u, blocks::AIR); }

    let value = textureLoad(chunk_index_map, wrapped, 0u).x;
    if value == 0u {
        _flag(wrapped);
        return LookupResult(false, 1u, blocks::AIR);
    }
    let is_air = value == ALL_AIR;
    return LookupResult(true, select(0u, 1u, is_air), select(value, blocks::AIR, is_air));
}
