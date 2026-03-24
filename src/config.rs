use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

pub const TILE_W: u32 = 28;
pub const TILE_H: u32 = 28;

#[derive(Resource, Debug, Deserialize)]
pub struct Config {
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub player_speed: f32,
    pub camera_zoom_speed: f32,
    pub camera_decay_rate: f32,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    pub num_blocks: u32,
    pub render_distance: u32,
    pub mode: WorldMode,
}

#[derive(Debug, Deserialize, Clone)]
pub enum WorldMode {
    #[serde(rename = "flat")]
    Flat,
    #[serde(rename = "skyblock")]
    Skyblock,
    #[serde(other)]
    Normal,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ConfigSet;

pub struct ConfigPlugin {
    pub path: String,
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConfigPath(self.path.clone()))
            .add_systems(Startup, load_config.in_set(ConfigSet));
    }
}

#[derive(Resource)]
struct ConfigPath(String);

fn load_config(mut commands: Commands, path_res: Res<ConfigPath>) {
    let config_str = fs::read_to_string(&path_res.0)
        .expect(&format!("Could not read config file at {}", path_res.0));

    let config: Config =
        toml::from_str(&config_str).expect("Failed to parse the config toml. Check your syntax!");

    commands.insert_resource(config);
    info!("Config loaded successfully!");
}
