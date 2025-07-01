use noise::{ Fbm, SuperSimplex, NoiseFn, MultiFractal, RidgedMulti, Seedable };
use crate::terrain::chunk::CHUNK_DEPTH;
use crate::config::{ NoiseParams, WorldGeneration };
use crate::terrain::block::{ Block, BlockPosition };

pub trait BlockGenerator: Send + Sync + Clone + 'static {
    /// Returns the noise calculated block from the passed global position.
    fn choose_block(&self, pos: BlockPosition, params: &WorldGeneration) -> Block;
}

#[derive(Clone)]
pub struct SkyblockGenerator;

impl BlockGenerator for SkyblockGenerator {
    fn choose_block(&self, pos: BlockPosition, _params: &WorldGeneration) -> Block {
        match pos.z {
            0 => Block::Bedrock,
            z if z < 4 => Block::Dirt,
            4 => Block::Grass,
            _ => Block::Air,
        }
    }
}

#[derive(Clone)]
pub struct FlatGenerator;

impl BlockGenerator for FlatGenerator {
    fn choose_block(&self, pos: BlockPosition, _params: &WorldGeneration) -> Block {
        match pos.z {
            0 => Block::Bedrock,
            z if z < 4 => Block::Dirt,
            4 => Block::Grass,
            _ => Block::Air,
        }
    }
}

#[derive(Clone)]
pub struct NormalGenerator {
    base_noise: Fbm<SuperSimplex>,
    mountain_ridge_noise: RidgedMulti<SuperSimplex>,
    cave_noise: Fbm<SuperSimplex>,
}

impl NormalGenerator {
    /// Initialize the noise functions specific to normal terrain.
    pub fn new(params: &WorldGeneration) -> Self {
        let seed: u32 = params.seed;

        Self {
            base_noise: configure_noise::<SuperSimplex, Fbm<SuperSimplex>, 2>(
                Fbm::<SuperSimplex>::new(seed),
                &params.base_noise
            ),
            mountain_ridge_noise: configure_noise::<SuperSimplex, RidgedMulti<SuperSimplex>, 2>(
                RidgedMulti::<SuperSimplex>::new(seed + 1),
                &params.mountain_ridge_noise
            ),
            cave_noise: configure_noise::<SuperSimplex, Fbm<SuperSimplex>, 3>(
                Fbm::<SuperSimplex>::new(seed + 2),
                &params.cave_noise
            ),
        }
    }

    fn get_density_val(&self, position: BlockPosition) -> f64 {
        self.cave_noise.get([position.x as f64, position.y as f64, position.z as f64])
    }

    fn get_height_val(&self, position: BlockPosition) -> f64 {
        let point: [f64; 2] = [position.x as f64, position.y as f64];
        self.base_noise.get(point) + self.mountain_ridge_noise.get(point) * 0.2
    }
}

impl BlockGenerator for NormalGenerator {
    fn choose_block(&self, pos: BlockPosition, params: &WorldGeneration) -> Block {
        if pos.z == 0 {
            return Block::Bedrock; // place bedrock at world floor
        }

        let density_val: f64 = self.get_density_val(pos);
        if density_val < params.cave_threshold {
            return Block::Air; // carve out caves
        }

        let height_val: f64 = self.get_height_val(pos);
        let height_val_normalized: f64 = (height_val + 1.0) / 2.0;
        let max_height: f64 = (CHUNK_DEPTH as f64) - (params.minimum_air_height as f64);
        let height: i32 = (max_height * height_val_normalized) as i32;
        let dirt_height: i32 = height - params.dirt_height;

        if pos.z > height {
            Block::Air // carve surface level
        } else if pos.z == height {
            Block::Grass // place grass at surface
        } else if pos.z >= dirt_height {
            Block::Dirt
        } else {
            Block::Stone
        }
    }
}

fn configure_noise<T, G, const DIM: usize>(noise_gen: G, params: &NoiseParams) -> G
    where T: NoiseFn<f64, DIM> + Sized + Default + Seedable, G: MultiFractal + Seedable
{
    noise_gen
        .set_octaves(params.octaves)
        .set_frequency(params.frequency)
        .set_lacunarity(params.lacunarity)
        .set_persistence(params.persistence)
}
