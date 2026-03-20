use glam::{IVec2, IVec3};

pub type BlockPos = IVec3;
pub type ChunkPos = IVec2;

pub const CHUNKS_DIR: &str = "chunks";

pub const CHUNK_ADJ_OFFSETS: [ChunkPos; 4] = [
    ChunkPos::new(-1, 0),
    ChunkPos::new(1, 0),
    ChunkPos::new(0, -1),
    ChunkPos::new(0, 1),
];

pub const BLOCK_OFFSETS: [BlockPos; 6] = [
    BlockPos::new(1, 0, 0),
    BlockPos::new(0, 1, 0),
    BlockPos::new(0, 0, 1),
    BlockPos::new(-1, 0, 0),
    BlockPos::new(0, -1, 0),
    BlockPos::new(0, 0, -1),
];

pub trait Packable: Sized + Default + Copy {
    fn to_u64(self) -> u64;
    fn from_u64(val: u64) -> Self;
}

impl Packable for u8 {
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v as Self
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
}

impl Packable for bool {
    #[inline(always)]
    fn from_u64(v: u64) -> Self {
        v != 0
    }
    #[inline(always)]
    fn to_u64(self) -> u64 {
        self as u64
    }
}

pub trait WorldField {
    type Storage: Packable;
    const BITS: u8;
    const INDEX: usize;
}
