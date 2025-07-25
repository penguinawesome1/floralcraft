use crate::world_controller::WorldController;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use floralcraft_terrain::ChunkPosition;
use spriso::IsoProjection;

#[derive(Resource)]
pub struct IsoProj(pub IsoProjection);

#[derive(Resource)]
pub struct ImageMap {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

pub fn draw_chunk(
    draw_context: &mut DrawContext,
    world_controller: &WorldController,
    chunk_pos: ChunkPosition,
) {
    match world_controller.world.chunk_render_data(chunk_pos) {
        Ok(render_data) => render_data.for_each(|(block, pos)| draw_context.draw_block(block, pos)),
        Err(e) => eprintln!("Failed to draw chunk at pos {:?}: {}", chunk_pos, e),
    }
}

#[derive(SystemParam)]
pub struct DrawContext<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub block_assets: Res<'w, ImageMap>,
    pub proj: Res<'w, IsoProj>,
}

impl<'w, 's> DrawContext<'w, 's> {
    pub fn draw_block(&mut self, block: u8, pos: glam::IVec3) {
        let screen_pos: glam::Vec3 = self.proj.0.world_to_screen(pos);

        self.commands.spawn((
            Sprite {
                image: self.block_assets.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.block_assets.layout.clone(),
                    index: block as usize,
                }),
                ..default()
            },
            Transform::from_xyz(
                screen_pos.x as f32,
                (-screen_pos.y + screen_pos.z) as f32,
                (pos.x + pos.y + pos.z) as f32,
            ),
        ));
    }
}
