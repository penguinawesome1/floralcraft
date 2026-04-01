use crate::GameState;
use crate::config::{Config, ConfigSet, WorldMode};
use crate::world::{
    Chunk, ChunkPos, MySection, SUBCHUNK_D, SUBCHUNK_V, World as AeWorld,
    dictionary::{BlockType, ENTRIES},
};
use bevy::prelude::*;
use bevy::tasks::block_on;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use lattice::{BlockGen, Blocks, FlatGen, NormalGen};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AeWorld>();
        app.init_resource::<ChunksLoading>();
        app.add_systems(Startup, setup_generator_resources.after(ConfigSet));
        app.add_systems(
            Update,
            (spawn_chunk_tasks, handle_chunk_tasks).run_if(in_state(GameState::Playing)),
        );
    }
}

struct MyBlocks;

impl Blocks for MyBlocks {
    type T = BlockType;
    const AIR: Self::T = 0;
    const GRASS: Self::T = 1;
    const DIRT: Self::T = 2;
    const STONE: Self::T = 3;
    const ROSE: Self::T = 4;
    const DANDELION: Self::T = 5;
    const BEDROCK: Self::T = 6;
}

#[derive(Resource, Clone)]
pub struct BlockGenRes(Arc<dyn BlockGen<MyBlocks, SUBCHUNK_D, SUBCHUNK_V>>);

#[derive(Component)]
struct ChunkTask(Task<(ChunkPos, Chunk)>);

#[derive(Resource, Default)]
struct ChunksLoading(HashSet<ChunkPos>);

fn setup_generator_resources(mut commands: Commands, config: Res<Config>) {
    let world_mode = &config.world.mode;
    let params = (&config.world.terrain).into();

    let generator: Arc<dyn BlockGen<MyBlocks, SUBCHUNK_D, SUBCHUNK_V>> = match world_mode {
        WorldMode::Flat => Arc::new(FlatGen::<MyBlocks>::default()),
        WorldMode::Normal => Arc::new(NormalGen::<MyBlocks>::new(
            &config.world.terrain.noise_profile,
            params,
        )),
        _ => panic!(),
    };

    commands.insert_resource(BlockGenRes(generator));
}

fn spawn_chunk_tasks(
    mut commands: Commands,
    world: Res<AeWorld>,
    block_gen: Res<BlockGenRes>,
    config: Res<Config>,
    mut chunks_loading: ResMut<ChunksLoading>,
) {
    let player_chunk_pos = ChunkPos::new(0, 0);
    let task_pool = AsyncComputeTaskPool::get();
    let chunk_positions: Vec<ChunkPos> =
        AeWorld::square_around(player_chunk_pos, config.world.render_distance)
            .filter(|pos| !world.contains(pos) && !chunks_loading.0.contains(pos))
            .collect();

    for pos in chunk_positions {
        chunks_loading.0.insert(pos);

        let gen_handle = block_gen.clone();

        let task = task_pool.spawn(async move {
            let chunk = gen_chunk(pos, gen_handle);
            (pos, chunk)
        });

        commands.spawn(ChunkTask(task));
    }
}

fn handle_chunk_tasks(
    mut commands: Commands,
    world: Res<AeWorld>,
    mut chunks_loading: ResMut<ChunksLoading>,
    mut tasks: Query<(Entity, &mut ChunkTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some((pos, chunk)) = block_on(future::poll_once(&mut task.0)) {
            world.insert(&pos, Some(chunk)).unwrap();
            chunks_loading.0.remove(&pos);
            commands.entity(entity).despawn();
        }
    }
}

fn gen_chunk(chunk_pos: ChunkPos, block_gen: BlockGenRes) -> Chunk {
    let mut chunk = Chunk::default();
    let origin_block_pos = AeWorld::to_block(&chunk_pos);
    let mut out = Box::new([BlockType::default(); SUBCHUNK_V]);

    for (i, subchunk) in chunk.iter_mut().enumerate() {
        let start_pos = origin_block_pos + ivec3(0, 0, (i * SUBCHUNK_D) as i32);
        block_gen.0.choose_blocks(start_pos, &mut out);
        subchunk.block = Some(MySection::from_data(out.as_slice()));
    }

    for pos in AeWorld::blocks_in(ChunkPos::ZERO) {
        let block = chunk.block(pos).unwrap();

        let is_exposed = ENTRIES[block as usize].is_visible()
            && AeWorld::block_neighbors(&pos)
                .filter_map(|adj_pos| chunk.block(adj_pos))
                .any(|adj_block| ENTRIES[adj_block as usize].is_transparent());

        chunk.set_is_exposed(pos, is_exposed).unwrap();
    }

    chunk
}
