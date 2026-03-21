use aether::prelude::*;
use bevy::prelude::*;
use floralcraft::world::World;
use floralcraft::{
    components::{GridPosition, ProjectionPlugin},
    config::{Config, ConfigPlugin, ConfigSet, TILE_H, TILE_W},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Floralcraft".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            ConfigPlugin {
                path: "assets/config.toml".to_string(),
            },
            ProjectionPlugin,
        ))
        .add_systems(Startup, (setup, spawn_world).chain().after(ConfigSet))
        .run();
}

#[derive(Resource)]
pub struct SpriteAssets {
    pub layout: Handle<TextureAtlasLayout>,
    pub texture: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    commands.spawn(Camera2d);

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_W, TILE_H),
        config.world.num_blocks,
        1,
        None,
        None,
    );

    let layout = texture_atlas_layouts.add(layout);
    let texture: Handle<Image> = asset_server.load("blocks.png");

    commands.insert_resource(SpriteAssets { layout, texture });
}

fn spawn_world(mut commands: Commands) {
    let world = World::default();
    commands.insert_resource(world);
}

fn draw_chunk(
    mut commands: Commands,
    sprite_assets: Res<SpriteAssets>,
    world: &World,
    chunk_pos: ChunkPos,
) {
    let sprite_texture = sprite_assets.texture.clone();
    let sprite_layout = sprite_assets.layout.clone();

    for pos in World::chunk_coords(chunk_pos) {
        let block_index = world.block(pos).unwrap() as usize;

        if block_index == 0 {
            continue;
        }

        commands.spawn((
            Sprite::from_atlas_image(
                sprite_texture.clone(),
                TextureAtlas {
                    layout: sprite_layout.clone(),
                    index: block_index,
                },
            ),
            GridPosition(pos.as_vec3()),
            Transform::IDENTITY,
        ));
    }
}
