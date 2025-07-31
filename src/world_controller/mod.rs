pub mod block_dictionary;
pub mod world;

mod block_generator;

use crate::config::{WorldGeneration, WorldMode};
use crate::world_controller::block_dictionary::definition;
use crate::world_controller::world::*;
use bevy::prelude::Resource;
use block_generator::{BlockGenerator, FlatGenerator, NormalGenerator, SkyblockGenerator};
use std::sync::Arc;
use terrain_data::prelude::*;

#[derive(Resource)]
pub struct WorldController {
    pub world: Arc<World>,
    pub generator: Box<dyn BlockGenerator>,
}

impl WorldController {
    pub fn new(params: &WorldGeneration) -> Self {
        Self {
            world: Arc::new(World::default()),
            generator: match params.world_mode {
                WorldMode::Normal => Box::new(NormalGenerator::new(params)),
                WorldMode::Flat => Box::new(FlatGenerator),
                WorldMode::Skyblock => Box::new(SkyblockGenerator),
            },
        }
    }

    pub fn chunk_render_data(
        &self,
        chunk_pos: ChunkPosition,
    ) -> Result<impl Iterator<Item = (u8, BlockPosition)>, ChunkAccessError> {
        let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

        let chunk = Arc::new(self.world.chunk(chunk_pos)?);
        let chunk_clone1 = Arc::clone(&chunk);

        Ok(World::chunk_coords(ChunkPosition::ZERO)
            .filter(move |&pos| chunk.is_exposed(pos).unwrap_or(false))
            .map(move |pos| {
                let block: u8 = chunk_clone1.block(pos).unwrap_or(0);
                let global_pos: BlockPosition = origin_block_pos + pos;
                (block, global_pos)
            }))
    }
}

pub async fn make_chunk(
    world: Arc<World>,
    generator: Box<dyn BlockGenerator>,
    chunk_pos: ChunkPosition,
    params: WorldGeneration,
) -> Result<Option<ChunkPosition>, AccessError> {
    if world.add_empty_chunk(chunk_pos).is_err() {
        return Ok(None); // return if chunk already exists
    }

    let mut chunk = world.chunk_mut(chunk_pos)?;
    let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

    for pos in World::chunk_coords(ChunkPosition::ZERO) {
        let block: u8 = generator.choose_block(origin_block_pos + pos, &params);
        chunk.value_mut().set_block(pos, block)?;
    }

    drop(chunk);

    update_exposed_blocks(world, chunk_pos).await?;

    Ok(Some(chunk_pos))
}

#[must_use]
async fn update_exposed_blocks(
    world: Arc<World>,
    chunk_pos: ChunkPosition,
) -> Result<(), AccessError> {
    let chunk = world.chunk(chunk_pos)?;
    let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);
    let mut exposed_set = Vec::new();

    World::chunk_coords(ChunkPosition::ZERO)
        .filter(|&pos| {
            let block: u8 = chunk.block(pos).unwrap_or(0);
            definition(block as usize).is_visible()
        })
        .for_each(|pos| {
            let is_exposed: bool =
                World::block_offsets(pos).any(|adj_pos| match chunk.block(adj_pos) {
                    Ok(adj_block) => !definition(adj_block as usize).is_visible(),
                    Err(_) => match world.block(origin_block_pos + adj_pos) {
                        Ok(adj_block) => !definition(adj_block as usize).is_visible(),
                        Err(_) => false,
                    },
                });

            exposed_set.push((pos, is_exposed));
        });

    drop(chunk);

    let mut chunk_mut = world.chunk_mut(chunk_pos)?;

    for (pos, is_exposed) in exposed_set {
        chunk_mut.set_is_exposed(pos, is_exposed)?;
    }

    Ok(())
}
