mod world_generator;
mod block_generator;

use glam::{ Vec2, Vec3 };
use crate::{
    terrain::block::{ Block, BlockPosition },
    terrain::World,
    terrain::chunk::{ Chunk, ChunkPosition, CHUNK_VOLUME },
    config::{ CONFIG, WorldGeneration },
    terrain_management::world_generator::{ new_generator, WorldGeneratorTrait },
    rendering::isometric_projection::PROJECTION,
};

pub struct BlockRenderData {
    pub block: Block,
    pub pos: Vec3,
    pub is_target: bool,
}

pub struct MouseTargets {
    /// The solid hovered block.
    pub solid: BlockPosition,
    /// The replaceable block in front of the hovered block.
    pub space: BlockPosition,
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
            return None;
        }

        let block: Block = self.world.block(pos).expect("position should exist inside chunk");
        if !block.definition().is_visible() {
            return None;
        }

        let is_target: bool = target_solid.map_or(false, |target| target == pos);
        let screen_pos: Vec3 = PROJECTION.world_to_screen(pos);
        Some(BlockRenderData { block, pos: screen_pos, is_target })
    }

    /// Finds the block the mouse is hovering over and the replaceable block next to it.
    /// Returns None if either one is missing.
    pub fn find_mouse_targets(&self, mouse_pos: Vec2, screen_height: u32) -> Option<MouseTargets> {
        let mut prev_air_pos: Option<BlockPosition> = None;

        Self::raycast_coords(mouse_pos, screen_height).find_map(|world_pos| {
            let block: Block = self.world.block(world_pos)?;
            let is_replaceable: bool = block.definition().is_replaceable();
            if is_replaceable {
                prev_air_pos = Some(world_pos);
                return None;
            }

            let Some(space) = prev_air_pos else {
                return None;
            };

            Some(MouseTargets { solid: world_pos, space })
        })
    }

    pub fn update(&mut self, origin: ChunkPosition) {
        let chunk_positions = World::positions_in_square(origin, CONFIG.world.render_distance);
        let dirty_positions = self.world.consume_dirty_chunks().into_iter();

        self.rebuild_found_chunks(dirty_positions);
        self.generate_missing_chunks(chunk_positions);
    }

    fn generate_missing_chunks<I>(&mut self, positions: I)
        where I: Iterator<Item = ChunkPosition> + Send + 'static
    {
        let missing_positions: Vec<ChunkPosition> = positions
            .filter(|&pos| !self.world.is_chunk_at_pos(pos))
            .collect();

        missing_positions.iter().for_each(|&pos| {
            let chunk: Chunk = self.generator.generate_chunk(pos, &CONFIG.world.generation);
            self.world.set_chunk(chunk);
            self.world.mark_chunks_dirty_with_adj(pos);
        });
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

    fn raycast_coords(pos: Vec2, height: u32) -> impl Iterator<Item = BlockPosition> {
        let mut positions: Vec<BlockPosition> = (0..height as i32)
            .rev()
            .map(move |z| {
                let screen_pos: Vec3 = Vec3::new(pos.x, pos.y - (z as f32), z as f32);
                PROJECTION.screen_to_world(screen_pos)
            })
            .collect();

        positions.dedup();
        positions.into_iter()
    }

    // may pass in non existent locations
    fn rebuild_found_chunks<I>(&mut self, positions: I) where I: Iterator<Item = ChunkPosition> {
        let mut exposed_list: Vec<(BlockPosition, bool)> = Vec::with_capacity(CHUNK_VOLUME);

        let found_positions: Vec<ChunkPosition> = positions
            .filter(|&pos| self.world.is_chunk_at_pos(pos))
            .collect();

        for chunk_pos in found_positions {
            let chunk_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

            for pos in Chunk::chunk_coords() {
                let world_pos: BlockPosition = chunk_block_pos + pos;
                let is_exposed: bool = self.find_is_exposed(world_pos);

                exposed_list.push((pos, is_exposed));
            }

            let chunk = self.world
                .mut_chunk(chunk_pos)
                .expect("mut chunk should exist where nonmut one is");
            exposed_list.iter().for_each(|(local_pos, is_exposed)| {
                chunk.set_block_exposed(*local_pos, *is_exposed);
            });

            exposed_list.clear();
        }
    }
}

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
