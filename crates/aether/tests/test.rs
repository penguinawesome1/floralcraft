use aether::prelude::*;
use std::sync::Arc;

#[cfg(feature = "persistence")]
use {
    aether::storage::{ChunkStorage, FileStorage},
    std::path::Path,
    tokio::fs,
};

#[cfg(feature = "persistence")]
const CHUNKS_DIR: &'static str = "tests/chunks";

world! {
    ==
    [16, 16, 16; 16],
    block: u8,
    sky_light: u8,
    is_exposed: bool,
}

#[test]
fn test_all() -> Result<(), AccessError> {
    let world: Arc<World> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    world.insert(&chunk_pos, None).unwrap();

    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);

    world.set_is_exposed(&pos_1, true)?;
    world.set_is_exposed(&pos_1, false)?;
    world.set_is_exposed(&pos_2, true)?;

    assert_eq!(world.is_exposed(&pos_1)?, false);
    assert_eq!(world.is_exposed(&pos_2)?, true);

    Ok(())
}

#[cfg(feature = "persistence")]
#[test]
fn test_get_and_set_chunk() -> Result<(), BoundsError> {
    let mut chunk: Chunk = Chunk::default();
    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);

    chunk.set_block(pos_1, 0)?;
    chunk.set_block(pos_1, 4)?;
    chunk.set_block(pos_2, 5)?;

    assert_eq!(chunk.block(pos_1).unwrap(), 4);
    assert_eq!(chunk.block(pos_2).unwrap(), 5);

    Ok(())
}

#[cfg(feature = "persistence")]
#[tokio::test]
async fn test_get_and_set_world() -> Result<(), AccessError> {
    let world: Arc<World> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    world.insert(&chunk_pos, None).unwrap();

    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);

    world.set_is_exposed(&pos_1, true)?;
    world.set_is_exposed(&pos_1, false)?;
    world.set_is_exposed(&pos_2, true)?;

    assert_eq!(world.is_exposed(&pos_1)?, false);
    assert_eq!(world.is_exposed(&pos_2)?, true);

    Ok(())
}

#[cfg(feature = "persistence")]
#[tokio::test]
async fn test_save_load_chunk() -> Result<(), ChunkStoreError> {
    let world: Arc<World> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    let pos = BlockPos::new(1, 2, 3);

    world.insert(&chunk_pos, None)?;
    world.set_block(&pos, 3)?;

    let storage = FileStorage::new(CHUNKS_DIR.into()).await?;
    storage
        .save_chunk(chunk_pos, &*world.get(&chunk_pos).unwrap())
        .await?;

    world.remove(&chunk_pos).unwrap();
    assert!(world.block(&pos).is_err());

    let chunk: Chunk = storage.load_chunk(chunk_pos).await?;

    _ = world.insert(&chunk_pos, Some(chunk));

    assert!(world.block(&pos)? == 3);

    if Path::new(CHUNKS_DIR).exists() {
        fs::remove_dir_all(CHUNKS_DIR).await.unwrap();
    }

    Ok(())
}

#[cfg(feature = "persistence")]
#[tokio::test]
async fn test_concurrent_set_block_and_add_chunk() -> Result<(), Box<dyn std::error::Error + Send>>
{
    tokio::fs::create_dir_all(CHUNKS_DIR).await.unwrap();

    let world: Arc<World> = Arc::new(World::default());

    let world_clone1: Arc<World> = Arc::clone(&world);
    let handle1 = tokio::spawn(async move {
        let chunk_pos = ChunkPos::ZERO;
        world_clone1.insert(&chunk_pos, None).unwrap();

        for pos in World::blocks_in(chunk_pos) {
            let value: u8 = (pos.x % 255) as u8;
            world_clone1.set_block(&pos, value).unwrap();
        }

        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });

    let world_clone2: Arc<World> = Arc::clone(&world);
    let handle2 = tokio::spawn(async move {
        let chunk_pos = ChunkPos::new(1, 0);
        world_clone2.insert(&chunk_pos, None).unwrap();

        for pos in World::blocks_in(chunk_pos) {
            let value: u8 = ((pos.x % 255) as u8) + 1;
            world_clone2.set_block(&pos, value).unwrap();
        }

        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });

    let _ = handle1.await.map_err(|e| eprintln!("{}", e)).unwrap();
    let _ = handle2.await.map_err(|e| eprintln!("{}", e)).unwrap();

    let pos = BlockPos::new(5, 5, 5);

    assert_eq!(world.block(&pos).unwrap(), 5);

    if Path::new(CHUNKS_DIR).exists() {
        fs::remove_dir_all(CHUNKS_DIR).await.unwrap();
    }

    Ok(())
}
