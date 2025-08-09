pub mod block_dictionary;
pub mod block_generator;
pub mod chunk_generation;
pub mod chunk_selection;
pub mod hover_block;
pub mod interaction;

use bevy::prelude::Resource;
use std::sync::Arc;
use terrain_data::prelude::world;

#[derive(Resource)]
pub struct ResWorld(pub Arc<World>);

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
