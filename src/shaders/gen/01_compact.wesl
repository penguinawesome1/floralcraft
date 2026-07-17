import constants::MAX_CHUNK_BATCH_SIZE;
import package::gen::core::LoadList;

@group(0) @binding(0) var<storage, read> gen_flags: array<u32>;
@group(0) @binding(1) var<storage, read_write> load_list: LoadList;

@compute @workgroup_size(64, 1, 1)
fn compact_load_set(@builtin(global_invocation_id) id: vec3u) {
    let word_idx = id.x;
    if word_idx >= arrayLength(&gen_flags) { return; }
    var word = gen_flags[word_idx];
    while word != 0u {
        let bit = firstTrailingBit(word);
        let linear_idx = word_idx * 32u + bit;
        let slot = atomicAdd(&load_list.len, 1u);
        if slot >= MAX_CHUNK_BATCH_SIZE { return; }
        load_list.indices[slot] = linear_idx;
        word &= word - 1u;
    }
}
