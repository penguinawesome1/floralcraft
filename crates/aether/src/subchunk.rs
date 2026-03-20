use crate::core::{BlockPos, Packable, WorldField};
use chroma::{BoundsError, Section};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Subchunk<
    const W: usize,
    const H: usize,
    const SUBCHUNK_D: usize,
    const NUM_FIELDS: usize,
> {
    #[serde(with = "serde_arrays")]
    pub sections: [Option<Section<W, H, SUBCHUNK_D>>; NUM_FIELDS],
}

impl<const W: usize, const H: usize, const SUBCHUNK_D: usize, const NUM_FIELDS: usize> Default
    for Subchunk<W, H, SUBCHUNK_D, NUM_FIELDS>
{
    fn default() -> Self {
        Self {
            sections: std::array::from_fn(|_| None),
        }
    }
}

impl<const W: usize, const H: usize, const SUBCHUNK_D: usize, const NUM_FIELDS: usize>
    Subchunk<W, H, SUBCHUNK_D, NUM_FIELDS>
{
    pub fn get<F: WorldField>(&self, pos: BlockPos) -> Result<F::Storage, BoundsError> {
        let val = self.sections[F::INDEX]
            .as_ref()
            .map_or(Ok(0), |s| s.item(pos))?;

        Ok(F::Storage::from_u64(val))
    }

    pub fn set<F: WorldField>(
        &mut self,
        pos: BlockPos,
        val: F::Storage,
    ) -> Result<(), BoundsError> {
        let raw_val = F::Storage::to_u64(val);

        if raw_val == 0 && self.sections[F::INDEX].is_none() {
            return Ok(());
        }

        let section = self.sections[F::INDEX].get_or_insert_with(|| Section::new(F::BITS));

        section.set_item(pos, raw_val)?;

        if raw_val == 0 && section.is_empty() {
            self.sections[F::INDEX] = None;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(Option::is_none)
    }
}
