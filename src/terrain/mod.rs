pub mod block;
pub mod chunk;
pub mod section;
pub mod subchunk;

use std::collections::{ HashMap, HashSet };
use itertools::iproduct;
use crate::terrain::chunk::{ Chunk, ChunkPosition, CHUNK_WIDTH, CHUNK_HEIGHT };
use crate::terrain::block::{ Block, BlockPosition };

const CHUNK_ADJ_OFFSETS: [ChunkPosition; 4] = [
    ChunkPosition::new(-1, 0),
    ChunkPosition::new(1, 0),
    ChunkPosition::new(0, -1),
    ChunkPosition::new(0, 1),
];

/// Stores all chunks and marks dirty chunks.
/// Allows access and modification to them.
pub struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
    dirty_chunks: HashSet<ChunkPosition>,
}

impl World {
    /// Create a new collection of decorated chunks with terrain.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::World;
    /// use floralcraft::terrain::block::{ BlockPosition, Block };
    ///
    /// let world: World = World::new();
    /// let pos: BlockPosition = BlockPosition::new(0, -9999, 0);
    ///
    /// let name: Option<Block> = world.block(pos);
    ///
    /// assert!(name.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            dirty_chunks: HashSet::new(),
        }
    }

    /// Gets an option of an immutable chunk reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::World;
    /// use floralcraft::terrain::chunk::{ Chunk, ChunkPosition };
    ///
    /// let world: World = World::new();
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    ///
    /// let chunk: Option<&Chunk> = world.chunk(pos);
    ///
    /// assert!(chunk.is_none());
    /// ```
    pub fn chunk(&self, pos: ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// Sets chunk at the position stored in its data.
    pub fn set_chunk(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.pos, chunk);
    }

    /// Gets an option of a mutable chunk reference.
    pub fn get_chunk_mut(&mut self, pos: ChunkPosition) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    /// Returns bool for if a chunk is found at the passed position.
    pub fn is_chunk_at_pos(&self, pos: ChunkPosition) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Gets an option of block at a given global position.
    pub fn block(&self, pos: BlockPosition) -> Option<Block> {
        let chunk_pos: ChunkPosition = Self::block_to_chunk_pos(pos);
        let local_pos: BlockPosition = Self::global_to_local_pos(pos);
        Some(self.chunk(chunk_pos)?.block(local_pos))
    }

    /// Sets the block at a given global position.
    pub fn set_block(&mut self, pos: BlockPosition, block: Block) -> Option<()> {
        let chunk_pos: ChunkPosition = Self::block_to_chunk_pos(pos);
        let local_pos: BlockPosition = Self::global_to_local_pos(pos);
        self.get_chunk_mut(chunk_pos)?.set_block(local_pos, block);
        self.mark_chunks_dirty_with_adj(chunk_pos);
        Some(())
    }

    /// Gets an option of block at a given global position.
    pub fn block_exposed(&self, pos: BlockPosition) -> Option<bool> {
        let chunk_pos: ChunkPosition = Self::block_to_chunk_pos(pos);
        let local_pos: BlockPosition = Self::global_to_local_pos(pos);
        Some(self.chunk(chunk_pos)?.block_exposed(local_pos))
    }

    /// Gets an iter of all chunk positions in a square around the passed origin position.
    /// Radius of 0 results in 1 position.
    pub fn positions_in_square(
        origin: ChunkPosition,
        radius: u32
    ) -> impl Iterator<Item = ChunkPosition> {
        let radius: i32 = radius as i32;
        iproduct!(-radius..=radius, -radius..=radius).map(
            move |(x, y)| origin + ChunkPosition::new(x, y)
        )
    }

    /// Gets an iter of all chunks in a square around the passed origin position.
    /// Radius of 0 results in 1 chunk.
    pub fn chunks_in_square(
        &self,
        origin: ChunkPosition,
        radius: u32
    ) -> impl Iterator<Item = &Chunk> {
        Self::positions_in_square(origin, radius).filter_map(|pos| self.chunk(pos))
    }

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

    /// Gets and clears dirty chunks.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use floralcraft::terrain::chunk::ChunkPosition;
    /// use floralcraft::terrain::World;
    ///
    /// let mut world: World = World::new();
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    ///
    /// world.mark_chunks_dirty_with_adj(pos);
    ///
    /// let dirty_chunks: HashSet<ChunkPosition> = world.consume_dirty_chunks();
    /// assert!(!dirty_chunks.is_empty());
    ///
    /// let dirty_chunks: HashSet<ChunkPosition> = world.consume_dirty_chunks();
    /// assert!(dirty_chunks.is_empty());
    /// ```
    pub fn consume_dirty_chunks(&mut self) -> HashSet<ChunkPosition> {
        std::mem::take(&mut self.dirty_chunks)
    }

    /// Marks chunks touching the sides as dirty.
    /// Includes passed position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::chunk::ChunkPosition;
    /// use floralcraft::terrain::World;
    ///
    /// let mut world: World = World::new();
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    ///
    /// world.mark_chunks_dirty_with_adj(pos);
    /// ```
    pub fn mark_chunks_dirty_with_adj(&mut self, pos: ChunkPosition) {
        self.dirty_chunks.insert(pos);
        for adj_pos in Self::chunk_offsets(pos) {
            self.dirty_chunks.insert(adj_pos);
        }
    }

    fn chunk_offsets(pos: ChunkPosition) -> impl Iterator<Item = ChunkPosition> {
        CHUNK_ADJ_OFFSETS.iter().map(move |offset| { pos + offset })
    }
}
