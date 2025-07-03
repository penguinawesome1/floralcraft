pub mod assets;
pub mod isometric_projection;

// use bevy::prelude::{ Commands, Res, ResMut, Sprite, Transform, Handle, Image, AssetServer };
// use glam::Vec3;
// use crate::terrain::block::{ Block, BlockPosition };
// use crate::terrain::chunk::{ Chunk, ChunkPosition };
// use crate::terrain::World;
// use crate::config::CONFIG;
// use crate::terrain_management::{ WorldLogic, MouseTargets, BlockRenderData };
// use crate::rendering::assets::{ Assets, ImageKey };

// pub struct Renderer {
//     assets: Assets,
// }

// impl Renderer {
//     /// Create new renderer struct to render images.
//     pub async fn new() -> Result<Self, String> {
//         Ok(Self {
//             assets: Assets::new(),
//         })
//     }

//     /// Renders all images to the screen.
//     pub async fn update(
//         &mut self,
//         player: &Player,
//         chunks: &[&Chunk],
//         world_logic: &WorldLogic,
//         targets: Option<MouseTargets>
//     ) {
//         let target_solid: Option<BlockPosition> = targets.map(|t| t.solid);

//         for &chunk in chunks {
//             self.draw_chunk(world_logic, chunk.pos, target_solid).await;
//         }

//         self.draw_player(player.hitbox.pos, player.image_name).await;
//     }

//     async fn draw_player(&mut self, screen_pos: Position3D, player_image_name: PlayerFrameKey) {
//         self.draw(
//             ImageKey::Player(player_image_name),
//             screen_pos.x,
//             screen_pos.y - screen_pos.z
//         ).await;
//     }

//     async fn draw_chunk(
//         &mut self,
//         world_logic: &WorldLogic,
//         chunk_pos: ChunkPosition,
//         target_solid: Option<BlockPosition>
//     ) {
//         let chunk_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

//         for pos in Chunk::chunk_coords() {
//             let world_pos: BlockPosition = chunk_block_pos + pos;

//             if let Some(data) = world_logic.block_render_data(world_pos, target_solid) {
//                 self.draw_block(data).await;
//             }
//         }
//     }

//     async fn draw_block(&mut self, data: BlockRenderData) {
//         let screen_pos: Position3D = PROJECTION.world_to_screen(data.pos);
//         self.draw(
//             ImageKey::Block(data.block),
//             screen_pos.x,
//             screen_pos.y -
//                 screen_pos.z -
//                 (data.is_target as i32 as f32) * CONFIG.world.target_hover_height
//         );
//     }

//     // draws image on screen given image key
//     // assigns a texture to the key and defaults to missing
//     fn draw(
//         &mut self,
//         mut commands: Commands,
//         asset_server: Res<AssetServer>,
//         image_key: ImageKey,
//         x: f32,
//         y: f32,
//         z: f32
//     ) {
//         let texture: Handle<Image> = self.assets.image(asset_server, image_key);
//         commands.spawn((Sprite::from_image(texture), Transform::from_xyz(x, y, z)));
//     }
// }

// // let texture: Handle<Image> = assets.image(asset_server, ImageKey::Block(Block::Stone));
