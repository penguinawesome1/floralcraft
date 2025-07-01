use serde::{ Serialize, Deserialize };
use crate::terrain::section::Section;
use crate::terrain::{ Block, BlockPosition };

#[derive(Clone, Serialize, Deserialize)]
pub struct Subchunk {
    blocks: Option<Section>,
    sky_light: Option<Section>,
    block_light: Option<Section>,
    exposed_blocks: Option<Section>,
}

impl Subchunk {
    /// Create a new zeroed out subchunk.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::subchunk::Subchunk;
    /// use floralcraft::terrain::block::{ Block, BlockPosition };
    ///
    /// let mut subchunk: Subchunk = Subchunk::new();
    /// let pos: BlockPosition = BlockPosition::new(5, 5, 5);
    ///
    /// subchunk.set_block(pos, Block::Dirt);
    /// assert_eq!(subchunk.block(pos), Block::Dirt);
    /// ```
    pub fn new() -> Self {
        Self {
            blocks: None,
            sky_light: None,
            block_light: None,
            exposed_blocks: None,
        }
    }

    /// Gets the block given its sub position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::subchunk::Subchunk;
    /// use floralcraft::terrain::block::{ Block, BlockPosition };
    ///
    /// let mut subchunk: Subchunk = Subchunk::new();
    /// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
    ///
    /// subchunk.set_block(pos, Block::Dirt);
    /// let block: Block = subchunk.block(pos);
    ///
    /// assert_eq!(block, Block::Dirt);
    /// ```
    pub fn block(&self, pos: BlockPosition) -> Block {
        Block::from_u32(self.blocks.as_ref().map_or(0, |section| section.item(pos)) as u32)
    }

    /// Sets block given its sub position.
    pub fn set_block(&mut self, pos: BlockPosition, block: Block) {
        Self::set_section_item(&mut self.blocks, pos, block, 4);
    }

    /// Gets skylight given its sub position.
    pub fn sky_light(&self, pos: BlockPosition) -> u64 {
        self.sky_light.as_ref().map_or(0, |section| section.item(pos))
    }

    /// Sets skylight given its sub position.
    pub fn set_sky_light(&mut self, pos: BlockPosition, value: u8) {
        Self::set_section_item(&mut self.sky_light, pos, value, 4);
    }

    /// Gets block light given its sub position.
    pub fn block_light(&self, pos: BlockPosition) -> u64 {
        self.block_light.as_ref().map_or(0, |section| section.item(pos))
    }

    /// Sets block light given its sub position.
    pub fn set_block_light(&mut self, pos: BlockPosition, value: u8) {
        Self::set_section_item(&mut self.block_light, pos, value, 4);
    }

    /// Gets if block is exposed given its sub position.
    pub fn block_exposed(&self, pos: BlockPosition) -> bool {
        self.exposed_blocks.as_ref().map_or(0, |section| section.item(pos)) != 0
    }

    /// Sets if block is exposed given its sub position.
    pub fn set_block_exposed(&mut self, pos: BlockPosition, value: bool) {
        Self::set_section_item(&mut self.exposed_blocks, pos, value, 4);
    }

    /// Returns a bool for if all sections are empty.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_none()
    }

    fn set_section_item<T>(
        section_option: &mut Option<Section>,
        pos: BlockPosition,
        value: T,
        bits_per_item: u8
    )
        where T: Into<u64>
    {
        let value_u64: u64 = value.into();
        if value_u64 == 0 && section_option.is_none() {
            return; // return is placement is redundant
        }

        let section: &mut Section = section_option.get_or_insert_with(
            || Section::new(bits_per_item) // create new section if needed
        );
        section.set_item(pos, value_u64);

        if section.is_empty() {
            *section_option = None; // convert empty section to none
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::IVec3;

    #[test]
    fn test_new_is_empty() {
        let subchunk: Subchunk = Subchunk::new();
        assert!(subchunk.is_empty());
    }

    #[test]
    fn test_set_and_get_block() {
        let mut subchunk: Subchunk = Subchunk::new();
        let pos_1: IVec3 = IVec3::new(15, 1, 1);
        let pos_2: IVec3 = IVec3::new(3, 0, 2);

        subchunk.set_block(pos_1, Block::Dirt);
        subchunk.set_block(pos_1, Block::Grass);
        subchunk.set_block(pos_2, Block::Air);

        assert_eq!(subchunk.block(pos_1), Block::Grass);
        assert_eq!(subchunk.block(pos_2), Block::Air);
    }
}
