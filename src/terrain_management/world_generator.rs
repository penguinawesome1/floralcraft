use itertools::iproduct;
use rand_pcg::Pcg64;
use rand::SeedableRng;
use crate::config::WorldGeneration;
use crate::terrain::{
    BlockPosition,
    ChunkPosition,
    Conversion,
    Chunk,
    CHUNK_WIDTH,
    CHUNK_HEIGHT,
    CHUNK_DEPTH,
    Block,
};
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
        let chunk_origin_block_pos: BlockPosition = Conversion::chunk_to_block_pos(pos);

        let generated_blocks: Vec<(BlockPosition, Block)> = iproduct!(
            0..CHUNK_WIDTH as i32,
            0..CHUNK_HEIGHT as i32,
            0..CHUNK_DEPTH as i32
        )
            .map(|(x, y, z)| {
                let local_block_pos: BlockPosition = BlockPosition::new(x, y, z);
                let world_block_pos: BlockPosition = chunk_origin_block_pos + local_block_pos;
                let block_name: Block = self.block_generator.get_block(world_block_pos, params);

                (local_block_pos, block_name)
            })
            .collect();

        generated_blocks.into_iter().for_each(|(local_block_pos, block_name)| {
            chunk.set_block_name(local_block_pos, block_name);
        });

        chunk
    }
}
