use aether::prelude::*;
use std::sync::Arc;
use tokio::fs;

#[derive(Copy, Clone, Default)]
pub struct Block;

impl WorldField for Block {
    type Storage = u8;
    const BITS: u8 = 1;
    const INDEX: usize = 0;
}

#[derive(Copy, Clone, Default)]
pub struct SkyLight;

impl WorldField for SkyLight {
    type Storage = u8;
    const BITS: u8 = 5;
    const INDEX: usize = 1;
}

#[derive(Copy, Clone, Default)]
pub struct IsExposed;

impl WorldField for IsExposed {
    type Storage = bool;
    const BITS: u8 = 1;
    const INDEX: usize = 2;
}

type MyWorld = World<16, 16, 16, 3, 16>;
type MyChunk = Chunk<16, 16, 16, 3, 16>;

#[test]
fn test_all() -> Result<(), AccessError> {
    let world: Arc<MyWorld> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    world.add_chunk(chunk_pos, None).unwrap();

    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);
    let pos_3 = BlockPos::new(3, 0, 9999999);

    world.set::<IsExposed>(pos_1, true)?;
    world.set::<IsExposed>(pos_1, false)?;
    world.set::<IsExposed>(pos_2, true)?;

    assert_eq!(world.get::<IsExposed>(pos_1)?, false);
    assert_eq!(world.get::<IsExposed>(pos_2)?, true);

    assert!(world.get::<IsExposed>(pos_3).is_err());
    assert!(world.set::<IsExposed>(pos_3, true).is_err());

    Ok(())
}

#[test]
fn test_get_and_set_chunk() -> Result<(), BoundsError> {
    let mut chunk = MyChunk::default();
    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);

    chunk.set::<Block>(pos_1, 0)?;
    chunk.set::<Block>(pos_1, 4)?;
    chunk.set::<Block>(pos_2, 5)?;

    assert_eq!(chunk.get::<Block>(pos_1)?, 4);
    assert_eq!(chunk.get::<Block>(pos_2)?, 5);

    Ok(())
}

#[tokio::test]
async fn test_get_and_set_world() -> Result<(), AccessError> {
    let world: Arc<MyWorld> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    world.add_chunk(chunk_pos, None).unwrap();

    let pos_1 = BlockPos::new(15, 1, 200);
    let pos_2 = BlockPos::new(3, 0, 2);

    world.set::<IsExposed>(pos_1, true)?;
    world.set::<IsExposed>(pos_1, false)?;
    world.set::<IsExposed>(pos_2, true)?;

    assert_eq!(world.get::<IsExposed>(pos_1)?, false);
    assert_eq!(world.get::<IsExposed>(pos_2)?, true);

    Ok(())
}

#[tokio::test]
async fn test_save_load_chunk() -> Result<(), ChunkStoreError> {
    let world: Arc<MyWorld> = Arc::new(World::default());
    let chunk_pos = ChunkPos::new(0, 0);
    let pos = BlockPos::new(1, 2, 3);

    world.add_chunk(chunk_pos, None)?;
    world.set::<Block>(pos, 3)?;

    world.unload_chunk(chunk_pos).await?;

    assert!(world.get::<Block>(pos).is_err());

    world.load_chunk(chunk_pos).await?;

    let block: u8 = world.get::<Block>(pos)?;
    assert!(block == 3);

    if std::path::Path::new(CHUNKS_DIR).exists() {
        fs::remove_dir_all(CHUNKS_DIR).await.unwrap();
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_set_block_and_add_chunk() -> Result<(), Box<dyn std::error::Error + Send>>
{
    tokio::fs::create_dir_all(CHUNKS_DIR).await.unwrap();

    let world: Arc<MyWorld> = Arc::new(World::default());

    let world_clone1: Arc<MyWorld> = Arc::clone(&world);
    let handle1 = tokio::spawn(async move {
        let chunk_pos = ChunkPos::ZERO;
        world_clone1.add_chunk(chunk_pos, None).unwrap();

        for pos in MyWorld::chunk_coords(chunk_pos) {
            let value: u8 = (pos.x % 255) as u8;
            world_clone1.set::<Block>(pos, value).unwrap();
        }

        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });

    let world_clone2: Arc<MyWorld> = Arc::clone(&world);
    let handle2 = tokio::spawn(async move {
        let chunk_pos = ChunkPos::new(1, 0);
        world_clone2.add_chunk(chunk_pos, None).unwrap();

        for pos in MyWorld::chunk_coords(chunk_pos) {
            let value: u8 = ((pos.x % 255) as u8) + 1;
            world_clone2.set::<Block>(pos, value).unwrap();
        }

        Ok::<(), Box<dyn std::error::Error + Send>>(())
    });

    let _ = handle1.await.map_err(|e| eprintln!("{}", e)).unwrap();
    let _ = handle2.await.map_err(|e| eprintln!("{}", e)).unwrap();

    let pos: BlockPos = BlockPos::new(5, 5, 5);

    assert_eq!(world.get::<Block>(pos).unwrap(), 5);

    if std::path::Path::new(CHUNKS_DIR).exists() {
        fs::remove_dir_all(CHUNKS_DIR).await.unwrap();
    }

    Ok(())
}
