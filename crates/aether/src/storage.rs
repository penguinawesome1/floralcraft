#![cfg(feature = "persistence")]

use crate::core::ChunkPos;
use crate::error::ChunkStoreError;
use async_trait::async_trait;
use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use tokio::fs;

#[async_trait]
pub trait ChunkStorage<C> {
    async fn save_chunk(&self, pos: ChunkPos, chunk: &C) -> Result<(), ChunkStoreError>;
    async fn load_chunk(&self, pos: ChunkPos) -> Result<C, ChunkStoreError>;
}

pub struct FileStorage {
    dir: PathBuf,
}

impl FileStorage {
    pub async fn new(dir: PathBuf) -> Result<Self, ChunkStoreError> {
        fs::create_dir_all(&dir).await?;
        Ok(Self { dir })
    }
}

#[async_trait]
impl<C> ChunkStorage<C> for FileStorage
where
    C: Serialize + DeserializeOwned + Send + Sync,
{
    async fn save_chunk(&self, pos: ChunkPos, chunk: &C) -> Result<(), ChunkStoreError> {
        let path = self.dir.join(format!("{}_{}.bin", pos.x, pos.y));
        let encoded = encode_to_vec(chunk, standard())?;
        fs::write(path, encoded).await?;
        Ok(())
    }

    async fn load_chunk(&self, pos: ChunkPos) -> Result<C, ChunkStoreError> {
        let path = self.dir.join(format!("{}_{}.bin", pos.x, pos.y));
        let data = fs::read(path).await?;
        let (chunk, _) = decode_from_slice(&data, standard())?;
        Ok(chunk)
    }
}
