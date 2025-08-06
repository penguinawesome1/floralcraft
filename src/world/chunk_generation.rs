use crate::config;
use crate::config::Config;
use crate::config::WorldGeneration;
use crate::world::Chunk;
use crate::world::{
    DirtyChunks, ResWorld, World,
    block_dictionary::{SnugType, definition},
    block_generator::BlockGenerator,
};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_async_task::AsyncReceiver;
use bevy_async_task::AsyncTask;
use std::collections::HashSet;
use std::collections::VecDeque;
use terrain_data::prelude::*;

const MAX_TASKS_PER_FRAME: usize = 5;

#[derive(Resource)]
pub struct ResGenerator(pub Box<dyn BlockGenerator>);

#[derive(Resource, Default)]
pub struct PendingChunks(pub Vec<ChunkPosition>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ChunkTaskPool(pub VecDeque<AsyncReceiver<(ChunkPosition, Chunk)>>);

#[derive(Resource, Default)]
pub struct ChunksTasksInTransit(pub HashSet<ChunkPosition>);

pub fn make_chunk_tasks(
    mut chunk_task_pool: ResMut<'_, ChunkTaskPool>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut transit_tasks: ResMut<ChunksTasksInTransit>,
    world: Res<ResWorld>,
    generator: Res<ResGenerator>,
    config: Res<Config>,
) {
    let params: &config::WorldGeneration = &config.world.generation;

    for _ in 0..MAX_TASKS_PER_FRAME {
        let chunk_pos: ChunkPosition = loop {
            let Some(chunk_pos) = pending_chunks.0.pop() else {
                return;
            };

            if !world.0.is_chunk_at_pos(chunk_pos) && !transit_tasks.0.contains(&chunk_pos) {
                break chunk_pos;
            }
        };

        let generator_clone: Box<dyn BlockGenerator> = generator.0.clone_box();
        let params_clone: WorldGeneration = params.clone();

        let (fut, receiver) =
            AsyncTask::new(make_chunk(generator_clone, chunk_pos, params_clone)).split();

        chunk_task_pool.push_back(receiver);
        AsyncComputeTaskPool::get().spawn(fut).detach();
        transit_tasks.0.insert(chunk_pos);
    }
}

pub fn handle_chunk_tasks(
    mut chunk_task_pool: ResMut<'_, ChunkTaskPool>,
    mut dirty_chunks: ResMut<DirtyChunks>,
    mut transit_tasks: ResMut<ChunksTasksInTransit>,
    world: Res<ResWorld>,
) {
    for _ in 0..MAX_TASKS_PER_FRAME {
        let Some(mut receiver) = chunk_task_pool.0.pop_front() else {
            continue;
        };

        let Some((chunk_pos, chunk)) = receiver.try_recv() else {
            chunk_task_pool.0.push_back(receiver);
            continue;
        };

        match world.0.add_chunk(chunk_pos, Some(chunk)) {
            Ok(()) => {
                dirty_chunks.0.insert(chunk_pos);
                transit_tasks.0.remove(&chunk_pos);
            }
            Err(e) => eprintln!("Error setting chunk: {}", e),
        }
    }
}

async fn make_chunk(
    generator: Box<dyn BlockGenerator>,
    chunk_pos: ChunkPosition,
    params: WorldGeneration,
) -> (ChunkPosition, Chunk) {
    let mut chunk: Chunk = Chunk::default();
    let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

    // choose all block types in the chunk
    for pos in World::chunk_coords(ChunkPosition::ZERO) {
        let block: SnugType = generator.choose_block(origin_block_pos + pos, &params);
        chunk.set_block(pos, block).unwrap();
    }

    // update which blocks are exposed
    for pos in World::chunk_coords(ChunkPosition::ZERO) {
        let block: SnugType = chunk.block(pos).unwrap_or(0);
        if !definition(block as usize).is_visible() {
            chunk.set_is_exposed(pos, false).unwrap();
            continue;
        }

        let is_exposed: bool =
            World::block_offsets(pos).any(|adj_pos| match chunk.block(adj_pos) {
                Ok(adj_block) => !definition(adj_block as usize).is_visible(),
                _ => false,
            });

        chunk.set_is_exposed(pos, is_exposed).unwrap();
    }

    (chunk_pos, chunk)
}
