// use crate::terrain::{ Block, BlockPosition };

// pub const SUBCHUNK_WIDTH: u32 = 16;
// pub const SUBCHUNK_HEIGHT: u32 = 16;
// pub const SUBCHUNK_DEPTH: u32 = 16;
// pub const SUBCHUNK_VOLUME: usize = (SUBCHUNK_WIDTH * SUBCHUNK_HEIGHT * SUBCHUNK_DEPTH) as usize;

// pub struct Subchunk {
//     palette: Vec<Block>,
//     data: Vec<u64>,
//     bits_per_block: u8,
// }

// impl Subchunk {
//     pub fn new(block: Block) -> Self {
//         let bits_per_block: u8 = 4;
//         let mut palette: Vec<Block> = Vec::with_capacity(16);
//         palette.push(block);

//         let data_len: usize = ((SUBCHUNK_VOLUME as usize) * (bits_per_block as usize) + 63) / 64;
//         let data: Vec<u64> = vec![0; data_len];

//         Self { palette, data, bits_per_block }
//     }

//     pub fn get_block(&self, pos: BlockPosition) -> Block {
//         let block_index: usize = Self::get_block_index(pos);
//         let palette_index: usize = self.get_palette_index(block_index);
//         self.palette[palette_index]
//     }

//     pub fn set_block(&mut self, pos: BlockPosition, block: Block) {
//         let block_index: usize = Self::get_block_index(pos);

//         if let Some(palette_index) = self.palette.iter().position(|&id| id == block) {
//             self.set_palette_index(block_index, palette_index);
//             return;
//         }

//         self.palette.push(block);

//         let required_bits: u8 = (self.palette.len() as f64).log2().ceil() as u8;
//         if required_bits > self.bits_per_block {
//             self.repack(required_bits);
//         }
//     }

//     pub fn is_empty(&self) -> bool {
//         self.palette.len() == 1 && self.palette[0] == Block::Air
//     }

//     fn get_block_index(pos: BlockPosition) -> usize {
//         debug_assert!(
//             pos.x < (SUBCHUNK_WIDTH as i32) &&
//                 pos.y < (SUBCHUNK_HEIGHT as i32) &&
//                 pos.z < (SUBCHUNK_DEPTH as i32),
//             "Out of bounds for index access in subchunk: {:?}",
//             pos
//         );
//         (pos.x as usize) * ((SUBCHUNK_HEIGHT * SUBCHUNK_DEPTH) as usize) +
//             (pos.y as usize) * (SUBCHUNK_DEPTH as usize) +
//             (pos.z as usize)
//     }

//     fn get_palette_index(&self, block_index: usize) -> usize {
//         let bit_offset: usize = block_index * (self.bits_per_block as usize);
//         let block_index: usize = bit_offset / 64;
//         let offset: usize = bit_offset % 64;

//         let mask: u64 = (1u64 << self.bits_per_block) - 1;
//         ((self.data[block_index] >> offset) & mask) as usize
//     }

//     fn set_palette_index(&mut self, block_index: usize, palette_index: usize) {
//         let bit_offset: usize = block_index * (self.bits_per_block as usize);
//         let block_index: usize = bit_offset / 64;
//         let offset: usize = bit_offset % 64;

//         let clear_mask: u64 = !(((1u64 << self.bits_per_block) - 1) << offset);
//         self.data[block_index] &= clear_mask; // clear old bit

//         let value_to_set: u64 = (palette_index as u64) << offset;
//         self.data[block_index] |= value_to_set; // set new bit
//     }

//     fn repack(&mut self, new_bits_per_block: u8) {
//         debug_assert!(
//             self.bits_per_block <= new_bits_per_block,
//             "bits per block should not be updated"
//         );

//         let old_bits_per_block: u8 = self.bits_per_block;
//         let old_data: Vec<u64> = std::mem::take(&mut self.data);

//         let new_data_len: usize =
//             ((SUBCHUNK_VOLUME as usize) * (new_bits_per_block as usize) + 63) / 64;
//         let mut new_data: Vec<u64> = vec![0; new_data_len];

//         let old_mask: u64 = (1u64 << old_bits_per_block).wrapping_sub(1);

//         for i in 0..SUBCHUNK_VOLUME {
//             let old_bit_offset: usize = i * (old_bits_per_block as usize);
//             let old_block_index: usize = old_bit_offset / 64;
//             let old_offset: usize = old_bit_offset % 64;

//             let mut palette_index: u64 = (old_data[old_block_index] >> old_offset) & old_mask;

//             let bits_remaining_in_u64: usize = 64 - old_offset;
//             if (old_bits_per_block as usize) > bits_remaining_in_u64 {
//                 let bits_from_next_u64: usize =
//                     (old_bits_per_block as usize) - bits_remaining_in_u64;

//                 if old_block_index + 1 < old_data.len() {
//                     let next_u64_part: u64 =
//                         old_data[old_block_index + 1] &
//                         (1u64 << bits_from_next_u64).wrapping_sub(1);
//                     palette_index |= next_u64_part << bits_remaining_in_u64;
//                 }
//             }

//             let new_bit_offset: usize = i * (new_bits_per_block as usize);
//             let new_block_index: usize = new_bit_offset / 64;
//             let new_offset: usize = new_bit_offset % 64;

//             let value_to_write: u64 = palette_index;
//             let new_mask: u64 = (1u64 << new_bits_per_block).wrapping_sub(1);

//             new_data[new_block_index] &= !(new_mask << new_offset);
//             new_data[new_block_index] |= value_to_write << new_offset;

//             let new_bits_remaining_in_u64: usize = 64 - new_offset;
//             if (new_bits_per_block as usize) > new_bits_remaining_in_u64 {
//                 if new_block_index + 1 < new_data.len() {
//                     new_data[new_block_index + 1] &= !(
//                         (new_mask >> new_bits_remaining_in_u64) <<
//                         0
//                     );
//                     new_data[new_block_index + 1] |= value_to_write >> new_bits_remaining_in_u64;
//                 }
//             }
//         }

//         self.data = new_data;
//         self.bits_per_block = new_bits_per_block;
//     }
// }
