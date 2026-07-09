import constants::{CHUNK_SIDE_SHIFT, BITS_PER_ID};
import package::data::{chunk::alloc, map::insert, noise, blocks};
import package::data::map::lookup::EMPTY;

const _CHUNK_SIDE = 1u << CHUNK_SIDE_SHIFT;
const _CHUNK_VOLUME = _CHUNK_SIDE * _CHUNK_SIDE * _CHUNK_SIDE;
const _CHUNK_LEN = _CHUNK_VOLUME * BITS_PER_ID / 32u;
const _IDS_PER_WORD = 32u / BITS_PER_ID;
const _COL_LEN = _CHUNK_SIDE / _IDS_PER_WORD;

@group(0) @binding(0) var<storage, read_write> chunk_pool: array<u32>;

var<workgroup> is_out_of_memory: bool;
var<workgroup> is_empty: bool;
var<workgroup> chunk_num: u32;

@compute @workgroup_size(_CHUNK_SIDE, 1, _CHUNK_SIDE)
fn gen_chunk(
    @builtin(local_invocation_index) lid: u32,
    @builtin(local_invocation_id) column_pos: vec3u,
    @builtin(workgroup_id) chunk_pos: vec3u
) {
    if lid == 0u {
        is_out_of_memory = !alloc::has_free_slot();
        is_empty = true;
    }
    workgroupBarrier();

    if is_out_of_memory { return; }

    let origin_pos = chunk_pos * _CHUNK_SIDE;
    let pos = vec2f(origin_pos.xz + column_pos.xz) * 0.04;
    let noise_normalized = noise::simplex2d(pos) * 0.5 + 0.5;
    let scaled = u32(noise_normalized * 50.0);

    var temp = array<u32, _COL_LEN>();
    for (var i = 0u; i < _COL_LEN; i++) {
        let word_base_y = origin_pos.y + (i * _IDS_PER_WORD);
        for (var j = 0u; j < _IDS_PER_WORD; j++) {
            let y = word_base_y + j;
            var block_id = blocks::AIR;

            if y == 0u { block_id = blocks::BEDROCK; }
            else if y + 4u < scaled { block_id = blocks::STONE; }
            else if y < scaled { block_id = blocks::DIRT; }
            else if y == scaled { block_id = blocks::GRASS; }
            else { block_id = blocks::AIR; }

            temp[i] = insertBits(temp[i], block_id, j * BITS_PER_ID, BITS_PER_ID);
        }
        if temp[i] != 0u { is_empty = false; }
    }
    workgroupBarrier();

    if is_empty {
        if lid == 0u { insert::insert(vec3i(chunk_pos), EMPTY); }
        return;
    }
    if lid == 0u { chunk_num = alloc::alloc(); }
    workgroupBarrier();

    if chunk_num == alloc::OUT_OF_MEMORY { return; }
    let chunk_base_idx = chunk_num * _CHUNK_LEN;
    let linear_idx = column_pos.z * (_CHUNK_SIDE * _CHUNK_SIDE) + column_pos.x * _CHUNK_SIDE;
    let start_word = chunk_base_idx + (linear_idx * BITS_PER_ID / 32u);
    for (var i = 0u; i < _COL_LEN; i++) {
        chunk_pool[start_word + i] = temp[i];
    }
    workgroupBarrier();

    if lid == 0u { insert::insert(vec3i(chunk_pos), chunk_num); }
}
