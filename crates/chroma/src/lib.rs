use glam::IVec3;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BoundsError {
    #[error("Position {0:?} is out of bounds for the section.")]
    OutOfBounds(IVec3),
}

pub trait Item: Clone + Copy + PartialEq + Default + Send + Sync + Debug + 'static {}
impl<T: Clone + Copy + PartialEq + Default + Send + Sync + Debug + 'static> Item for T {}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Section<T: Item, const W: usize, const H: usize, const D: usize> {
    data: Vec<u64>,
    palette: Vec<T>,
    bits_per_item: u8,
}

impl<T: Item, const W: usize, const H: usize, const D: usize> Default for Section<T, W, H, D> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<T: Item, const W: usize, const H: usize, const D: usize> Section<T, W, H, D> {
    const VOLUME: usize = W * H * D;
    const STRIDE_Z: usize = W * H;
    const STRIDE_Y: usize = W;

    /// Creates a new section given dimensions and initial bits per item.
    ///
    /// The more bits per item the more memory but less likely to repack.
    ///
    /// # Examples
    ///
    /// ```
    /// use glam::IVec3;
    /// use chroma::Section;
    ///
    /// let mut section: Section<16, 16, 16> = Section::new(2);
    /// assert!(section.is_empty());
    ///
    /// let pos: IVec3 = IVec3::new(0, 0, 0);
    /// section.set(pos, 2);
    /// assert!(!section.is_empty());
    /// ```
    pub fn new(bits_per_item: u8) -> Self {
        let palette_len: usize = 1 << bits_per_item.max(1);
        let total_bits_needed: usize = (bits_per_item as usize) * Self::VOLUME;
        let data_len: usize = total_bits_needed.div_ceil(u64::BITS as usize) + 1;

        let mut palette = Vec::with_capacity(palette_len);
        palette.push(T::default());

        Self {
            data: vec![0; data_len],
            palette,
            bits_per_item,
        }
    }

    /// Create and fill an entire section at once with data.
    ///
    /// Data uses this indexing: z * width * height + y * width + x
    pub fn from_data(data: &[T]) -> Self {
        assert_eq!(data.len(), Self::VOLUME, "Data must match section volume");

        let mut palette = Vec::new();
        palette.push(T::default());

        for item in data {
            if !palette.contains(item) {
                palette.push(*item);
            }
        }

        let bits_per_item = (palette.len().max(2) - 1).ilog2() as u8 + 1;

        let total_bits = (bits_per_item as usize) * Self::VOLUME;
        let data_len = total_bits.div_ceil(u64::BITS as usize) + 1;

        let mut section = Self {
            data: vec![0; data_len],
            palette,
            bits_per_item,
        };

        for (i, item) in data.iter().enumerate() {
            let palette_index = section.palette.iter().position(|id| id == item).unwrap();
            unsafe {
                section.set_ex(i, palette_index);
            }
        }

        section
    }

    /// Returns if there is only one item type and it has a value of zero.
    pub fn is_empty(&self) -> bool {
        if self.bits_per_item == 0 {
            return self
                .palette
                .first()
                .map_or(true, |&val| val == T::default());
        }

        self.data.iter().all(|&word| word == 0)
    }

    /// Returns the dimensions (width, height, depth) of the section.
    pub const fn dimensions(&self) -> IVec3 {
        IVec3::new(W as i32, H as i32, D as i32)
    }

    /// Returns the total number of items in the section.
    pub const fn volume(&self) -> usize {
        Self::VOLUME
    }

    /// Gets an item given its three dimensional position.
    pub fn get(&self, pos: impl Into<IVec3>) -> Option<T> {
        let pos = pos.into();
        if !Self::is_in_bounds(pos) {
            return None;
        }
        Some(unsafe { self.get_unchecked(pos) })
    }

    /// Sets an item at the given three dimensional position.
    /// Returns an error if position is out of the section bounds.
    pub fn set(&mut self, pos: impl Into<IVec3>, item: T) -> Result<T, BoundsError> {
        let pos = pos.into();
        if !Self::is_in_bounds(pos) {
            return Err(BoundsError::OutOfBounds(pos));
        }

        unsafe {
            let old_item = self.get_unchecked(pos);
            self.set_unchecked(pos, item);
            Ok(old_item)
        }
    }

    /// Gets an item given its three dimensional position.
    ///
    /// # Panics
    ///
    /// Will be unchecked and may panic if position is out of bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that pos is strictly within the bounds of W, H, and D
    pub unsafe fn get_unchecked(&self, pos: IVec3) -> T {
        let item_index: usize = Self::item_index(pos);
        let palette_index: usize = Self::palette_index(&self.data, self.bits_per_item, item_index);
        unsafe { *self.palette.get_unchecked(palette_index) }
    }

    /// Sets an item at the given three dimensional position.
    ///
    /// # Panics
    ///
    /// Will be unchecked and may panic if position is out of bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that pos is strictly within the bounds of W, H, and D
    pub unsafe fn set_unchecked(&mut self, pos: IVec3, item: T) {
        let palette_index = self.get_or_insert_palette_index(item);
        let item_index: usize = Self::item_index(pos);

        unsafe {
            self.set_ex(item_index, palette_index);
        }
    }

    unsafe fn set_ex(&mut self, item_index: usize, palette_index: usize) {
        debug_assert!(
            palette_index < 1usize << self.bits_per_item,
            "repack needed first"
        );

        let (word_index, bit_in_word) = Self::split_index(item_index, self.bits_per_item);
        let bits_in_first_word: usize = u64::BITS as usize - bit_in_word;

        unsafe {
            if (self.bits_per_item as usize) <= bits_in_first_word {
                let item_mask: u64 = (1u64 << self.bits_per_item).wrapping_sub(1);
                *self.data.get_unchecked_mut(word_index) &= !(item_mask << bit_in_word);
                *self.data.get_unchecked_mut(word_index) |=
                    ((palette_index as u64) & item_mask) << bit_in_word;
            } else {
                let bits_in_second_word: usize = (self.bits_per_item as usize) - bits_in_first_word;
                let mask_for_first_word: u64 = (1u64 << bits_in_first_word).wrapping_sub(1);
                *self.data.get_unchecked_mut(word_index) &= !(mask_for_first_word << bit_in_word);
                *self.data.get_unchecked_mut(word_index) |=
                    ((palette_index as u64) & mask_for_first_word) << bit_in_word;

                debug_assert!(
                    word_index + 1 < self.data.len(),
                    "should not write beyond data bounds"
                );

                let mask_for_second_word: u64 = (1u64 << bits_in_second_word).wrapping_sub(1);
                *self.data.get_unchecked_mut(word_index + 1) &= !mask_for_second_word;
                *self.data.get_unchecked_mut(word_index + 1) |=
                    ((palette_index as u64) >> bits_in_first_word) & mask_for_second_word;
            }
        }
    }

    /// Adjusts the data to account for a new amount of bits per item.
    fn repack(&mut self, new_bits_per_item: u8) {
        debug_assert!(
            self.bits_per_item < new_bits_per_item,
            "Repack must increase bits"
        );

        let old_data = std::mem::take(&mut self.data);
        let old_bits = self.bits_per_item;

        self.bits_per_item = new_bits_per_item;
        let new_total_bits: usize = (self.bits_per_item as usize) * Self::VOLUME;
        let new_data_len = new_total_bits.div_ceil(u64::BITS as usize) + 1;
        self.data = vec![0; new_data_len];

        for item_index in 0..Self::VOLUME {
            let palette_index = Self::palette_index(&old_data, old_bits, item_index);
            unsafe {
                self.set_ex(item_index, palette_index);
            }
        }
    }

    fn get_or_insert_palette_index(&mut self, item: T) -> usize {
        self.palette
            .iter()
            .position(|&id| id == item)
            .unwrap_or_else(|| {
                let new_index: usize = self.palette.len();
                self.palette.push(item);

                if 1 << self.bits_per_item <= new_index {
                    self.repack(self.bits_per_item + 1);
                }

                new_index
            })
    }

    #[inline]
    const fn split_index(item_index: usize, bits_per_item: u8) -> (usize, usize) {
        let bit_offset: usize = item_index * (bits_per_item as usize);
        let word_index: usize = bit_offset / u64::BITS as usize;
        let bit_in_word: usize = bit_offset % u64::BITS as usize;
        (word_index, bit_in_word)
    }

    #[inline]
    const fn item_index(pos: IVec3) -> usize {
        (pos.z as usize) * Self::STRIDE_Z + (pos.y as usize) * Self::STRIDE_Y + (pos.x as usize)
    }

    #[inline]
    fn palette_index(data: &[u64], bits: u8, item_index: usize) -> usize {
        let (word_index, bit_in_word) = Self::split_index(item_index, bits);
        let mut item = data[word_index];

        if bit_in_word + (bits as usize) > u64::BITS as usize {
            item >>= bit_in_word;
            let remaining_bits_n: usize = bit_in_word + (bits as usize) - u64::BITS as usize;
            let next_word: u64 = data[word_index + 1];
            item |= next_word << ((bits as usize) - remaining_bits_n);
        } else {
            item >>= bit_in_word;
        }

        let mask = (1u64 << bits) - 1;
        (item & mask) as usize
    }

    #[inline]
    const fn is_in_bounds(pos: IVec3) -> bool {
        (pos.x as u32) < W as u32 && (pos.y as u32) < H as u32 && (pos.z as u32) < D as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MySection = Section<u32, 16, 16, 16>;

    #[test]
    fn test_new_is_empty() {
        let section: Section<bool, 16, 16, 16> = Section::new(2);
        assert!(section.is_empty());
    }

    #[test]
    fn test_set_and_get_conversion() {
        let mut section: Section<u32, 16, 16, 16> = Section::new(4);
        let pos_1: IVec3 = IVec3::new(15, 1, 1);
        let pos_2: IVec3 = IVec3::new(15, 1, 2);

        section.set((15, 1, 1), 3).unwrap();
        section.set((15, 1, 1), 2).unwrap();
        section.set((15, 1, 2), 1).unwrap();

        assert_eq!(section.get((15, 1, 1)).unwrap(), 2);
        assert_eq!(section.get((15, 1, 2)).unwrap(), 1);

        assert_eq!(section.get(pos_2).unwrap(), 1);

        section.set(pos_1, 4).unwrap();
        assert_eq!(section.get((15, 1, 1)).unwrap(), 4);
    }

    #[test]
    fn test_set_and_get() {
        let mut section: Section<u32, 16, 16, 16> = Section::new(4);
        let pos_1: IVec3 = IVec3::new(15, 1, 1);
        let pos_2: IVec3 = IVec3::new(15, 1, 2);

        unsafe {
            section.set_unchecked(pos_1, 3);
            section.set_unchecked(pos_1, 2);
            section.set_unchecked(pos_2, 1);

            assert_eq!(section.get_unchecked(pos_1), 2);
            assert_eq!(section.get_unchecked(pos_2), 1);
        }
    }

    #[test]
    fn test_repack() {
        let mut section: Section<u32, 16, 16, 16> = Section::new(1);
        let pos: IVec3 = IVec3::new(3, 5, 3);

        unsafe {
            section.set_unchecked(pos, 30);
            assert_eq!(section.get_unchecked(pos), 30);
        }
    }

    #[test]
    fn test_max_fill() {
        let mut section: Section<u32, 16, 16, 16> = Section::new(0);

        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    let pos: IVec3 = IVec3::new(x, y, z);
                    section.set(pos, (x + y + z) as u32).unwrap();
                }
            }
        }
    }

    #[test]
    fn test_from_data_consistency() {
        let mut data = vec![0u32; MySection::VOLUME];
        data[0] = 10;
        data[1] = 20;
        data[2] = 30;
        data[3] = 10;

        let section = MySection::from_data(&data);

        assert!(section.palette.contains(&0));
        assert!(section.palette.contains(&10));
        assert!(section.palette.contains(&20));
        assert!(section.palette.contains(&30));
        assert_eq!(section.palette.len(), 4);

        assert_eq!(section.bits_per_item, 2);

        for (i, &original_val) in data.iter().enumerate() {
            let palette_index = MySection::palette_index(&section.data, section.bits_per_item, i);

            let stored_val = section.palette[palette_index];
            assert_eq!(original_val, stored_val, "Value mismatch at index {}", i);
        }
    }

    #[test]
    fn test_bit_width_edge_cases() {
        let calc = |len: usize| (len.max(2) - 1).ilog2() as u8 + 1;

        assert_eq!(calc(0), 1, "Empty palette should be 1 bit");
        assert_eq!(calc(1), 1, "Single item palette should be 1 bit");
        assert_eq!(calc(2), 1, "Two item palette should be 1 bit");
        assert_eq!(calc(3), 2, "Three items need 2 bits");
        assert_eq!(calc(4), 2, "Four items need 2 bits");
        assert_eq!(calc(5), 3, "Five items need 3 bits");
    }
}
