pub mod chunk_loader;
pub mod dictionary;

use crate::world::dictionary::BlockType;
use aether::prelude::*;
use bevy::prelude::Resource;

pub const SUBCHUNK_D: usize = 16;
pub const CHUNK_W: usize = 16;
pub const CHUNK_H: usize = 16;
pub const NUM_SUBCHUNKS: usize = 16;

pub const CHUNK_D: usize = SUBCHUNK_D * NUM_SUBCHUNKS;
pub const SUBCHUNK_V: usize = CHUNK_W * CHUNK_H * SUBCHUNK_D;
pub const CHUNK_V: usize = SUBCHUNK_V * NUM_SUBCHUNKS;

pub type MySection = Section<BlockType, CHUNK_W, CHUNK_H, SUBCHUNK_D>;

world! {
    #[derive(Resource)]
    ==
    [CHUNK_W, CHUNK_H, SUBCHUNK_D; NUM_SUBCHUNKS],
    block: u8,
    sky_light: u8,
    is_exposed: bool,
}
