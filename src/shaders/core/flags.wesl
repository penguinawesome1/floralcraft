import constants::GEN_SIDE;

const ALL_AIR = 1u << 31u;

struct Addr {
    word: u32,
    bit: u32,
}

fn addr(wrapped_chunk: vec3u) -> Addr {
    let linear_idx = wrapped_chunk.z * (GEN_SIDE * GEN_SIDE) + wrapped_chunk.x * GEN_SIDE + wrapped_chunk.y;
    return Addr(linear_idx / 32u, linear_idx % 32u);
}

fn wrapped_chunk(linear_idx: u32) -> vec3u {
    let x = (linear_idx % (GEN_SIDE * GEN_SIDE)) / GEN_SIDE;
    let y = linear_idx % GEN_SIDE;
    let z = linear_idx / (GEN_SIDE * GEN_SIDE);
    return vec3u(x, y, z);
}