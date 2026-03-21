use glam::{IVec2, IVec3};
pub type BlockPos = IVec3;
pub type ChunkPos = IVec2;

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

pub trait Packable: Sized {
    fn from_u16(value: u16) -> Self;
    fn to_u16(value: Self) -> u16;
}

macro_rules! impl_packable {
    ($($t:ty),*) => {
        $(
            impl Packable for $t {
                fn from_u16(value: u16) -> Self {
                    value as $t
                }
                fn to_u16(value: Self) -> u16 {
                    value as u16
                }
            }
        )*
    };
}

impl_packable!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

impl Packable for bool {
    fn from_u16(value: u16) -> Self {
        value != 0
    }
    fn to_u16(value: Self) -> u16 {
        if value { 1 } else { 0 }
    }
}

#[cfg(feature = "persistence")]
pub trait Persistable: serde::Serialize + serde::de::DeserializeOwned {}
#[cfg(feature = "persistence")]
impl<T: serde::Serialize + serde::de::DeserializeOwned> Persistable for T {}

#[cfg(not(feature = "persistence"))]
pub trait Persistable {}
#[cfg(not(feature = "persistence"))]
impl<T> Persistable for T {}

pub trait WorldField {
    type T: Clone + Copy + PartialEq + Default + Packable + Persistable;
    const BITS: u8;
    const INDEX: usize;
}
