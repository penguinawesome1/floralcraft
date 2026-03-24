use crate::GameState;
use crate::config::{Config, ConfigSet, WorldMode};
use crate::player::PlayerMovedFilter;
use crate::position::GridPosition;
use crate::world::dictionary::ENTRIES;
use crate::world::{Chunk, ChunkPos, World as AeWorld};
use bevy::prelude::*;
use lattice::{BlockGen, BlockGenParams, Blocks, FlatGen};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AeWorld>();
        app.add_systems(
            Startup,
            (setup_generator_resources, load_chunks)
                .chain()
                .after(ConfigSet),
        );
        app.add_systems(Update, load_chunks.run_if(in_state(GameState::Playing)));
    }
}

struct MyBlocks;

impl Blocks for MyBlocks {
    type T = u8;
    const AIR: Self::T = 0;
    const GRASS: Self::T = 1;
    const DIRT: Self::T = 2;
    const STONE: Self::T = 3;
    const ROSE: Self::T = 4;
    const DANDELION: Self::T = 5;
    const BEDROCK: Self::T = 6;
}

#[derive(Resource)]
pub struct BlockGenRes(Box<dyn BlockGen<MyBlocks>>);

fn setup_generator_resources(mut commands: Commands, config: Res<Config>) {
    let world_mode = &config.world.mode;

    let generator: Box<dyn BlockGen<MyBlocks>> = match world_mode {
        WorldMode::Flat => Box::new(FlatGen::<MyBlocks>::new()),
        _ => panic!(),
    };

    commands.insert_resource(BlockGenRes(generator));
}

fn load_chunks(
    world: Res<AeWorld>,
    config: Res<Config>,
    block_gen: Res<BlockGenRes>,
    query: Query<&GridPosition, PlayerMovedFilter>,
) {
    let radius = config.world.render_distance;

    query
        .iter()
        .map(|player_pos| AeWorld::to_chunk(player_pos.0.as_ivec3()))
        .flat_map(|chunk_pos| AeWorld::square_around(chunk_pos, radius))
        .for_each(|chunk_pos| load_chunk(&world, chunk_pos, &block_gen));
}

fn load_chunk(world: &AeWorld, chunk_pos: ChunkPos, block_gen: &BlockGenRes) {
    let mut chunk = Chunk::default();
    let params = BlockGenParams;
    let origin_block_pos = AeWorld::to_block(chunk_pos);

    for pos in AeWorld::blocks_in(ChunkPos::ZERO) {
        let block = block_gen.0.choose_block(origin_block_pos + pos, &params);
        chunk.set_block(AeWorld::to_local(pos), block).unwrap();
    }

    for pos in AeWorld::blocks_in(ChunkPos::ZERO) {
        let block = chunk.block(pos).unwrap();

        let is_exposed = ENTRIES[block as usize].is_visible()
            && AeWorld::block_neighbors(pos)
                .filter_map(|adj_pos| chunk.block(adj_pos))
                .any(|adj_block| ENTRIES[adj_block as usize].is_transparent());

        chunk.set_is_exposed(pos, is_exposed).unwrap();
    }

    _ = world.insert(chunk_pos, Some(chunk))
}
