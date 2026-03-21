#![allow(unused)]

#[macro_export]
macro_rules! subchunk {
    (
        $( #[$meta:meta] )*
        [$w:expr, $h:expr, $d:expr],
        $( $field:ident: $ty:ty ),* $(,)?
    ) => {
        $crate::derive_persistence! {
            $( #[$meta] )*
            #[derive(Default)]
            pub struct Subchunk {
                $( pub $field: Option<$crate::__private::chroma::Section<$ty, $w, $h, $d>>, )*
            }
        }

        impl Subchunk {
            $(
                pub fn $field(&self, pos: $crate::core::BlockPos) -> Option<$ty> {
                    match self.$field.as_ref() {
                        Some(f) => f.get(pos),
                        None => Some(<$ty>::default())
                    }
                }

                $crate::__private::paste::paste! {
                    pub fn [<set_ $field>](&mut self, pos: $crate::core::BlockPos, val: $ty)
                            -> Result<$ty, $crate::__private::chroma::BoundsError> {
                        let is_val_default = val == <$ty>::default();

                        if is_val_default && self.$field.is_none() {
                            return Ok(<$ty>::default());
                        }

                        let section = self.$field
                            .get_or_insert_with(|| $crate::__private::chroma::Section::new(2));

                        let old_val = section.set(pos, val)?;

                        if is_val_default && section.is_empty() {
                            self.$field = None;
                        }

                        Ok(old_val)
                    }
                }
            )*

            pub const fn is_empty(&self) -> bool {
                true $( && self.$field.is_none() )*
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::core::BlockPos;

    subchunk! {
        [16, 16, 16],
        is_true: bool,
    }

    #[test]
    fn it_works() {
        let mut subchunk = Subchunk::default();

        assert!(subchunk.is_empty());

        let pos1 = BlockPos::new(0, 0, 0);
        let pos2 = BlockPos::new(0, 99999, 0);

        assert!(subchunk.is_true(pos1) == Some(false));
        assert!(subchunk.is_true(pos2) == Some(false));

        assert!(subchunk.set_is_true(pos1, true) == Ok(false));
        assert!(!subchunk.is_empty());

        assert!(subchunk.set_is_true(pos1, false) == Ok(true));
        assert!(subchunk.is_empty());
    }
}
