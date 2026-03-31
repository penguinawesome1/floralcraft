use crate::GameState;
use crate::camera::update_camera;
use crate::config::{Config, TILE_H, TILE_W};
use crate::player::PlayerMovedFilter;
use crate::position::GridPosition;
use crate::world::World as AeWorld;
use crate::world::dictionary::ENTRIES;
use aether::prelude::*;
use bevy::prelude::*;
use std::collections::HashSet;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderedChunks>();
        app.add_systems(Startup, setup_renderer);
        app.add_systems(
            Update,
            (draw_chunks, update_camera).run_if(in_state(GameState::Playing)),
        );
    }
}

fn setup_renderer(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
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
    let texture: Handle<Image> = asset_server.load("blocks.png");

    commands.insert_resource(SpriteAssets { layout, texture });
}

#[derive(Resource)]
pub struct SpriteAssets {
    pub layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct RenderedChunks(pub HashSet<IVec2>);

pub fn draw_chunks(
    mut commands: Commands,
    mut rendered_chunks: ResMut<RenderedChunks>,
    sprite_assets: Res<SpriteAssets>,
    config: Res<Config>,
    world: Res<AeWorld>,
    query: Query<&GridPosition, PlayerMovedFilter>,
) {
    let radius = config.world.render_distance;

    for player_pos in &query {
        let chunk_pos = AeWorld::to_chunk(player_pos.0.as_ivec3());

        for pos in AeWorld::square_around(chunk_pos, radius) {
            if rendered_chunks.0.contains(&pos) || !world.contains(&pos) {
                continue;
            }

            draw_chunk(&mut commands, &sprite_assets, &world, pos);
            rendered_chunks.0.insert(pos);
        }
    }
}

fn draw_chunk(
    commands: &mut Commands,
    sprite_assets: &SpriteAssets,
    world: &AeWorld,
    chunk_pos: ChunkPos,
) {
    let sprite_texture = sprite_assets.texture.clone();
    let sprite_layout = sprite_assets.layout.clone();

    for pos in AeWorld::blocks_in(chunk_pos) {
        let Ok(block) = world.block(&pos) else {
            return; // chunk isn't loaded
        };

        if block == 0 || !world.is_exposed(&pos).unwrap() {
            continue;
        }

        commands.spawn((
            Sprite::from_atlas_image(
                sprite_texture.clone(),
                TextureAtlas {
                    layout: sprite_layout.clone(),
                    index: block as usize,
                },
            ),
            GridPosition(pos.as_vec3()),
            Transform::IDENTITY,
            Visibility::default(),
        ));
    }
}
