pub mod world_generator;
pub mod block_generator;

use macroquad::prelude::screen_height;
use std::collections::HashSet;
use crate::game::physics::{ Position2D, Position3D };
use crate::terrain::World;
use crate::terrain::block::{ BlockPosition, Block };
use crate::terrain::chunk::{ ChunkPosition, Chunk };
use crate::config::CONFIG;
use crate::rendering::PROJECTION;
use crate::config::WorldGeneration;
use crate::terrain_management::world_generator::{ WorldGeneratorTrait, new_generator };

pub struct BlockRenderData {
    pub block: Block,
    pub pos: BlockPosition,
    pub is_target: bool,
}

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

    /// Generates an option with block name and screen position given a block position.
    /// Ensures the block is visible and updates the target hover height.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::terrain::chunk::ChunkPosition;
    /// use floralcraft::terrain::block::{ Block, BlockPosition };
    /// use floralcraft::terrain_management::WorldLogic;
    ///
    /// let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
    /// let world_logic: WorldLogic = WorldLogic::new();
    /// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
    ///
    /// if let Some(_) = world_logic.block_render_data(pos, None) {
    ///     panic!();
    /// }
    /// ```
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
        Some(BlockRenderData { block, pos, is_target })
    }

    /// Progresses the world by one frame.
    pub fn update(
        &mut self,
        origin: ChunkPosition,
        radius: i32,
        mouse_pos: Position2D,
        is_left_click: bool,
        is_right_click: bool
    ) {
        let positions = World::positions_in_square(origin, radius);
        self.generate_missing_chunks(positions);
        self.handle_mouse_input(mouse_pos, is_left_click, is_right_click);
        self.rebuild_dirty_chunk_meshes();
    }

    /// Finds the block the mouse is hovering over and the replaceable block next to it.
    pub fn find_mouse_targets(&self, mouse_pos: Position2D) -> Option<MouseTargets> {
        let mut prev_air_pos: Option<BlockPosition> = None;

        for pos in Self::raycast_coords(mouse_pos) {
            let Some(block) = self.world.block(pos) else {
                continue; // continue if no block at position
            };

            if block.definition().is_replaceable() {
                prev_air_pos = Some(pos);
                continue; // continue if block is replaceable
            }

            // success if prev air block exists this pass
            if let Some(prev_air_pos) = prev_air_pos {
                return Some(MouseTargets { solid: pos, space: prev_air_pos });
            }

            return None; // fail if first block is solid
        }

        None
    }

    fn raycast_coords(pos: Position2D) -> impl Iterator<Item = BlockPosition> {
        let mut positions: Vec<BlockPosition> = (0..screen_height() as i32)
            .rev()
            .map(move |z| {
                let screen_pos: Position3D = Position3D::new(pos.x, pos.y + (z as f32), z as f32);
                PROJECTION.screen_to_world(screen_pos)
            })
            .collect();

        positions.dedup();
        positions.into_iter()
    }

    fn handle_mouse_input(
        &mut self,
        mouse_pos: Position2D,
        is_left_click: bool,
        is_right_click: bool
    ) {
        match self.find_mouse_targets(mouse_pos) {
            Some(targets) if is_left_click => self.break_block(targets.solid),
            Some(targets) if is_right_click => self.place_block(targets.space, Block::Dirt),
            _ => (),
        }
    }

    fn place_block(&mut self, pos: BlockPosition, block: Block) {
        self.world.set_block(pos, block);
    }

    fn break_block(&mut self, pos: BlockPosition) {
        let Some(block) = self.world.block(pos) else {
            return; // return if no block at position
        };

        if !block.definition().is_breakable() {
            return; // return if block not breakable
        }

        self.world.set_block(pos, Block::Air);
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
}
