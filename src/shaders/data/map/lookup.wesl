import package::data::flags::{ALL_AIR, GEN_FLAGS_CAPACITY};
import package::data::{pos, blocks, flags};
import package::raytrace::cam;

struct LookupResult {
    is_loaded: bool,
    mip_level: u32,
    chunk_idx: u32,
}

@group(0) @binding(1) var chunk_index_map: texture_3d<u32>;
@group(0) @binding(2) var<storage, read_write> gen_flags: array<u32, GEN_FLAGS_CAPACITY>;

fn lookup(chunk_pos: vec3i) -> LookupResult {
    let player_chunk = pos::chunk(vec3i(cam.pos));
    if !pos::is_in_range(chunk_pos, player_chunk) { return LookupResult(false, 1u, blocks::AIR); }
    let wrapped = pos::wrapped_chunk(chunk_pos, player_chunk);
    let value = textureLoad(chunk_index_map, wrapped, 0u).x;
    if value == 0u {
        let addr = flags::addr(wrapped);
        gen_flags[addr.word] |= 1u << addr.bit;
        return LookupResult(false, 1u, blocks::AIR);
    }
    let is_air = value == ALL_AIR;
    return LookupResult(true, select(0u, 1u, is_air), select(value, blocks::AIR, is_air));
}
