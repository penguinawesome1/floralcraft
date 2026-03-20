use crate::chunk::Chunk;
use crate::core::{BLOCK_OFFSETS, BlockPos, CHUNK_ADJ_OFFSETS, CHUNKS_DIR, ChunkPos, WorldField};
use crate::error::{AccessError, ChunkAccessError, ChunkOverwriteError, ChunkStoreError};
use ahash::RandomState;
use bincode::config;
use bincode::serde::encode_to_vec;
use dashmap::mapref::one::{Ref, RefMut};
use dashmap::{DashMap, Entry};
use itertools::iproduct;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct World<
    const CHUNK_W: usize,
    const CHUNK_H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
    const NUM_SUBCHUNKS: usize,
> {
    chunks: DashMap<
        ChunkPos,
        Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>,
        RandomState,
    >,
}

impl<
    const CHUNK_W: usize,
    const CHUNK_H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
    const NUM_SUBCHUNKS: usize,
> World<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>
{
    const CHUNK_D: usize = SUBCHUNK_D * NUM_SUBCHUNKS;

    pub fn get<F>(&self, pos: BlockPos) -> Result<F::Storage, AccessError>
    where
        F: WorldField,
    {
        let chunk_pos = Self::block_to_chunk_pos(pos);
        let local_pos = Self::global_to_local_pos(pos);
        Ok(self.chunk(chunk_pos)?.get::<F>(local_pos)?)
    }

    pub fn set<F>(&self, pos: BlockPos, val: F::Storage) -> Result<(), AccessError>
    where
        F: WorldField,
    {
        let chunk_pos = Self::block_to_chunk_pos(pos);
        let local_pos = Self::global_to_local_pos(pos);
        Ok(self
            .chunk_mut(chunk_pos)?
            .value_mut()
            .set::<F>(local_pos, val)?)
    }

    pub fn chunk(
        &self,
        pos: ChunkPos,
    ) -> Result<
        Ref<'_, ChunkPos, Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>>,
        ChunkAccessError,
    > {
        self.chunks
            .get(&pos)
            .ok_or(ChunkAccessError::ChunkUnloaded(pos))
    }

    pub fn chunk_mut(
        &self,
        pos: ChunkPos,
    ) -> Result<
        RefMut<'_, ChunkPos, Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>>,
        ChunkAccessError,
    > {
        self.chunks
            .get_mut(&pos)
            .ok_or(ChunkAccessError::ChunkUnloaded(pos))
    }

    /// Returns a bool for whether or not a chunk is found at the passed position.
    pub fn is_chunk_at_pos(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    /// Sets new given chunk at the passed position.
    /// Returns an error if a chunk is already at the position.
    pub fn add_chunk(
        &self,
        pos: ChunkPos,
        chunk: Option<Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>>,
    ) -> Result<(), ChunkOverwriteError> {
        match self.chunks.entry(pos) {
            Entry::Occupied(_) => Err(ChunkOverwriteError::ChunkAlreadyLoaded(pos)),
            Entry::Vacant(entry) => {
                entry.insert(chunk.unwrap_or(Chunk::default()));
                Ok(())
            }
        }
    }

    /// Gets an iter of all chunk positions in a square around the passed origin position.
    /// Radius of 0 results in 1 position.
    pub fn positions_in_square(origin: ChunkPos, radius: u32) -> impl Iterator<Item = ChunkPos> {
        let radius: i32 = radius as i32;
        iproduct!(-radius..=radius, -radius..=radius)
            .map(move |(x, y)| origin + ChunkPos::new(x, y))
    }

    /// Returns all adjacent chunk offsets.
    pub fn chunk_offsets(pos: ChunkPos) -> impl Iterator<Item = ChunkPos> {
        CHUNK_ADJ_OFFSETS.iter().map(move |offset| pos + offset)
    }

    /// Returns all adjacent block offsets.
    pub fn block_offsets(pos: BlockPos) -> impl Iterator<Item = BlockPos> {
        BLOCK_OFFSETS.iter().map(move |offset| pos + offset)
    }

    /// Returns an iter for every global position found in the passed chunk positions.
    pub fn coords_in_chunks<I>(chunk_positions: I) -> impl Iterator<Item = BlockPos>
    where
        I: Iterator<Item = ChunkPos>,
    {
        chunk_positions.flat_map(move |chunk_pos| Self::chunk_coords(chunk_pos))
    }

    /// Returns an iter for all block positions in the chunk offset by the chunk position.
    /// Passing in zero offset returns local positions.
    pub fn chunk_coords(offset: ChunkPos) -> impl Iterator<Item = BlockPos> {
        let base_block_pos = Self::chunk_to_block_pos(offset);

        iproduct!(
            0..CHUNK_W as i32,
            0..CHUNK_H as i32,
            0..Self::CHUNK_D as i32
        )
        .map(move |(x, y, z)| base_block_pos + BlockPos::new(x, y, z))
    }

    /// Converts a given chunk position to its zero corner block position.
    pub const fn chunk_to_block_pos(pos: ChunkPos) -> BlockPos {
        BlockPos::new(pos.x * (CHUNK_W as i32), pos.y * (CHUNK_H as i32), 0)
    }

    /// Gets the chunk position a block position falls into.
    pub const fn block_to_chunk_pos(pos: BlockPos) -> ChunkPos {
        ChunkPos::new(
            pos.x.div_euclid(CHUNK_W as i32),
            pos.y.div_euclid(CHUNK_H as i32),
        )
    }

    /// Finds the remainder of a global position using chunk size.
    pub const fn global_to_local_pos(pos: BlockPos) -> BlockPos {
        BlockPos::new(
            pos.x.rem_euclid(CHUNK_W as i32),
            pos.y.rem_euclid(CHUNK_H as i32),
            pos.z,
        )
    }

    pub async fn unload_chunk(&self, pos: ChunkPos) -> Result<(), ChunkStoreError> {
        let (_, chunk): (
            ChunkPos,
            Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>,
        ) = self.chunks.remove(&pos).ok_or(AccessError::ChunkAccess(
            ChunkAccessError::ChunkUnloaded(pos),
        ))?;

        fs::create_dir_all(CHUNKS_DIR).await?;
        let path: PathBuf = PathBuf::from(CHUNKS_DIR).join(format!("{}_{}.bin", pos.x, pos.y));
        let mut file: fs::File = fs::File::create(&path).await?;

        let encoded_data = encode_to_vec(&chunk, config::standard())?;

        file.write_all(&encoded_data).await?;

        Ok(())
    }

    pub async fn load_chunk(&self, pos: ChunkPos) -> Result<(), ChunkStoreError> {
        if self.is_chunk_at_pos(pos) {
            return Err(ChunkStoreError::ChunkOverwrite(
                ChunkOverwriteError::ChunkAlreadyLoaded(pos),
            ));
        }

        let path: PathBuf = PathBuf::from(CHUNKS_DIR).join(format!("{}_{}.bin", pos.x, pos.y));
        let encoded_data: Vec<u8> = fs::read(&path).await?;

        let (chunk, _): (
            Chunk<CHUNK_W, CHUNK_H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>,
            usize,
        ) = bincode::serde::decode_from_slice(&encoded_data, config::standard())?;

        self.chunks.insert(pos, chunk);

        Ok(())
    }
}
