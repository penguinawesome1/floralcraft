import constants::{CHUNK_SIDE, BITS_PER_ID};
import package::core::{pos, blocks, flags, flags::ALL_AIR, player::Player};
import package::gen::core::{LoadListView, OUT_OF_MEMORY, FreeList};
import package::gen::terrain_shape;

const _CHUNK_VOLUME = CHUNK_SIDE * CHUNK_SIDE * CHUNK_SIDE;
const _CHUNK_LEN = _CHUNK_VOLUME * BITS_PER_ID / 32u;
const _IDS_PER_WORD = 32u / BITS_PER_ID;
const _COL_LEN = CHUNK_SIDE / _IDS_PER_WORD;
const _STEEPNESS_SAMPLE_OFFSET = 10i;
const _STEEPNESS_STONE_THRESHOLD = 50.0;
const _SURFACE_STONE_THRESHOLD = 10.0;
const _SURFACE_DIRT_THRESHOLD = 3.0;

@group(0) @binding(0) var<storage, read> load_list: LoadListView;
@group(0) @binding(1) var<storage, read> player: Player;
@group(0) @binding(2) var<storage, read_write> free_list: FreeList;
@group(0) @binding(3) var chunk_index_map: texture_storage_3d<r32uint, write>;
@group(0) @binding(4) var<storage, read_write> chunk_pool: array<u32>;

fn _insert(chunk_pos: vec3i, value: u32) {
    let wrapped = pos::wrapped_chunk(chunk_pos);
    textureStore(chunk_index_map, wrapped, vec4u(value, 0u, 0u, 0u));
}

fn _has_free_slot() -> bool {
    return atomicLoad(&free_list.len) < arrayLength(&free_list.indices);
}

fn _alloc() -> u32 {
    let slot = atomicAdd(&free_list.len, 1u);
    if slot >= arrayLength(&free_list.indices) {
        atomicSub(&free_list.len, 1u);
        return OUT_OF_MEMORY;
    }
    return free_list.indices[slot];
}

fn _choose_block_id(pos: vec3i, surface_height: f32, steepness: f32) -> u32 {
    var bid = blocks::AIR;
    let surface_density = terrain_shape::surface_density(pos, surface_height);
    if surface_density > 0.0 {
        let cave_density = terrain_shape::cave_density(pos, surface_height);
        if cave_density < 1.0 {
            if surface_density > _SURFACE_STONE_THRESHOLD || steepness > _STEEPNESS_STONE_THRESHOLD {
                bid = blocks::STONE;
            } else if surface_density > _SURFACE_DIRT_THRESHOLD {
                bid = blocks::DIRT;
            } else {
                bid = blocks::GRASS;
            }
        }
    }
    return bid;
}

fn _gen_column(chunk_pos: vec3i, column_pos: vec3u, temp: ptr<function, array<u32, _COL_LEN>>) -> bool {
    let origin_pos = chunk_pos * i32(CHUNK_SIDE);
    let base_pos = origin_pos.xz + vec2i(column_pos.xz);
    let surface_height = terrain_shape::surface_height(base_pos);

    let height_dx = terrain_shape::surface_height(base_pos + vec2i(_STEEPNESS_SAMPLE_OFFSET, 0));
    let height_dz = terrain_shape::surface_height(base_pos + vec2i(0, _STEEPNESS_SAMPLE_OFFSET));
    let steepness = max(abs(height_dx - surface_height), abs(height_dz - surface_height));

    var col_is_air = true;
    for (var i = 0u; i < _COL_LEN; i++) {
        let word_base_y = origin_pos.y + i32(i * _IDS_PER_WORD);
        for (var j = 0u; j < _IDS_PER_WORD; j++) {
            let y = word_base_y + i32(j);
            let pos = vec3i(base_pos.x, i32(y), base_pos.y);
            let bid = _choose_block_id(pos, surface_height, steepness);
            (*temp)[i] = insertBits((*temp)[i], bid, j * BITS_PER_ID, BITS_PER_ID);
        }
        if (*temp)[i] != 0u { col_is_air = false; }
    }
    return col_is_air;
}

var<workgroup> is_alive: bool;
var<workgroup> chunk_is_air: atomic<u32>;
var<workgroup> chunk_num: u32;

@compute @workgroup_size(CHUNK_SIDE, 1, CHUNK_SIDE)
fn gen_chunk(
    @builtin(local_invocation_index) lid: u32,
    @builtin(local_invocation_id) column_pos: vec3u,
    @builtin(workgroup_id) wgid: vec3u
) {
    let wrapped = flags::wrapped_chunk(load_list.indices[wgid.x]);
    let player_chunk = pos::chunk(vec3i(floor(player.pos)));
    let chunk_pos = pos::unwrapped_chunk(wrapped, player_chunk);
    if lid == 0u {
        if terrain_shape::is_definitely_empty(chunk_pos.y * i32(CHUNK_SIDE)) {
            _insert(chunk_pos, ALL_AIR);
            is_alive = false;
        } else {
            is_alive = _has_free_slot();
            atomicStore(&chunk_is_air, 1u);
        }
    }
    workgroupBarrier();
    var temp = array<u32, _COL_LEN>();
    if is_alive && !_gen_column(chunk_pos, column_pos, &temp) {
        atomicMin(&chunk_is_air, 0u);
    }
    workgroupBarrier();
    if is_alive && lid == 0u {
        if atomicLoad(&chunk_is_air) == 1u {
            _insert(chunk_pos, ALL_AIR);
            is_alive = false;
        } else {
            chunk_num = _alloc();
            is_alive &= chunk_num != OUT_OF_MEMORY;
        }
    }
    workgroupBarrier();
    if is_alive {
        let chunk_base_idx = chunk_num * _CHUNK_LEN;
        let linear_idx = column_pos.z * (CHUNK_SIDE * CHUNK_SIDE) + column_pos.x * CHUNK_SIDE;
        let start_word = chunk_base_idx + (linear_idx * BITS_PER_ID / 32u);
        for (var i = 0u; i < _COL_LEN; i++) {
            chunk_pool[start_word + i] = temp[i];
        }
    }
    workgroupBarrier();
    if is_alive && lid == 0u {
        _insert(chunk_pos, chunk_num);
    }
}
