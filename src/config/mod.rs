use crate::world::block_dictionary::initialize_dictionary;
use bevy::prelude::*;
use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use thiserror::Error;

pub const NUM_BLOCKS: u32 = 6;

pub const TILE_WIDTH: u32 = 28;
pub const TILE_HEIGHT: u32 = 28;
pub const HALF_TILE_WIDTH: u32 = TILE_WIDTH / 2;
pub const HALF_TILE_HEIGHT: u32 = TILE_HEIGHT / 2;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct ConfigSet;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_config_resources.in_set(ConfigSet));
    }
}

fn setup_config_resources(mut commands: Commands) {
    let config: Config = load_config("Config.toml").unwrap_or_else(|e| {
        eprintln!("Error loading config: {}", e);
        panic!();
    });

    commands.insert_resource(config);

    if let Err(e) = initialize_dictionary(Path::new("Blocks.toml")) {
        eprintln!("{:?}", e);
    }
}

#[must_use]
pub fn load_config(path: &str) -> Result<Config, CliError> {
    let contents: String = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Debug, Deserialize, Resource)]
pub struct Config {
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub game_mode: String,
    pub gravity_per_second: f32,
    pub friction_per_second: f32,
    pub player_speed: f32,
    pub jump_velocity: f32,
    pub camera_zoom_speed: f32,
    pub camera_decay_rate: f32,
}

#[derive(Debug, Deserialize)]
pub struct WorldConfig {
    pub generation: WorldGeneration,
    pub render_distance: u32,
    pub simulation_distance: u32,
    pub num_rotations_90_deg_clockwise: u8,
    pub target_hover_height: f32,
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

#[derive(Debug, Deserialize, Clone)]
pub struct WorldGeneration {
    pub world_mode: WorldMode,
    pub seed: u32,
    pub dirt_height: i32,
    pub grass_threshold: f64,
    pub lowest_surface_height: u32,
    pub highest_surface_height: u32,
    pub cave_threshold: f64,

    pub base_noise: NoiseParams,
    pub mountain_ridge_noise: NoiseParams,
    pub cave_noise: NoiseParams,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NoiseParams {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
}
