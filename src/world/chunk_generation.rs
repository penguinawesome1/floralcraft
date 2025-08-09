use crate::config;
use crate::config::Config;
use crate::config::WorldGeneration;
use crate::config::WorldMode;
use crate::renderer::ChunksToRender;
use crate::world::Chunk;
use crate::world::{
    ResWorld, World,
    block_dictionary::{SnugType, definition},
    block_generator::{BlockGenerator, FlatGenerator, NormalGenerator, SkyblockGenerator},
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

#[derive(Resource, Deref, DerefMut, Default)]
pub struct ChunkTaskPool(pub VecDeque<AsyncReceiver<(ChunkPosition, Chunk)>>);

#[derive(Resource, Default)]
pub struct ChunksStillGenerating(pub HashSet<ChunkPosition>);

#[derive(Resource, Default)]
pub struct ChunksToGenerate(pub Vec<ChunkPosition>);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GenerationSet;

pub struct GenerationPlugin;

impl Plugin for GenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_generator_resources.in_set(GenerationSet))
            .add_systems(Update, (make_chunk_tasks, handle_chunk_tasks).chain());
    }
}

fn setup_generator_resources(mut commands: Commands, config: Res<Config>) {
    let generator: Box<dyn BlockGenerator> = match &config.world.generation.world_mode {
        WorldMode::Normal => Box::new(NormalGenerator::new(&config.world.generation)),
        WorldMode::Flat => Box::new(FlatGenerator),
        WorldMode::Skyblock => Box::new(SkyblockGenerator),
    };

    commands.insert_resource(ResGenerator(generator));
    commands.insert_resource(ChunkTaskPool::default());
    commands.insert_resource(ChunksStillGenerating::default());
    commands.insert_resource(ChunksToGenerate::default());
}

fn make_chunk_tasks(
    mut chunk_task_pool: ResMut<'_, ChunkTaskPool>,
    mut chunks_to_generate: ResMut<ChunksToGenerate>,
    mut chunks_still_generating: ResMut<ChunksStillGenerating>,
    generator: Res<ResGenerator>,
    config: Res<Config>,
) {
    let params: &config::WorldGeneration = &config.world.generation;

    for chunk_pos in chunks_to_generate.0.drain(..) {
        let generator_clone: Box<dyn BlockGenerator> = generator.0.clone_box();
        let params_clone: WorldGeneration = params.clone();

        let (fut, receiver) =
            AsyncTask::new(make_chunk(generator_clone, chunk_pos, params_clone)).split();

        chunk_task_pool.push_back(receiver);
        AsyncComputeTaskPool::get().spawn(fut).detach();
        chunks_still_generating.0.insert(chunk_pos);
    }
}

fn handle_chunk_tasks(
    mut chunk_task_pool: ResMut<'_, ChunkTaskPool>,
    mut chunks_to_render: ResMut<ChunksToRender>,
    mut chunks_still_generating: ResMut<ChunksStillGenerating>,
    world: Res<ResWorld>,
) {
    for _ in 0..MAX_TASKS_PER_FRAME {
        let Some(mut receiver) = chunk_task_pool.0.pop_front() else {
            return;
        };

        let Some((chunk_pos, chunk)) = receiver.try_recv() else {
            chunk_task_pool.0.push_back(receiver);
            continue;
        };

        match world.0.add_chunk(chunk_pos, Some(chunk)) {
            Ok(()) => {
                chunks_to_render.0.push(chunk_pos);
                chunks_still_generating.0.remove(&chunk_pos);
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
        let block: SnugType = chunk.block(pos).unwrap();

        let is_exposed: bool = definition(block as usize).is_visible()
            && World::block_offsets(pos).any(|adj_pos| match chunk.block(adj_pos) {
                Ok(adj_block) => definition(adj_block as usize).is_transparent(),
                _ => false,
            });

        chunk.set_is_exposed(pos, is_exposed).unwrap();
    }

    (chunk_pos, chunk)
}
