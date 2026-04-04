mod camera;
pub mod config;
pub mod player;
pub mod position;
pub mod renderer;
pub mod world;

use bevy::prelude::*;
use config::{Config, ConfigHandle, ConfigPlugin};
use player::PlayerPlugin;
use position::ProjectionPlugin;
use renderer::{RendererPlugin, SpriteAssets};
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
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Floralcraft".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            ConfigPlugin {
                path: "config.toml".to_string(),
            },
            RendererPlugin,
            PlayerPlugin,
            ProjectionPlugin,
            ChunkLoaderPlugin,
        ))
        .init_state::<GameState>()
        .add_systems(
            Update,
            transition_to_playing
                .run_if(in_state(GameState::Loading))
                .run_if(resource_exists::<ConfigHandle>)
                .run_if(resource_exists::<SpriteAssets>),
        )
        .run();
}

fn transition_to_playing(
    config_handle: Res<ConfigHandle>,
    config_assets: Res<Assets<Config>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Some(_config) = config_assets.get(&config_handle.0) {
        next_state.set(GameState::Playing);
        info!("Config loaded and GameState transitioning to Playing.");
    }
}
