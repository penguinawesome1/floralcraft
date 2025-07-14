mod block_generator;

use crate::config::{ WorldGeneration, WorldMode };
use bevy::prelude::Resource;
use block_generator::{ BlockGenerator, FlatGenerator, NormalGenerator, SkyblockGenerator };
use floralcraft_terrain::{ BlockPosition, ChunkAccessError, ChunkPosition, World };

const SUBCHUNK_DEPTH: usize = 16;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 16;
pub const CHUNK_DEPTH: usize = SUBCHUNK_DEPTH * 4;

pub type SizedWorld = World<CHUNK_WIDTH, CHUNK_HEIGHT, CHUNK_DEPTH, SUBCHUNK_DEPTH>;

mod block_dictionary {
    use block_dictionary::{ Block, load_blocks };
    use once_cell::sync::Lazy;

    static BLOCK_DICTIONARY: Lazy<Result<Vec<Block>, block_dictionary::CliError>> = Lazy::new(||
        load_blocks("Blocks.toml")
    );

    pub fn definition(value: u8) -> Block {
        match BLOCK_DICTIONARY.as_ref() {
            Ok(dictionary) =>
                dictionary
                    .get(value as usize)
                    .copied()
                    .unwrap_or(Block::MISSING),
            Err(e) => {
                eprintln!("Error loading block dictionary: {}", e);
                Block::MISSING
            }
        }
    }
}

#[derive(Resource)]
pub struct WorldController {
    pub world: SizedWorld,
    generator: Box<dyn BlockGenerator>,
}

impl WorldController {
    pub fn new(params: &WorldGeneration) -> Self {
        Self {
            world: SizedWorld::default(),
            generator: match params.world_mode {
                WorldMode::Normal => Box::new(NormalGenerator::new(params)),
                WorldMode::Flat => Box::new(FlatGenerator),
                WorldMode::Skyblock => Box::new(SkyblockGenerator),
            },
        }
    }

    pub fn update(&mut self, params: &WorldGeneration) {
        let origin: ChunkPosition = ChunkPosition::new(0, 0);
        let radius: u32 = 0;

        for chunk_pos in SizedWorld::positions_in_square(origin, radius) {
            if self.world.add_default_chunk(chunk_pos).is_ok() {
                self.set_up_chunk(chunk_pos, params);
            }
        }
    }

    fn set_up_chunk(&mut self, pos: ChunkPosition, params: &WorldGeneration) {
        let block_set = self.generator.generate_chunk_blocks(pos, params).into_iter();

        unsafe {
            if let Err(e) = self.world.decorate_chunk(pos, block_set) {
                eprintln!("Failed to decorate chunk {}: {:?}", pos, e);
                return;
            }
        }

        if let Err(e) = self.update_exposed_blocks(pos) {
            eprintln!("Failed to update exposed blocks {}: {:?}", pos, e);
        }
    }

    #[must_use]
    fn update_exposed_blocks(&mut self, chunk_pos: ChunkPosition) -> Result<(), ChunkAccessError> {
        for pos in SizedWorld::chunk_coords(chunk_pos) {
            let is_exposed: bool = self.is_exposed(pos);

            unsafe {
                self.world.set_block_exposed(pos, is_exposed)?;
            }
        }

        Ok(())
    }

    fn is_exposed(&self, pos: BlockPosition) -> bool {
        let block: u8 = unsafe { self.world.block(pos).unwrap() };
        if !block_dictionary::definition(block).is_visible() {
            return false; // block is not exposed if it is not visible
        }

        SizedWorld::block_offsets(pos).any(|adj_pos| {
            unsafe {
                match self.world.block(adj_pos) {
                    Ok(adj_block) => !block_dictionary::definition(adj_block).is_visible(),
                    Err(ChunkAccessError::ChunkUnloaded) => false,
                }
            }
        })
    }
}
