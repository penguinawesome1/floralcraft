import constants::GEN_SIDE;

const ALL_AIR = 1u << 31u;

struct Addr {
    word: u32,
    bit: u32,
}

fn addr(pos: vec3u, side: u32) -> Addr {
    let linear_idx = pos.z * (side * side) + pos.x * side + pos.y;
    return Addr(linear_idx / 32u, linear_idx % 32u);
}

fn wrapped_chunk(linear_idx: u32) -> vec3u {
    let x = (linear_idx % (GEN_SIDE * GEN_SIDE)) / GEN_SIDE;
    let y = linear_idx % GEN_SIDE;
    let z = linear_idx / (GEN_SIDE * GEN_SIDE);
    return vec3u(x, y, z);
}