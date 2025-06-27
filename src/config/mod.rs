use serde::Deserialize;
use std::error::Error;
use std::fs::read_to_string;
use std::collections::HashMap;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    load_config().expect("Failed to load config")
});

fn load_config() -> Result<Config, Box<dyn Error>> {
    let toml_str = read_to_string("Config.toml").map_err(|e|
        format!("Could not read Config.toml: {}", e)
    )?;
    let config: Config = toml
        ::from_str(&toml_str)
        .map_err(|e| format!("Could not parse Config.toml: {}", e))?;

    Ok(config)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub assets: AssetsConfig,
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize)]
pub struct AssetsConfig {
    pub player: HashMap<String, String>,
    pub blocks: HashMap<String, String>,
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
pub struct WorldGeneration {
    pub world_mode: String,
    pub seed: u32,
    pub dirt_height: i32,
    pub grass_threshold: f64,
    pub minimum_air_height: u32,
    pub cave_threshold: f64,

    pub base_noise: NoiseParams,
    pub mountain_ridge_noise: NoiseParams,
    pub cave_noise: NoiseParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoiseParams {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
}
