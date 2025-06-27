pub mod assets;
pub mod isometric_projection;

pub use crate::rendering::assets::*;
pub use crate::rendering::isometric_projection::*;

use macroquad::prelude::{ Texture2D, draw_texture, WHITE };
use itertools::iproduct;
use crate::terrain::{
    Block,
    BlockPosition,
    BlockRenderData,
    Chunk,
    CHUNK_WIDTH,
    CHUNK_HEIGHT,
    CHUNK_DEPTH,
};
use crate::config::CONFIG;
use crate::game::physics::Position3D;
use crate::game::player::{ PlayerFrameKey, Player };
use crate::terrain_management::MouseTargets;

/// Generates an option with block name and screen position given a block position.
/// Ensures the block is visible and updates the target hover height.
///
/// # Examples
///
/// ```
/// use floralcraft::terrain::{ ChunkPosition, BlockPosition };
/// use floralcraft::terrain::Block;
/// use floralcraft::terrain::chunk::Chunk;
///
/// let chunk_pos: ChunkPosition = ChunkPosition::new(0, 0);
/// let chunk: Chunk = Chunk::new(chunk_pos);
/// let pos: BlockPosition = BlockPosition::new(0, 0, 0);
///
/// if let Some(_) = chunk.get_block_render_data(pos, None) {
///     panic!();
/// }
/// ```
// pub fn get_block_render_data(
//     &self,
//     pos: BlockPosition,
//     target_solid: Option<BlockPosition>
// ) -> Option<BlockRenderData> {
//     self.is_block_exposed(pos)
//         .filter(|&is_exposed| is_exposed)
//         .and_then(|_| self.get_block_name(pos))
//         .filter(|block_name| block_name.definition().is_visible())
//         .map(|block_name| {
//             let world_pos = self.local_to_global_pos(pos);
//             let is_target = target_solid.map_or(false, |target| target == world_pos);
//             BlockRenderData { block_name, world_pos, is_target }
//         })
// }
// const fn local_to_global_pos(&self, pos: BlockPosition) -> BlockPosition {
//     BlockPosition::new(
//         self.pos.x * (CHUNK_WIDTH as i32) + pos.x,
//         self.pos.y * (CHUNK_HEIGHT as i32) + pos.y,
//         pos.z
//     )
// }
pub struct Renderer {
    assets: Assets,
}

impl Renderer {
    /// Create new renderer struct to render images.
    pub async fn new() -> Result<Self, String> {
        Ok(Self {
            assets: Assets::new().await?,
        })
    }

    /// Renders all images to the screen.
    pub async fn update(
        &mut self,
        player: &Player,
        chunks: &[&Chunk],
        targets: Option<MouseTargets>
    ) {
        let target_solid: Option<BlockPosition> = targets.map(|t| t.solid);

        for &chunk in chunks {
            self.draw_chunk(chunk, target_solid).await;
        }

        self.draw_player(player.hitbox.pos, player.image_name).await;
    }

    async fn draw_player(&mut self, screen_pos: Position3D, player_image_name: PlayerFrameKey) {
        self.draw(
            ImageKey::Player(player_image_name),
            screen_pos.x,
            screen_pos.y - screen_pos.z
        ).await;
    }

    async fn draw_chunk(&mut self, chunk: &Chunk, target_solid: Option<BlockPosition>) {
        for (x, y, z) in iproduct!(
            0..CHUNK_WIDTH as i32,
            0..CHUNK_HEIGHT as i32,
            0..CHUNK_DEPTH as i32
        ) {
            let pos: BlockPosition = BlockPosition::new(x, y, z);

            // if let Some(data) = chunk.get_block_render_data(pos, target_solid) {
            //     self.draw_block(data).await;
            // }
        }
    }

    async fn draw_block(&mut self, data: BlockRenderData) {
        let screen_pos: Position3D = PROJECTION.world_to_screen(data.world_pos);
        self.draw(
            ImageKey::Block(data.block_name),
            screen_pos.x,
            screen_pos.y -
                screen_pos.z -
                (data.is_target as i32 as f32) * CONFIG.world.target_hover_height
        ).await;
    }

    // draws image on screen given image key
    // assigns a texture to the key and defaults to missing
    async fn draw(&mut self, image_key: ImageKey, x: f32, y: f32) {
        let texture: &Texture2D = match self.assets.get_or_load_image(image_key).await {
            Ok(tex) => tex,
            Err(e) => {
                eprintln!("Error drawing image {:?}: {}", image_key, e);
                self.assets
                    .get_or_load_image(ImageKey::Block(Block::Missing)).await
                    .expect("Critical: Missing texture could not be loaded either!")
            }
        };

        draw_texture(&texture, x, y, WHITE);
    }
}
