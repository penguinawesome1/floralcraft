use crate::terrain::Block;
use crate::terrain::{ BlockPosition, ChunkPosition };

pub const CHUNK_WIDTH: u32 = 16;
pub const CHUNK_HEIGHT: u32 = 16;
pub const CHUNK_DEPTH: u32 = 16;
pub const CHUNK_VOLUME: usize = (CHUNK_WIDTH * CHUNK_HEIGHT * CHUNK_DEPTH) as usize;

/// Used to send info from chunk module to renderer.
/// Contains the block name and its screen position.
pub struct BlockRenderData {
    pub block_name: Block,
    pub world_pos: BlockPosition,
    pub is_target: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pos: ChunkPosition,
    blocks: [Block; CHUNK_VOLUME],

    // using u8 for 4-bit packed values (0-15), 2 values per byte
    // this halves the memory compared to storing u8 per value
    sky_light: [u8; CHUNK_VOLUME / 2],
    block_light: [u8; CHUNK_VOLUME / 2],
    exposed_blocks: [u64; CHUNK_VOLUME / 64],
}

impl Chunk {
    /// Create a new empty chunk (filled with air blocks) at a given pos.
    /// World generation logic should then fill this chunk with appropriate blocks.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::{ Block, Block };
    /// use floralcraft::terrain::chunk::Chunk;
    /// use floralcraft::terrain::{ ChunkPosition, BlockPosition };
    ///
    /// let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
    /// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
    /// let chunk: Chunk = Chunk::new(chunk_pos);
    /// let is_block_exposed: Option<bool> = chunk.is_block_exposed(pos);
    ///
    /// if let Some(block) = chunk.get_block_name(pos) {
    ///     assert_eq!(block, Block::Air);
    ///     assert!(!is_block_exposed.unwrap());
    /// } else {
    ///     panic!();
    /// }
    /// ```
    pub const fn new(pos: ChunkPosition) -> Self {
        Self {
            pos,
            blocks: [Block::Air; CHUNK_VOLUME],
            sky_light: [0; CHUNK_VOLUME / 2],
            block_light: [0; CHUNK_VOLUME / 2],
            exposed_blocks: [0; CHUNK_VOLUME / 64],
        }
    }

    // returns chunk array index given local position
    const fn get_index(pos: BlockPosition) -> Option<usize> {
        if
            pos.x < 0 ||
            pos.x >= (CHUNK_WIDTH as i32) ||
            pos.y < 0 ||
            pos.y >= (CHUNK_HEIGHT as i32) ||
            pos.z < 0 ||
            pos.z >= (CHUNK_DEPTH as i32)
        {
            return None;
        }

        Some(
            (pos.x as usize) * ((CHUNK_HEIGHT * CHUNK_DEPTH) as usize) +
                (pos.y as usize) * (CHUNK_DEPTH as usize) +
                (pos.z as usize)
        )
    }

    /// Gets an option for the name of the block at a given local position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::Block;
    /// use floralcraft::terrain::chunk::Chunk;
    /// use floralcraft::terrain::{ BlockPosition, ChunkPosition };
    ///
    /// let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
    /// let chunk: Chunk = Chunk::new(chunk_pos);
    ///
    /// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
    /// let name: Option<Block> = chunk.get_block_name(pos);
    ///
    /// if let Some(name) = chunk.get_block_name(pos) {
    ///     assert_eq!(name, Block::Air);
    /// } else {
    ///     panic!();
    /// }
    /// ```
    pub fn get_block_name(&self, pos: BlockPosition) -> Option<Block> {
        let index: usize = Self::get_index(pos)?;
        Some(self.blocks[index])
    }

    /// Sets the name of a block at a given local position.
    /// Returns None if no index found.
    pub fn set_block_name(&mut self, pos: BlockPosition, block_name: Block) -> Option<()> {
        let index: usize = Self::get_index(pos)?;
        self.blocks[index] = block_name;
        Some(())
    }

    /// Gets an option for the mutable name of the block at a given local position.
    pub fn get_block_name_mut(&mut self, pos: BlockPosition) -> Option<&mut Block> {
        let index: usize = Self::get_index(pos)?;
        Some(&mut self.blocks[index])
    }

    /// Gets an option for the value of skylight at a given local position.
    pub fn get_sky_light(&self, pos: BlockPosition) -> Option<u8> {
        let index: usize = Self::get_index(pos)?;
        let byte_index: usize = index / 2;
        let byte: u8 = self.sky_light[byte_index];
        Some(if index % 2 == 0 { byte & 0xf } else { byte >> 4 })
    }

    /// Sets the value of skylight at a given local position.
    /// Returns None if no index found.
    /// Value must be 0-15.
    pub fn set_sky_light(&mut self, pos: BlockPosition, level: u8) -> Option<()> {
        let index: usize = Self::get_index(pos)?;
        let byte_index: usize = index / 2;
        let mut byte: u8 = self.sky_light[byte_index];

        let clamped_level: u8 = level & 0xf; // level must not exceed 4 bits

        if index % 2 == 0 {
            byte = (byte & 0xf0) | clamped_level; // lower 4 bits
        } else {
            byte = (byte & 0x0f) | (clamped_level << 4); // upper 4 bits
        }

        self.sky_light[byte_index] = byte;
        Some(())
    }

    /// Gets an option of value of block light at a given local position.
    pub fn get_block_light(&self, pos: BlockPosition) -> Option<u8> {
        let index: usize = Self::get_index(pos)?;
        let byte_index: usize = index / 2;
        let byte: u8 = self.block_light[byte_index];
        Some(if index % 2 == 0 { byte & 0xf } else { byte >> 4 })
    }

    /// Sets the value of block light at a given local position.
    /// Returns None if no index found.
    /// Value must be 0-15.
    pub fn set_block_light(&mut self, pos: BlockPosition, level: u8) -> Option<()> {
        let index: usize = Self::get_index(pos)?;
        let byte_index: usize = index / 2;
        let mut byte: u8 = self.block_light[byte_index];
        let clamped_level: u8 = level & 0xf; // level must not exceed 4 bits

        if index % 2 == 0 {
            byte = (byte & 0xf0) | clamped_level; // lower 4 bits
        } else {
            byte = (byte & 0x0f) | (clamped_level << 4); // upper 4 bits
        }

        self.block_light[byte_index] = byte;

        Some(())
    }

    /// Gets option of is exposed at a given local position.
    pub fn is_block_exposed(&self, pos: BlockPosition) -> Option<bool> {
        let index: usize = Self::get_index(pos)?;
        let array_index: usize = index / 64;
        let bit_index: usize = index % 64;
        Some((self.exposed_blocks[array_index] & (1 << bit_index)) != 0)
    }

    /// Sets is exposed at a given local position.
    /// Returns None if no index found.
    pub fn set_block_exposed(&mut self, pos: BlockPosition, is_exposed: bool) -> Option<()> {
        let index: usize = Self::get_index(pos)?;
        let array_index: usize = index / 64;
        let bit_index: usize = index % 64;

        if is_exposed {
            self.exposed_blocks[array_index] |= 1 << bit_index;
        } else {
            self.exposed_blocks[array_index] &= !(1 << bit_index);
        }

        Some(())
    }
}
