use crate::core::{BlockPos, WorldField};
use crate::subchunk::Subchunk;
use chroma::BoundsError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Chunk<
    const W: usize,
    const H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
    const NUM_SUBCHUNKS: usize,
> {
    #[serde(with = "serde_arrays")]
    pub subchunks: [Subchunk<W, H, SUBCHUNK_D, NUM_FIELDS>; NUM_SUBCHUNKS],
}

impl<
    const W: usize,
    const H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
    const NUM_SUBCHUNKS: usize,
> Default for Chunk<W, H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>
{
    fn default() -> Self {
        Self {
            subchunks: std::array::from_fn(|_| Subchunk::default()),
        }
    }
}

impl<
    const W: usize,
    const H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
    const NUM_SUBCHUNKS: usize,
> Chunk<W, H, SUBCHUNK_D, NUM_FIELDS, NUM_SUBCHUNKS>
{
    pub fn get<F>(&self, pos: BlockPos) -> Result<F::Storage, BoundsError>
    where
        F: WorldField,
    {
        let index = Self::subchunk_index(pos.z);
        let sub_pos = Self::local_to_sub(pos);

        self.subchunks
            .get(index)
            .ok_or(BoundsError::OutOfBounds(pos))?
            .get::<F>(sub_pos)
    }

    pub fn set<F>(&mut self, pos: BlockPos, val: F::Storage) -> Result<(), BoundsError>
    where
        F: WorldField,
    {
        let index = Self::subchunk_index(pos.z);
        let sub_pos = Self::local_to_sub(pos);

        self.subchunks
            .get_mut(index)
            .ok_or(BoundsError::OutOfBounds(pos))?
            .set::<F>(sub_pos, val)
    }

    const fn subchunk_index(pos_z: i32) -> usize {
        (pos_z as usize).div_euclid(SUBCHUNK_D)
    }

    const fn local_to_sub(pos: BlockPos) -> BlockPos {
        BlockPos::new(pos.x, pos.y, pos.z.rem_euclid(SUBCHUNK_D as i32))
    }
}
