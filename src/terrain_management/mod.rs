mod world_generator;
mod block_generator;

use glam::{ Vec2, Vec3 };
use std::collections::HashSet;
use crate::{
    terrain::block::{ Block, BlockPosition },
    terrain::World,
    terrain::chunk::{ Chunk, ChunkPosition },
    config::{ CONFIG, WorldGeneration },
    terrain_management::world_generator::{ new_generator, WorldGeneratorTrait },
    rendering::isometric_projection::PROJECTION,
};

pub struct BlockRenderData {
    pub block: Block,
    pub pos: Vec3,
    pub is_target: bool,
}

#[derive(bevy::prelude::Resource)]
pub struct WorldLogic {
    pub world: World,
    generator: Box<dyn WorldGeneratorTrait>,
}

impl WorldLogic {
    pub fn new() -> Self {
        let params: &WorldGeneration = &CONFIG.world.generation;

        Self {
            world: World::new(),
            generator: new_generator(&params),
        }
    }

    /// Generates an option with block name and screen position given a block position.
    /// Ensures the block is visible and updates the target hover height.
    pub fn block_render_data(
        &self,
        pos: BlockPosition,
        target_solid: Option<BlockPosition>
    ) -> Option<BlockRenderData> {
        let is_exposed: bool = self.world
            .block_exposed(pos)
            .expect("chunk should exist when retrieving its exposure");
        if !is_exposed {
            return None; // skip if block not exposed
        }

        let block: Block = self.world.block(pos)?;
        if !block.definition().is_visible() {
            return None; // skip if block not visible
        }

        let is_target: bool = target_solid.map_or(false, |target| target == pos);
        let screen_pos: Vec3 = PROJECTION.world_to_screen(pos);
        Some(BlockRenderData { block, pos: screen_pos, is_target })
    }

    pub fn update(&mut self, origin: ChunkPosition) {
        let chunk_positions = World::positions_in_square(origin, CONFIG.world.render_distance);
        self.generate_missing_chunks(chunk_positions);
        self.rebuild_dirty_chunk_meshes();
    }

    fn generate_missing_chunks<I>(&mut self, positions: I)
        where I: Iterator<Item = ChunkPosition> + Send + 'static
    {
        let missing_positions: Vec<ChunkPosition> = positions
            .filter(|&pos| !self.world.is_chunk_at_pos(pos))
            .collect();

        for pos in missing_positions {
            let chunk: Chunk = self.generator.generate_chunk(pos, &CONFIG.world.generation);
            self.world.set_chunk(chunk);
            self.world.mark_chunks_dirty_with_adj(pos);
        }
    }

    // rebuilds data of dirty marked chunks
    // includes: if blocks are exposed
    fn rebuild_dirty_chunk_meshes(&mut self) {
        let chunks_to_rebuild: HashSet<ChunkPosition> = self.world.consume_dirty_chunks();

        for chunk_pos in chunks_to_rebuild {
            if let None = self.world.chunk(chunk_pos) {
                continue; // continue if no chunk at position
            }

            let mut exposed_list: Vec<(BlockPosition, bool)> = Vec::new();
            let chunk_block_base_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

            // for every block in chunk
            for pos in Chunk::chunk_coords() {
                let world_pos: BlockPosition = chunk_block_base_pos + pos;
                let is_exposed: bool = self.find_is_exposed(world_pos);

                exposed_list.push((pos, is_exposed));
            }

            // update the mutable chunk
            if let Some(chunk) = self.world.get_chunk_mut(chunk_pos) {
                exposed_list.into_iter().for_each(|(local_pos, is_exposed)| {
                    chunk.set_block_exposed(local_pos, is_exposed);
                });
            }
        }
    }

    // returns bool for if block is visible
    // will not be considered visible next to no block
    fn find_is_exposed(&self, pos: BlockPosition) -> bool {
        Chunk::block_offsets(pos).any(|adj_pos| {
            if let Some(block) = self.world.block(adj_pos) {
                !block.definition().is_visible()
            } else {
                false
            }
        })
    }

    fn _raycast_coords(pos: Vec2, height: u32) -> impl Iterator<Item = BlockPosition> {
        let mut positions: Vec<BlockPosition> = (0..height as i32)
            .rev()
            .map(move |z| {
                let screen_pos: Vec3 = Vec3::new(pos.x, pos.y + (z as f32), z as f32);
                PROJECTION.screen_to_world(screen_pos)
            })
            .collect();

        positions.dedup();
        positions.into_iter()
    }
}

// pub struct MouseTargets {
//     /// The solid hovered block.
//     pub solid: BlockPosition,
//     /// The replaceable block in front of the hovered block.
//     pub space: BlockPosition,
// }

//     // /// Finds the block the mouse is hovering over and the replaceable block next to it.
//     // pub fn find_mouse_targets(&self, mouse_pos: Vec2) -> Option<MouseTargets> {
//     //     let mut prev_air_pos: Option<BlockPosition> = None;

//     //     for pos in Self::raycast_coords(mouse_pos) {
//     //         let Some(block) = self.world.block(pos) else {
//     //             continue; // continue if no block at position
//     //         };

//     //         if block.definition().is_replaceable() {
//     //             prev_air_pos = Some(pos);
//     //             continue; // continue if block is replaceable
//     //         }

//     //         // success if prev air block exists this pass
//     //         if let Some(prev_air_pos) = prev_air_pos {
//     //             return Some(MouseTargets { solid: pos, space: prev_air_pos });
//     //         }

//     //         return None; // fail if first block is solid
//     //     }

//     //     None
//     // }

//     // fn handle_mouse_input(
//     // &mut self,
//     // mouse_pos: Vec2,
//     // is_left_click: bool,
//     // is_right_click: bool
//     // ) {
//     // match self.find_mouse_targets(mouse_pos) {
//     // Some(targets) if is_left_click => self.break_block(targets.solid),
//     // Some(targets) if is_right_click => self.place_block(targets.space, Block::Dirt),
//     // _ => (),
//     // }
//     // }

//     fn place_block(&mut self, pos: BlockPosition, block: Block) {
//         self.world.set_block(pos, block);
//     }

//     fn break_block(&mut self, pos: BlockPosition) {
//         let Some(block) = self.world.block(pos) else {
//             return; // return if no block at position
//         };

//         if !block.definition().is_breakable() {
//             return; // return if block not breakable
//         }

//         self.world.set_block(pos, Block::Air);
//     }
