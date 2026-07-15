import constants::MAX_CHUNK_BATCH_SIZE;

struct LoadList {
    len: u32,
    indices: array<u32>,
}

@group(0) @binding(1) var<storage, read> load_list: LoadList;
@group(0) @binding(0) var<storage, read_write> indirect_args: array<u32, 3>;

@compute @workgroup_size(1, 1, 1)
fn write_indirect_args() {
    indirect_args[0] = min(load_list.len, MAX_CHUNK_BATCH_SIZE);
    indirect_args[1] = 1u;
    indirect_args[2] = 1u;
}
