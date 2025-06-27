pub mod world_generator;
pub mod block_generator;

pub use crate::terrain_management::block_generator::*;
pub use crate::terrain_management::world_generator::*;

use macroquad::prelude::screen_height;
use std::collections::HashSet;
use itertools::iproduct;
use crate::game::physics::{ Position2D, Position3D };
use crate::terrain::{
    BlockPosition,
    ChunkPosition,
    Block,
    Chunk,
    World,
    BlockDefinition,
    Conversion,
    CHUNK_WIDTH,
    CHUNK_HEIGHT,
    CHUNK_DEPTH,
};
use crate::config::CONFIG;
use crate::rendering::PROJECTION;
use crate::config::WorldGeneration;

pub struct MouseTargets {
    /// The solid hovered block.
    pub solid: BlockPosition,
    /// The replaceable block in front of the hovered block.
    pub space: BlockPosition,
}

pub struct WorldLogic {
    pub world: World,
    pub generator: Box<dyn WorldGeneratorTrait>,
}

impl WorldLogic {
    /// Creates an instance based on a mutable reference to world.
    ///
    /// This interface allows more complex logic behind world interactions.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::World;
    /// use floralcraft::terrain_management::WorldLogic;
    ///
    /// let world_logic: WorldLogic = WorldLogic::new();
    /// ```
    pub fn new() -> Self {
        let params: &WorldGeneration = &CONFIG.world.generation;

        Self {
            world: World::new(),
            generator: new_generator(&params),
        }
    }

    /// Progresses the world by one frame.
    pub fn update(
        &mut self,
        origin: ChunkPosition,
        mouse_pos: Position2D,
        is_left_click: bool,
        is_right_click: bool
    ) {
        let radius: i32 = CONFIG.world.render_distance as i32;
        let positions = World::get_positions_in_square(origin, radius);
        self.generate_missing_chunks(positions);

        if let Some(targets) = self.find_mouse_targets(mouse_pos) {
            if is_left_click {
                self.break_block(targets.solid);
            }
            if is_right_click {
                self.place_block(targets.space, Block::Dirt);
            }
        }

        self.rebuild_dirty_chunk_meshes();
    }

    fn place_block(&mut self, pos: BlockPosition, block: Block) {
        self.world.set_block_name(pos, block);
    }

    fn break_block(&mut self, pos: BlockPosition) {
        let Some(block) = self.world.get_block_name(pos) else {
            return;
        };

        // return if block not breakable
        let block_def: BlockDefinition = block.definition();
        if !block_def.is_breakable() {
            return;
        }

        self.world.set_block_name(pos, Block::Air);
    }

    /// Finds the block the mouse is hovering over and the replaceable block next to it.
    pub fn find_mouse_targets(&self, mouse_pos: Position2D) -> Option<MouseTargets> {
        let mut prev_air_pos: Option<BlockPosition> = None;

        for z in (0..screen_height() as i32).rev() {
            let screen_pos: Position3D = Position3D::new(
                mouse_pos.x,
                mouse_pos.y + (z as f32),
                z as f32
            );
            let world_pos: BlockPosition = PROJECTION.screen_to_world(screen_pos);

            let Some(block) = self.world.get_block_name(world_pos) else {
                continue; // continue if no block at position
            };

            let block_def: BlockDefinition = block.definition();
            if block_def.is_replaceable() {
                prev_air_pos = Some(world_pos);
                continue; // continue if block is replaceable
            }

            // success if prev air block exists this pass
            if let Some(prev_air_pos) = prev_air_pos {
                return Some(MouseTargets { solid: world_pos, space: prev_air_pos });
            }

            return None; // fail if first block is solid
        }

        None
    }

    fn generate_missing_chunks<I>(&mut self, positions: I)
        where I: Iterator<Item = ChunkPosition> + Send + 'static
    {
        let missing_positions: Vec<ChunkPosition> = positions
            .filter(|&pos| !self.world.is_chunk_at_pos(pos))
            .collect();

        for pos in missing_positions {
            let chunk: Chunk = self.generator.generate_chunk(pos, &CONFIG.world.generation);
            self.world.set_chunk(chunk, pos);
            self.world.mark_chunks_dirty_with_adj(pos);
        }
    }

    // rebuilds data of dirty marked chunks
    // includes: if blocks are exposed
    fn rebuild_dirty_chunk_meshes(&mut self) {
        let chunks_to_rebuild: HashSet<ChunkPosition> = self.world.consume_dirty_chunks();

        for pos in chunks_to_rebuild {
            let mut exposed_list: Vec<(BlockPosition, bool)> = Vec::new();

            if let None = self.world.get_chunk(pos) {
                continue; // continue if no chunk at position
            }

            let chunk_block_base_pos: BlockPosition = Conversion::chunk_to_block_pos(pos);

            // for every block in chunk
            for (x, y, z) in iproduct!(
                0..CHUNK_WIDTH as i32,
                0..CHUNK_HEIGHT as i32,
                0..CHUNK_DEPTH as i32
            ) {
                let pos: BlockPosition = BlockPosition::new(x, y, z);
                let world_pos: BlockPosition = chunk_block_base_pos + pos;
                let is_exposed: bool = self.find_is_exposed(world_pos);

                exposed_list.push((pos, is_exposed));
            }

            // update the mutable chunk
            if let Some(chunk) = self.world.get_chunk_mut(pos) {
                exposed_list.into_iter().for_each(|(local_pos, is_exposed)| {
                    chunk.set_block_exposed(local_pos, is_exposed);
                });
            }
        }
    }

    // returns bool for if block is visible
    // will not be considered visible next to no block
    fn find_is_exposed(&self, world_pos: BlockPosition) -> bool {
        let visible_offsets: [BlockPosition; 5] = [
            BlockPosition::new(1, 0, 0),
            BlockPosition::new(0, 1, 0),
            BlockPosition::new(0, 0, 1),
            BlockPosition::new(-1, 0, 0),
            BlockPosition::new(0, -1, 0),
        ];

        // iterate through each offset to check if the offset exists or is transparent
        visible_offsets.iter().any(|&offset| {
            let adj_pos: BlockPosition = world_pos + offset;

            if let Some(block_name) = self.world.get_block_name(adj_pos) {
                !block_name.definition().is_visible()
            } else {
                false
            }
        })
    }
}
