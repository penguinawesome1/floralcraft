#![allow(unused)]

#[macro_export]
macro_rules! chunk {
    (
        $( #[$meta:meta] )*
        [$w:expr, $h:expr, $d:expr; $n:expr],
        $( $field:ident: $ty:ty ),* $(,)?
    ) => {
        $crate::subchunk! {
            $( #[$meta] )*
            [$w, $h, $d],
            $( $field: $ty ),*
        }

        $crate::derive_persistence! {
            $( #[$meta] )*
            #[derive(Default)]
            pub struct Chunk {
                pub subchunks: [Subchunk; $n],
            }
        }

        impl Chunk {
            $(
                pub fn $field(&self, pos: $crate::core::BlockPos) -> Option<$ty> {
                    let index = Self::subchunk_index(pos.z);
                    let sub_pos = Self::local_to_sub(pos);
                    self.subchunks.get(index).and_then(|s| s.$field(sub_pos))
                }

                $crate::__private::paste::paste! {
                    pub fn [<set_ $field>](&mut self, pos: $crate::core::BlockPos, val: $ty)
                            -> Result<$ty, $crate::__private::chroma::BoundsError> {
                        let index = Self::subchunk_index(pos.z);
                        let sub_pos = Self::local_to_sub(pos);

                        self.subchunks
                            .get_mut(index)
                            .ok_or($crate::__private::chroma::BoundsError::OutOfBounds(pos))
                            .and_then(|s| s.[<set_ $field>](sub_pos, val))
                    }
                }
            )*

            const fn subchunk_index(z: i32) -> usize {
                (z.div_euclid($d as i32)) as usize
            }

            const fn local_to_sub(pos: BlockPos) -> BlockPos {
                $crate::core::BlockPos::new(pos.x, pos.y, pos.z.rem_euclid($d as i32))
            }

            pub fn is_empty(&self) -> bool {
                self.subchunks.iter().all(|s| s.is_empty())
            }
        }

        impl ::std::ops::Deref for Chunk {
            type Target = [Subchunk; $n];

            fn deref(&self) -> &Self::Target {
                &self.subchunks
            }
        }

        impl ::std::ops::DerefMut for Chunk {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.subchunks
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::core::BlockPos;

    chunk! {
        [16, 16, 16; 16],
        is_true: bool,
    }

    #[test]
    fn it_works() {
        let mut chunk = Chunk::default();

        assert!(chunk.is_empty());

        let pos1 = BlockPos::new(0, 0, 0);
        let pos2 = BlockPos::new(0, 99999, 0);

        assert!(chunk.is_true(pos1) == Some(false));
        assert!(chunk.is_true(pos2) == Some(false));

        assert!(chunk.set_is_true(pos1, true) == Ok(false));
        assert!(!chunk.is_empty());

        assert!(chunk.set_is_true(pos1, false) == Ok(true));
        assert!(chunk.is_empty());
    }
}
