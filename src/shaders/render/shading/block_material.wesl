import package::core::blocks;

struct BlockMaterial {
    spectral_coeffs: vec3f,
    roughness: f32,
    metallic: f32,
    f0: f32,
    emission: f32,
}

fn lookup(id: u32) -> BlockMaterial {
    switch id {
        case blocks::GRASS: {
            return BlockMaterial(vec3f(-12.0, 12.0, -2.2), 0.5, 0.0, 0.04, 0.0);
        }
        case blocks::DIRT: {
            return BlockMaterial(vec3f(5.8, 1.2, -4.3), 0.92, 0.0, 0.04, 0.0);
        }
        case blocks::STONE: {
            return BlockMaterial(vec3f(0.5, -0.1, -1.0), 0.65, 0.0, 0.04, 0.0);
        }
        case blocks::SAND: {
            return BlockMaterial(vec3f(3.5, 0.6, -0.3), 0.85, 0.0, 0.04, 0.0);
        }
        case blocks::BASALT: {
            return BlockMaterial(vec3f(-0.9141, 0.6019, -2.4712), 0.6, 0.0, 0.04, 0.0);
        }
        case blocks::SLATE: {
            return BlockMaterial(vec3f(-1.8012, -0.5785, -0.5657), 0.35, 0.0, 0.05, 0.0);
        }
        case blocks::BEDROCK: {
            return BlockMaterial(vec3f(-1.6307, 0.9414, -3.6511), 0.95, 0.0, 0.04, 0.0);
        }
        case blocks::ICE: {
            return BlockMaterial(vec3f(-0.2226, -2.4808, 2.0462), 0.05, 0.0, 0.02, 0.0);
        }
        default: {
            return BlockMaterial(vec3f(1.0, 1.0, 1.0), 0.0, 0.0, 0.04, 15.0);
        }
    }
}