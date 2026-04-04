use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy_common_assets::toml::TomlAssetPlugin;
use lattice::BlockGenParams;
use serde::Deserialize;

pub const TILE_W: u32 = 28;
pub const TILE_H: u32 = 28;
pub const HALF_TILE_W: u32 = TILE_W / 2;
pub const HALF_TILE_H: u32 = TILE_H / 2;

pub struct ConfigPlugin {
    pub path: String,
}

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConfigPath(self.path.clone()))
            .add_plugins(TomlAssetPlugin::<Config>::new(&["toml"]))
            .add_systems(Startup, load_config_asset)
            .add_systems(Update, sync_config_to_resource);
    }
}

fn sync_config_to_resource(
    mut commands: Commands,
    mut message: MessageReader<AssetEvent<Config>>,
    config_assets: Res<Assets<Config>>,
    config_handle: Res<ConfigHandle>,
) {
    for ev in message.read() {
        if !ev.is_loaded_with_dependencies(&config_handle.0) {
            return;
        }

        let Some(config) = config_assets.get(&config_handle.0) else {
            return;
        };

        commands.insert_resource(config.clone());
        info!("Config resource updated");
    }
}

#[derive(Resource)]
struct ConfigPath(String);

#[derive(Resource)]
pub struct ConfigHandle(pub Handle<Config>);

fn load_config_asset(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    path: Res<ConfigPath>,
) {
    let handle = asset_server.load(&path.0);
    commands.insert_resource(ConfigHandle(handle));
}

#[derive(Asset, TypePath, Debug, Deserialize, Clone, Resource)]
pub struct Config {
    pub player: PlayerConfig,
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerConfig {
    pub speed: f32,
    pub camera: CameraConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CameraConfig {
    pub zoom_speed: f32,
    pub decay_rate: f32,
}

#[derive(Debug, Deserialize, Clone)]
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
