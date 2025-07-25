mod block_dictionary;
mod block_generator;

use crate::config::{WorldGeneration, WorldMode};
use bevy::prelude::Resource;
use block_dictionary::definition;
use block_generator::{BlockGenerator, FlatGenerator, NormalGenerator, SkyblockGenerator};
use floralcraft_terrain::{BlockPosition, ChunkAccessError, ChunkPosition, World};
use glam::IVec3;
use std::collections::HashSet;

pub const CHUNK_WIDTH: usize = 16;
pub const CHUNK_HEIGHT: usize = 16;
pub const SUBCHUNK_DEPTH: usize = 16;
pub const NUM_SUBCHUNKS: usize = 16;

pub const CHUNK_DEPTH: usize = SUBCHUNK_DEPTH * NUM_SUBCHUNKS;
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

pub type SizedWorld = World<CHUNK_WIDTH, CHUNK_HEIGHT, SUBCHUNK_DEPTH, NUM_SUBCHUNKS>;

#[derive(Resource, Default)]
pub struct DirtyChunks(pub HashSet<ChunkPosition>);

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

    pub fn update<I>(
        &mut self,
        params: &WorldGeneration,
        dirty_chunks: &mut DirtyChunks,
        positions: I,
    ) where
        I: Iterator<Item = ChunkPosition>,
    {
        for chunk_pos in positions {
            if self.world.add_default_chunk(chunk_pos).is_ok() {
                let _ = self
                    .set_up_chunk(chunk_pos, params)
                    .map_err(|e| eprintln!("Failed to set up chunk {}: {:?}", chunk_pos, e));

                dirty_chunks.0.insert(chunk_pos);
            }
        }
    }

    #[must_use]
    fn set_up_chunk(
        &mut self,
        chunk_pos: ChunkPosition,
        params: &WorldGeneration,
    ) -> Result<(), ChunkAccessError> {
        let origin_block_pos: IVec3 = SizedWorld::chunk_to_block_pos(chunk_pos);

        self.world.decorate_chunk(chunk_pos, |chunk, pos| unsafe {
            let block: u8 = self.generator.choose_block(origin_block_pos + pos, params);
            chunk.set_block(pos, block);
        })?;

        self.update_exposed_blocks(chunk_pos)?;

        Ok(())
    }

    #[must_use]
    fn update_exposed_blocks(&mut self, chunk_pos: ChunkPosition) -> Result<(), ChunkAccessError> {
        let mut exposed_states = SizedWorld::chunk_coords(chunk_pos)
            .map(|pos| unsafe { self.is_exposed(pos) })
            .collect::<Result<Vec<bool>, ChunkAccessError>>()?
            .into_iter();

        self.world.decorate_chunk(chunk_pos, |chunk, pos| unsafe {
            chunk.set_block_exposed(pos, exposed_states.next().unwrap());
        })?;

        Ok(())
    }

    unsafe fn is_exposed(&self, pos: BlockPosition) -> Result<bool, ChunkAccessError> {
        let block: u8 = unsafe { self.world.block(pos)? };
        if !definition(block).is_visible() {
            return Ok(false); // block is not exposed if it is not visible
        }

        let is_exposed: bool = SizedWorld::block_offsets(pos).any(|adj_pos| unsafe {
            match self.world.block(adj_pos) {
                Ok(adj_block) => !definition(adj_block).is_visible(),
                Err(ChunkAccessError::ChunkUnloaded) => false,
            }
        });

        Ok(is_exposed)
    }

    // fn raycast_coords(pos: Position2D) -> impl Iterator<Item = BlockPosition> {
    //     let mut positions: Vec<BlockPosition> = (0..screen_height() as i32)
    //         .rev()
    //         .map(move |z| {
    //             let screen_pos: Position3D = Position3D::new(pos.x, pos.y + (z as f32), z as f32);

    //             PROJECTION.screen_to_world(screen_pos)
    //         })
    //         .collect();

    //     positions.dedup();
    //     positions.into_iter()
    // }

    // fn raycast_coords(pos: Vec2, screen_height: u32) -> impl Iterator<Item = BlockPosition> {
    //     (0..screen_height as i32)
    //         .rev()
    //         .map(move |z| {
    //             let screen_pos: Vec3 = Vec3::new(pos.x, pos.y + z as f32, z as f32);
    //         })
    //         .collect::<HashSet<BlockPosition>>()
    //         .into_iter()
    // }
}
