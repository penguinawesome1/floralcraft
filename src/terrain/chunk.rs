use serde::{ Serialize, Deserialize };
use itertools::iproduct;
use crate::terrain::subchunk::{ SUBCHUNK_WIDTH, SUBCHUNK_HEIGHT, SUBCHUNK_DEPTH };
use crate::terrain::subchunk::Subchunk;
use crate::terrain::{ BlockPosition, Block };

const SUBCHUNKS_IN_CHUNK: usize = 4;

pub const CHUNK_WIDTH: usize = SUBCHUNK_WIDTH;
pub const CHUNK_HEIGHT: usize = SUBCHUNK_HEIGHT;
pub const CHUNK_DEPTH: usize = SUBCHUNK_DEPTH * SUBCHUNKS_IN_CHUNK;
pub const CHUNK_VOLUME: usize = CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH;

pub const BLOCK_OFFSETS: [BlockPosition; 6] = [
    BlockPosition::new(1, 0, 0),
    BlockPosition::new(0, 1, 0),
    BlockPosition::new(0, 0, 1),
    BlockPosition::new(-1, 0, 0),
    BlockPosition::new(0, -1, 0),
    BlockPosition::new(0, 0, -1),
];

/// Stores the two dimensional integer position of a chunk.
pub type ChunkPosition = glam::IVec2;

macro_rules! impl_getter {
    (
        $(#[$meta:meta])*
        $name:ident,
        $return_type:ty,
        $sub_method:ident,
        $default:expr
    ) => {
        $(#[$meta])*
        pub fn $name(&self, pos: BlockPosition) -> $return_type {
            if let Some(subchunk) = self.subchunk(pos.z) {
                let sub_pos: BlockPosition = Self::local_to_sub(pos);
                subchunk.$sub_method(sub_pos)
            } else {
                $default
            }
        }
    };
}

macro_rules! impl_setter {
    (
        $(#[$meta:meta])*
        $name:ident,
        $value_type:ty,
        $sub_method:ident,
        $default:expr
    ) => {
        $(#[$meta])*
        pub fn $name(&mut self, pos: BlockPosition, value: $value_type) {
            let index: usize = Self::subchunk_index(pos.z);
            let subchunk_opt: &mut Option<Subchunk> = &mut self.subchunks[index];

            if value == $default && subchunk_opt.is_none() {
                return; // return if placement is redundant
            }

            let subchunk: &mut Subchunk = subchunk_opt.get_or_insert_with(|| Subchunk::new());
            let sub_pos: BlockPosition = Self::local_to_sub(pos);

            subchunk.$sub_method(sub_pos, value);

            if subchunk.is_empty() {
                *subchunk_opt = None; // set empty subchunks to none
            }
        }
    };
}

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub pos: ChunkPosition,
    subchunks: [Option<Subchunk>; SUBCHUNKS_IN_CHUNK],
}

impl Chunk {
    /// Create a new zeroed out chunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::chunk::{ Chunk, ChunkPosition };
    ///
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    /// let chunk: Chunk = Chunk::new(pos);
    /// ```
    pub fn new(pos: ChunkPosition) -> Self {
        Self {
            pos,
            subchunks: std::array::from_fn(|_| None),
        }
    }

    impl_getter!(
        /// Gets the block given its local position.
        ///
        /// # Examples
        ///
        /// ```
        /// use floralcraft::terrain::chunk::{ Chunk, ChunkPosition };
        /// use floralcraft::terrain::block::{ Block, BlockPosition };
        ///
        /// let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
        /// let mut chunk: Chunk = Chunk::new(chunk_pos);
        /// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
        ///
        /// chunk.set_block(pos, Block::Dirt);
        /// let block: Block = chunk.block(pos);
        ///
        /// assert_eq!(block, Block::Dirt);
        /// ```
        block,
        Block,
        block,
        Block::Air
    );

    impl_getter!(
        /// Gets skylight given its local position.
        sky_light,
        u8,
        sky_light,
        0
    );

    impl_getter!(
        /// Gets block light given its local position.
        block_light,
        u8,
        block_light,
        0
    );

    impl_getter!(
        /// Gets if block is exposed given its local position.
        block_exposed,
        bool,
        block_exposed,
        false
    );

    impl_setter!(
        /// Sets block given its local position.
        set_block,
        Block,
        set_block,
        Block::Air
    );

    impl_setter!(
        /// Sets skylight given its local position.
        set_sky_light,
        u8,
        set_sky_light,
        0
    );

    impl_setter!(
        /// Sets block light given its local position.
        set_block_light,
        u8,
        set_block_light,
        0
    );

    impl_setter!(
        /// Sets if block is exposed given its local position.
        set_block_exposed,
        bool,
        set_block_exposed,
        false
    );

    /// Returns a bool for if all subchunks are empty.
    pub fn is_empty(&self) -> bool {
        self.subchunks.iter().all(|subchunk| subchunk.is_none())
    }

    /// Returns an iterator for all block positions.
    pub fn chunk_coords() -> impl Iterator<Item = BlockPosition> {
        iproduct!(0..CHUNK_WIDTH as i32, 0..CHUNK_HEIGHT as i32, 0..CHUNK_DEPTH as i32).map(
            |(x, y, z)| BlockPosition::new(x, y, z)
        )
    }

    /// Returns all adjacent block offsets.
    /// Filters out illegal vertical offsets.
    pub fn block_offsets(pos: BlockPosition) -> impl Iterator<Item = BlockPosition> {
        BLOCK_OFFSETS.iter()
            .map(move |offset| { pos + offset })
            .filter(|adj_pos| { adj_pos.z >= 0 && adj_pos.z < (CHUNK_DEPTH as i32) })
    }

    const fn subchunk(&self, pos_z: i32) -> Option<&Subchunk> {
        let index: usize = Self::subchunk_index(pos_z);
        self.subchunks[index].as_ref()
    }

    const fn subchunk_index(pos_z: i32) -> usize {
        (pos_z as usize).div_euclid(SUBCHUNK_DEPTH)
    }

    const fn local_to_sub(pos: BlockPosition) -> BlockPosition {
        BlockPosition::new(pos.x, pos.y, pos.z.rem_euclid(SUBCHUNK_DEPTH as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec3;

    #[test]
    fn test_new_is_empty() {
        let pos: ChunkPosition = ChunkPosition::new(0, 0);
        let chunk: Chunk = Chunk::new(pos);
        assert!(chunk.is_empty());
    }

    #[test]
    fn test_set_and_get_block() {
        let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
        let mut chunk: Chunk = Chunk::new(chunk_pos);
        let pos_1: IVec3 = IVec3::new(15, 1, 21);
        let pos_2: IVec3 = IVec3::new(3, 0, 2);

        chunk.set_block(pos_1, Block::Dirt);
        chunk.set_block(pos_1, Block::Grass);
        chunk.set_block(pos_2, Block::Air);

        assert_eq!(chunk.block(pos_1), Block::Grass);
        assert_eq!(chunk.block(pos_2), Block::Air);
    }
}
