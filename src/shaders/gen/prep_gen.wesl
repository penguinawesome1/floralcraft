import constants::MAX_CHUNK_BATCH_SIZE;
import package::gen::core::LoadListView;

@group(0) @binding(0) var<storage, read> load_list: LoadListView;
@group(0) @binding(1) var<storage, read_write> indirect_args: array<u32, 6>;

@compute @workgroup_size(1, 1, 1)
fn prep_gen() {
    let n = min(load_list.len, MAX_CHUNK_BATCH_SIZE);
    indirect_args[0] = n;
    indirect_args[1] = 1u;
    indirect_args[2] = 1u;
    indirect_args[3] = (n + 63u) / 64u;
    indirect_args[4] = 1u;
    indirect_args[5] = 1u;
}
