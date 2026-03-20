use bevy::prelude::*;
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
        .add_systems(Startup, setup.after(ConfigSet))
        .run();
}

fn setup(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    commands.spawn(Camera2d);

    let texture = asset_server.load("blocks.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(TILE_W, TILE_H),
        config.world.num_blocks,
        1,
        None,
        None,
    );
    let layout_handle = texture_atlas_layouts.add(layout);

    commands.spawn((
        Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas {
                layout: layout_handle.clone(),
                index: 0, // The first tile in the sheet
            },
        ),
        GridPosition(Vec3::new(0.0, 0.0, 0.0)),
        Transform::IDENTITY,
    ));

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: layout_handle,
                index: 1,
            },
        ),
        GridPosition(Vec3::new(0.0, 0.0, 1.0)),
        Transform::IDENTITY,
    ));
}
