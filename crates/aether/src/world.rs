#![allow(unused)]

#[macro_export]
macro_rules! world {
    (
        $( #[$world_meta:meta] )*
        ==
        $( #[$common_meta:meta] )*
        [$w:expr, $h:expr, $d:expr; $n:expr],
        $( $field:ident: $ty:ty ),* $(,)?
    ) => {
        $crate::chunk! {
            $( #[$common_meta] )*
            [$w, $h, $d; $n],
            $( $field: $ty ),*
        }

        $( #[$common_meta] )*
        $( #[$world_meta] )*
        #[derive(Default)]
        pub struct World {
            pub chunks: $crate::__private::dashmap::DashMap<$crate::core::ChunkPos, Chunk, $crate::__private::ahash::RandomState>,
        }

        impl World {
            const CHUNK_D: usize = ($d * $n) as usize;

            $(
                pub fn $field(&self, pos: $crate::core::BlockPos)
                        -> Result<$ty, $crate::error::AccessError> {
                    let chunk_pos = Self::block_to_chunk_pos(pos);
                    let local_pos = Self::global_to_local_pos(pos);

                    self.chunk(chunk_pos)?
                        .$field(local_pos)
                        .ok_or($crate::__private::chroma::BoundsError::OutOfBounds(pos).into())
                }

                $crate::__private::paste::paste! {
                    pub fn [<set_ $field>](&self, pos: $crate::core::BlockPos, val: $ty)
                            -> Result<$ty, $crate::error::AccessError> {
                        let chunk_pos = Self::block_to_chunk_pos(pos);
                        let local_pos = Self::global_to_local_pos(pos);
                        Ok(self.chunk_mut(chunk_pos)?.[<set_ $field>](local_pos, val)?)
                    }
                }
            )*

            pub fn chunk(
                &self,
                pos: $crate::core::ChunkPos,
            ) -> Result<
                $crate::__private::dashmap::mapref::one::Ref<'_, $crate::core::ChunkPos, Chunk>,
                $crate::error::ChunkAccessError
            > {
                self.chunks
                    .get(&pos)
                    .ok_or($crate::error::ChunkAccessError::ChunkUnloaded(pos))
            }

            pub fn chunk_mut(
                &self,
                pos: $crate::core::ChunkPos,
            ) -> Result<
                $crate::__private::dashmap::mapref::one::RefMut<'_, $crate::core::ChunkPos, Chunk>,
                $crate::error::ChunkAccessError
            > {
                self.chunks
                    .get_mut(&pos)
                    .ok_or($crate::error::ChunkAccessError::ChunkUnloaded(pos))
            }

            /// Sets new given `Chunk` at the passed position.
            /// Passing in `None` will create a default `Chunk`.
            /// Returns an `Err` if a `Chunk` is already at the `ChunkPos`.
            pub fn add_chunk(
                &self,
                pos: $crate::core::ChunkPos,
                chunk: Option<Chunk>,
            ) -> Result<(), $crate::error::ChunkOverwriteError> {
                match self.chunks.entry(pos) {
                    $crate::__private::dashmap::Entry::Occupied(_) =>
                        Err($crate::error::ChunkOverwriteError::ChunkAlreadyLoaded(pos)),
                    $crate::__private::dashmap::Entry::Vacant(entry) => {
                        entry.insert(chunk.unwrap_or(Chunk::default()));
                        Ok(())
                    }
                }
            }

            /// Removes any `Chunk` at the passed position.
            /// Returns an `Err` if a `Chunk` is not at the `ChunkPos`.
            pub fn remove_chunk(&self, pos: $crate::core::ChunkPos)
                    -> Result<Chunk, $crate::error::ChunkAccessError> {
                self.chunks
                    .remove(&pos)
                    .map(|(_, chunk)| chunk)
                    .ok_or($crate::error::ChunkAccessError::ChunkUnloaded(pos))
            }

            pub fn is_chunk_at_pos(&self, pos: $crate::core::ChunkPos) -> bool {
                self.chunks.contains_key(&pos)
            }

            /// Gets an iter of all chunk positions in a square around the passed origin position.
            /// Radius of 0 results in 1 position.
            pub fn positions_in_square(origin: $crate::core::ChunkPos, radius: u32)
                    -> impl Iterator<Item = $crate::core::ChunkPos> {
                let radius: i32 = radius as i32;
                $crate::__private::itertools::iproduct!(-radius..=radius, -radius..=radius)
                    .map(move |(x, y)| origin + $crate::core::ChunkPos::new(x, y))
            }

            /// Returns all adjacent chunk offsets.
            pub fn chunk_offsets(pos: $crate::core::ChunkPos)
                    -> impl Iterator<Item = $crate::core::ChunkPos> {
                $crate::core::CHUNK_ADJ_OFFSETS.iter().map(move |offset| pos + offset)
            }

            /// Returns all adjacent block offsets.
            pub fn block_offsets(pos: BlockPos)
                    -> impl Iterator<Item = $crate::core::BlockPos> {
                $crate::core::BLOCK_OFFSETS.iter().map(move |offset| pos + offset)
            }

            /// Returns an iter for every global position found in the passed chunk positions.
            pub fn coords_in_chunks<I>(chunk_positions: I)
                    -> impl Iterator<Item = $crate::core::BlockPos>
            where
                I: Iterator<Item = $crate::core::ChunkPos>,
            {
                chunk_positions.flat_map(move |chunk_pos| Self::chunk_coords(chunk_pos))
            }

            /// Returns an iter for all block positions in the chunk offset by the chunk position.
            /// Passing in zero offset returns local positions.
            pub fn chunk_coords(offset: ChunkPos) -> impl Iterator<Item = BlockPos> {
                let base_block_pos = Self::chunk_to_block_pos(offset);

                $crate::__private::itertools::iproduct!(
                    0..$w as i32,
                    0..$h as i32,
                    0..Self::CHUNK_D as i32
                )
                .map(move |(x, y, z)| base_block_pos + BlockPos::new(x, y, z))
            }

            /// Converts a given chunk position to its zero corner block position.
            pub const fn chunk_to_block_pos(pos: ChunkPos) -> BlockPos {
                BlockPos::new(pos.x * ($w as i32), pos.y * ($h as i32), 0)
            }

            /// Gets the `ChunkPos` the passed `BlockPos` falls into.
            pub const fn block_to_chunk_pos(pos: $crate::core::BlockPos)
                    -> $crate::core::ChunkPos {
                $crate::core::ChunkPos::new(
                    pos.x.div_euclid($w as i32),
                    pos.y.div_euclid($h as i32),
                )
            }

            /// Finds the remainder of a global position using `Chunk` size.
            pub const fn global_to_local_pos(pos: $crate::core::BlockPos)
                    -> $crate::core::BlockPos {
                $crate::core::BlockPos::new(
                    pos.x.rem_euclid($w as i32),
                    pos.y.rem_euclid($h as i32),
                    pos.z,
                )
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        core::{BlockPos, ChunkPos},
        error::ChunkAccessError,
    };

    world! {
        ==
        [16, 16, 16; 16],
        is_true: bool,
    }

    #[test]
    fn it_works() {
        let world = World::default();
        let pos1 = BlockPos::new(0, 0, 0);
        assert!(world.is_true(pos1).is_err());

        let chunk_pos = ChunkPos::new(0, 0);
        world.add_chunk(chunk_pos, None);

        assert_eq!(world.set_is_true(pos1, true), Ok(false));
    }
}
