pub mod block_dictionary;
pub mod block_generator;
pub mod chunk_generation;

use bevy::prelude::Resource;
use std::collections::HashSet;
use std::sync::Arc;
use terrain_data::prelude::{ChunkPosition, world};

#[derive(Resource)]
pub struct ResWorld(pub Arc<World>);

#[derive(Default, Resource)]
pub struct DirtyChunks(pub HashSet<ChunkPosition>);

world! {
    chunk_width: 16,
    chunk_height: 16,
    subchunk_depth: 16,
    num_subchunks: 16,
    Block r#as block: u8 = 4,
    BlockLight r#as block_light: u8 = 4,
    SkyLight r#as sky_light: u8 = 4,
    Exposed r#as is_exposed: bool = 1,
}
