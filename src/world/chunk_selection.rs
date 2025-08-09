use crate::{
    config::Config,
    player::PlayerWorldPos,
    world::{
        ResWorld, World,
        chunk_generation::{ChunksStillGenerating, ChunksToGenerate},
    },
};
use bevy::prelude::*;
use terrain_data::prelude::ChunkPosition;

pub fn choose_chunks_to_generate(
    mut chunks_to_generate: ResMut<ChunksToGenerate>,
    player_world_pos: Res<PlayerWorldPos>,
    world: Res<ResWorld>,
    config: Res<Config>,
    chunks_still_generating: Res<ChunksStillGenerating>,
) {
    let origin: ChunkPosition = World::block_to_chunk_pos(player_world_pos.0.as_ivec3());
    let radius: u32 = config.world.render_distance;
    let positions = World::positions_in_square(origin, radius)
        .filter(|&pos| !world.0.is_chunk_at_pos(pos) && !chunks_still_generating.0.contains(&pos));

    chunks_to_generate.0.extend(positions);
}
