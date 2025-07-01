use rand_pcg::Pcg64;
use rand::SeedableRng;
use crate::config::WorldGeneration;
use crate::terrain::World;
use crate::terrain::block::{ Block, BlockPosition };
use crate::terrain::chunk::{ Chunk, ChunkPosition };

use crate::terrain_management::block_generator::{
    BlockGenerator,
    SkyblockGenerator,
    FlatGenerator,
    NormalGenerator,
};

pub fn new_generator(params: &WorldGeneration) -> Box<dyn WorldGeneratorTrait> {
    let seed: u32 = params.seed;
    match params.world_mode.as_str() {
        "skyblock" => Box::new(WorldGenerator::new(SkyblockGenerator, seed)),
        "flat" => Box::new(WorldGenerator::new(FlatGenerator, seed)),
        "normal" | _ => Box::new(WorldGenerator::new(NormalGenerator::new(params), seed)),
    }
}

pub trait WorldGeneratorTrait: Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn WorldGeneratorTrait>;
    fn generate_chunk(&self, pos: ChunkPosition, params: &WorldGeneration) -> Chunk;
}

#[derive(Clone)]
pub struct WorldGenerator<T> where T: BlockGenerator + Send + Sync + Clone + 'static {
    block_generator: T,
    _rng: Pcg64,
}

impl<T> WorldGenerator<T>
    where
        T: BlockGenerator + Send + Sync + 'static + Clone // Bounds for the struct's inherent methods
{
    fn new(block_generator: T, seed: u32) -> Self {
        let mut seed_bytes: [u8; 32] = [0; 32];
        seed_bytes[..4].copy_from_slice(&seed.to_le_bytes());
        Self {
            block_generator,
            _rng: Pcg64::from_seed(seed_bytes),
        }
    }
}

impl<T> WorldGeneratorTrait
    for WorldGenerator<T>
    where
        T: BlockGenerator + Send + Sync + 'static + Clone // Bounds for trait implementation
{
    fn clone_box(&self) -> Box<dyn WorldGeneratorTrait> {
        Box::new(self.clone())
    }

    fn generate_chunk(&self, pos: ChunkPosition, params: &WorldGeneration) -> Chunk {
        let mut chunk: Chunk = Chunk::new(pos);
        let chunk_block_pos: BlockPosition = World::chunk_to_block_pos(pos);

        let generated_blocks: Vec<(BlockPosition, Block)> = Chunk::chunk_coords()
            .map(|pos| {
                let world_pos: BlockPosition = chunk_block_pos + pos;
                let block: Block = self.block_generator.choose_block(world_pos, params);

                (pos, block)
            })
            .collect();

        generated_blocks.into_iter().for_each(|(pos, block)| {
            chunk.set_block(pos, block);
        });

        chunk
    }
}
