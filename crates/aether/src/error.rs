use crate::core::ChunkPos;
use bincode::error::{DecodeError, EncodeError};
use chroma::BoundsError;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccessError {
    #[error(transparent)]
    ChunkAccess(#[from] ChunkAccessError),
    #[error(transparent)]
    Bounds(#[from] BoundsError),
}

#[derive(Debug, Error)]
pub enum ChunkAccessError {
    #[error("Chunk {0:?} is currently unloaded.")]
    ChunkUnloaded(ChunkPos),
}

#[derive(Debug, Error)]
pub enum ChunkOverwriteError {
    #[error("Chunk {0:?} already exists.")]
    ChunkAlreadyLoaded(ChunkPos),
}

#[derive(Debug, Error)]
pub enum ChunkStoreError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Access(#[from] AccessError),
    #[error(transparent)]
    ChunkOverwrite(#[from] ChunkOverwriteError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error(transparent)]
    Decode(#[from] DecodeError),
}
