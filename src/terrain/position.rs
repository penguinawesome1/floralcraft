use glam::{ IVec3, IVec2 };
use crate::terrain::chunk::{ CHUNK_WIDTH, CHUNK_HEIGHT };

pub type BlockPosition = IVec3;
pub type ChunkPosition = IVec2;

pub struct Conversion;

impl Conversion {
    /// Converts a given chunk position to its zero corner block position.
    pub const fn chunk_to_block_pos(pos: ChunkPosition) -> BlockPosition {
        BlockPosition::new(pos.x * (CHUNK_WIDTH as i32), pos.y * (CHUNK_HEIGHT as i32), 0)
    }

    /// Gets the chunk position a block position falls into.
    pub const fn block_to_chunk_pos(pos: BlockPosition) -> ChunkPosition {
        ChunkPosition::new(
            pos.x.div_euclid(CHUNK_WIDTH as i32),
            pos.y.div_euclid(CHUNK_HEIGHT as i32)
        )
    }

    /// Finds the remainder of a global position using chunk size.
    pub const fn global_to_local_pos(pos: BlockPosition) -> BlockPosition {
        BlockPosition::new(
            pos.x.rem_euclid(CHUNK_WIDTH as i32),
            pos.y.rem_euclid(CHUNK_HEIGHT as i32),
            pos.z
        )
    }
}
