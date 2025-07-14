use serde::Deserialize;
use thiserror::Error;
use std::io;
use std::fs;
use indexmap::IndexMap;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")] IoError(#[from] io::Error),
    #[error("TOML deserialization error: {0}")] TomlDeError(#[from] toml::de::Error),
}

#[must_use]
pub fn load_config(path: &str) -> Result<Config, CliError> {
    let contents: String = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

#[derive(Debug, Deserialize, bevy::prelude::Resource)]
pub struct Config {
    pub assets: AssetsConfig,
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize)]
pub struct AssetsConfig {
    pub player: IndexMap<String, String>,
    pub blocks: IndexMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerConfig {
    pub game_mode: String,
    pub gravity_per_second: f32,
    pub friction_per_second: f32,
    pub acceleration_per_second: f32,
    pub jump_velocity: f32,
    pub stop_threshold: f32,
    pub camera_zoom: f32,
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
    pub minimum_air_height: u32,
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
