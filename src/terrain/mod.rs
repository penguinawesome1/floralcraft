pub mod block;
pub mod chunk;
pub mod position;
pub mod subchunk;

pub use crate::terrain::block::*;
pub use crate::terrain::chunk::*;
pub use crate::terrain::position::*;

use std::collections::{ HashMap, HashSet };
use itertools::iproduct;

const CHUNK_ADJ_OFFSETS: [ChunkPosition; 5] = [
    ChunkPosition::new(0, 0),
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
    /// use floralcraft::terrain::BlockPosition;
    /// use floralcraft::terrain::block::Block;
    ///
    /// let world: World = World::new();
    /// let pos: BlockPosition = BlockPosition::new(0, -9999, 0);
    ///
    /// let name: Option<Block> = world.get_block_name(pos);
    ///
    /// assert_eq!(name, None);
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
    /// use floralcraft::terrain::ChunkPosition;
    /// use floralcraft::terrain::World;
    /// use floralcraft::terrain::Chunk;
    ///
    /// let world: World = World::new();
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    ///
    /// let chunk: Option<&Chunk> = world.get_chunk(pos);
    ///
    /// assert_eq!(chunk, None);
    /// ```
    pub fn get_chunk(&self, pos: ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// Sets chunk at the position stored in its data.
    pub fn set_chunk(&mut self, chunk: Chunk, pos: ChunkPosition) {
        self.chunks.insert(pos, chunk);
    }

    /// Gets an option of a mutable chunk reference.
    pub fn get_chunk_mut(&mut self, pos: ChunkPosition) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    /// Returns bool for if a chunk is found at the passed position.
    pub fn is_chunk_at_pos(&self, pos: ChunkPosition) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Gets and clears dirty chunks.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use floralcraft::terrain::ChunkPosition;
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

    /// Gets an option of block name at a given global position.
    pub fn get_block_name(&self, pos: BlockPosition) -> Option<Block> {
        let chunk_pos: ChunkPosition = Conversion::block_to_chunk_pos(pos);
        let local_pos: BlockPosition = Conversion::global_to_local_pos(pos);
        self.get_chunk(chunk_pos)?.get_block_name(local_pos)
    }

    /// Sets the block name at a given global position.
    pub fn set_block_name(&mut self, pos: BlockPosition, block_name: Block) -> Option<()> {
        let chunk_pos: ChunkPosition = Conversion::block_to_chunk_pos(pos);
        let local_pos: BlockPosition = Conversion::global_to_local_pos(pos);
        self.get_chunk_mut(chunk_pos)?.set_block_name(local_pos, block_name);
        self.mark_chunks_dirty_with_adj(chunk_pos);
        Some(())
    }

    /// Gets an iter of all chunk positions in a square around the passed origin position.
    /// Radius of 0 results in 1 chunk.
    pub fn get_positions_in_square(
        origin: ChunkPosition,
        radius: i32
    ) -> impl Iterator<Item = ChunkPosition> {
        iproduct!(-radius..=radius, -radius..=radius).map(
            move |(x, y)| origin + ChunkPosition::new(x, y)
        )
    }

    /// Gets an iter of all chunks in a square around the passed origin position.
    /// Radius of 0 results in 1 chunk.
    pub fn get_chunks_in_square(
        &self,
        origin: ChunkPosition,
        radius: i32
    ) -> impl Iterator<Item = &Chunk> {
        Self::get_positions_in_square(origin, radius).filter_map(|pos| self.get_chunk(pos))
    }

    /// Marks chunks touching the sides as dirty.
    /// Includes passed position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::ChunkPosition;
    /// use floralcraft::terrain::World;
    ///
    /// let mut world: World = World::new();
    /// let pos: ChunkPosition = ChunkPosition::new(0, 0);
    ///
    /// world.mark_chunks_dirty_with_adj(pos);
    /// ```
    pub fn mark_chunks_dirty_with_adj(&mut self, pos: ChunkPosition) {
        for &offset in &CHUNK_ADJ_OFFSETS {
            self.dirty_chunks.insert(pos + offset);
        }
    }
}
