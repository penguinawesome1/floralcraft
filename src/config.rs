use bevy::prelude::*;
use lattice::BlockGenParams;
use serde::Deserialize;
use std::fs;

pub const TILE_W: u32 = 28;
pub const TILE_H: u32 = 28;
pub const HALF_TILE_W: u32 = TILE_W / 2;
pub const HALF_TILE_H: u32 = TILE_H / 2;

use bevy::prelude::Resource;

#[derive(Resource, Debug, Deserialize)]
pub struct Config {
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub speed: f32,
    pub camera: CameraConfig,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub zoom_speed: f32,
    pub decay_rate: f32,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    pub mode: WorldMode,
    pub render_distance: u32,
    pub terrain: TerrainConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TerrainConfig {
    pub noise_profile: String,
    pub seed: i32,
    pub scale: f32,
    pub min_depth: u32,
    pub max_depth: u32,
    pub dirt_depth: u32,
}

impl From<&TerrainConfig> for BlockGenParams {
    fn from(config: &TerrainConfig) -> Self {
        Self {
            seed: config.seed,
            scale: config.scale,
            min_depth: config.min_depth,
            max_depth: config.max_depth,
            dirt_depth: config.dirt_depth,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorldMode {
    #[default]
    Normal,
    Flat,
    Skyblock,
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
