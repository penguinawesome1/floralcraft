mod chunks;

use crate::GameState;
use crate::camera::update_camera;
use crate::config::{TILE_H, TILE_W};
use crate::world::dictionary::ENTRIES;
use bevy::prelude::*;
use chunks::{DrawsQueued, RenderedChunks, dispatch_chunk_drawing, poll_chunk_drawing};

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderedChunks>();
        app.init_resource::<DrawsQueued>();
        app.add_systems(Startup, setup_renderer);
        app.add_systems(
            Update,
            (dispatch_chunk_drawing, poll_chunk_drawing, update_camera)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
pub struct SpriteAssets {
    pub layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
    pub material: Handle<ColorMaterial>,
}

fn setup_renderer(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_W, TILE_H),
        ENTRIES.len() as u32,
        1,
        None,
        None,
    );

    let layout = texture_atlas_layouts.add(layout);
    let texture = asset_server.load("blocks.png");
    let material = materials.add(ColorMaterial::from(texture.clone()));

    commands.insert_resource(SpriteAssets {
        layout,
        texture,
        material,
    });
}
