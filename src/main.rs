mod camera;
pub mod config;
pub mod player;
pub mod position;
pub mod renderer;
pub mod world;

use bevy::prelude::*;
use config::Config;
use config::ConfigPlugin;
use player::PlayerPlugin;
use position::ProjectionPlugin;
use renderer::RendererPlugin;
use renderer::SpriteAssets;
use world::chunk_loader::ChunkLoaderPlugin;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
}

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
            RendererPlugin,
            PlayerPlugin,
            ProjectionPlugin,
            ChunkLoaderPlugin,
        ))
        .init_state::<GameState>()
        .add_systems(
            Update,
            transition_to_playing.run_if(in_state(GameState::Loading)),
        )
        .run();
}

fn transition_to_playing(
    config: Option<Res<Config>>,
    sprite_assets: Option<Res<SpriteAssets>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if config.is_some() && sprite_assets.is_some() {
        next_state.set(GameState::Playing);
        info!("System Ready: Transitioning to Playing");
    }
}
