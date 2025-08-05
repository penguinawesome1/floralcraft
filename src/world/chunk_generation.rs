use crate::config;
use crate::config::Config;
use crate::config::WorldGeneration;
use crate::world::{
    DirtyChunks, ResWorld, World,
    block_dictionary::{SnugType, definition},
    block_generator::BlockGenerator,
    chunk_generation::future::block_on,
};
use anyhow::{Result, anyhow};
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::collections::HashSet;
use std::sync::Arc;
use terrain_data::prelude::*;

#[derive(Resource)]
pub struct ResGenerator(pub Box<dyn BlockGenerator>);

#[derive(Default, Resource)]
pub struct ChunksBeingGenerated(pub HashSet<ChunkPosition>);

#[derive(Component)]
pub struct ChunkGenerationTask(pub Task<Result<ChunkPosition>>);

pub fn make_chunk_tasks(
    mut commands: Commands,
    world: Res<ResWorld>,
    generator: Res<ResGenerator>,
    config: Res<Config>,
    mut chunks_being_generated: ResMut<ChunksBeingGenerated>,
) {
    let compute_pool = AsyncComputeTaskPool::get();

    let params: &config::WorldGeneration = &config.world.generation;
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = config.world.render_distance;
    let positions = World::positions_in_square(origin, radius);

    const MAX_TASKS_PER_FRAME: usize = 1;
    let mut success_num: usize = 0;

    for chunk_pos in positions {
        if success_num >= MAX_TASKS_PER_FRAME {
            break;
        }

        if world.0.is_chunk_at_pos(chunk_pos) || chunks_being_generated.0.contains(&chunk_pos) {
            continue;
        }

        let world_clone: Arc<World> = Arc::clone(&world.0);
        let generator_clone: Box<dyn BlockGenerator> = generator.0.clone_box();
        let params_clone: WorldGeneration = params.clone();

        let task = compute_pool.spawn(async move {
            make_chunk(&world_clone, generator_clone, chunk_pos, params_clone)
        });

        commands.spawn(ChunkGenerationTask(task));
        chunks_being_generated.0.insert(chunk_pos);
        success_num += 1;
    }
}

pub fn handle_chunk_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkGenerationTask)>,
    mut chunks_being_generated: ResMut<ChunksBeingGenerated>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    for (entity, mut task) in &mut query {
        let Some(result) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        match result {
            Ok(chunk_pos) => {
                chunks_being_generated.0.remove(&chunk_pos);
                dirty_chunks.0.insert(chunk_pos);
            }
            Err(e) => eprintln!("Error generating chunk: {:?}", e),
        }

        commands.entity(entity).despawn();
    }
}

fn make_chunk(
    world: &World,
    generator: Box<dyn BlockGenerator>,
    chunk_pos: ChunkPosition,
    params: WorldGeneration,
) -> Result<ChunkPosition> {
    if world.add_empty_chunk(chunk_pos).is_err() {
        return Err(anyhow!("Chunk {:?} already exists", chunk_pos));
    }

    let mut chunk = world.chunk_mut(chunk_pos)?;
    let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

    // choose all block types in the chunk
    for pos in World::chunk_coords(ChunkPosition::ZERO) {
        let block: SnugType = generator.choose_block(origin_block_pos + pos, &params);
        chunk.value_mut().set_block(pos, block)?;
    }

    // update which blocks are exposed
    for pos in World::chunk_coords(ChunkPosition::ZERO) {
        let block: SnugType = chunk.block(pos).unwrap_or(0);
        if !definition(block as usize).is_visible() {
            chunk.set_is_exposed(pos, false)?;
            continue;
        }

        let is_exposed: bool =
            World::block_offsets(pos).any(|adj_pos| match chunk.block(adj_pos) {
                Ok(adj_block) => !definition(adj_block as usize).is_visible(),
                _ => false,
            });

        chunk.set_is_exposed(pos, is_exposed)?;
    }

    Ok(chunk_pos)
}
