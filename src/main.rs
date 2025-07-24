use bevy::prelude::*;
use floralcraft::{
    config,
    config::{Config, load_config},
    renderer, world_controller,
    world_controller::WorldController,
};
use floralcraft_terrain::ChunkPosition;
use riso::IsometricProjection;
use thiserror::Error;

#[derive(Resource)]
pub struct IsoProjWrapper(IsometricProjection);

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    ConfigCliError(#[from] config::CliError),
    #[error(transparent)]
    BlockCliError(#[from] block_dictionary::CliError),
}

fn main() -> Result<(), CliError> {
    let config: Config = load_config("Config.toml")?;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Floralcraft"),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(config)
        .add_systems(Startup, setup)
        .add_systems(Update, update_world_controller)
        .add_systems(Update, draw_blocks)
        .run();

    Ok(())
}

fn setup(mut commands: Commands, config: Res<Config>) {
    commands.insert_resource(IsoProjWrapper(IsometricProjection::new::<14, 14>()));
    commands.insert_resource(renderer::Assets::default());
    commands.insert_resource(WorldController::new(&config.world.generation));
    commands.spawn(Camera2d);

    info!("Setup complete!");
}

fn update_world_controller(mut world_controller: ResMut<WorldController>, config: Res<Config>) {
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = config.world.render_distance;
    world_controller.update(&config.world.generation, origin, radius);
}

fn draw_blocks(
    mut commands: Commands,
    mut assets: ResMut<renderer::Assets>,
    mut world_controller: ResMut<WorldController>,
    asset_server: Res<AssetServer>,
    iso_proj: Res<IsoProjWrapper>,
    config: Res<Config>,
) {
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = config.world.render_distance;
    let image_paths: Vec<String> = config
        .assets
        .blocks
        .clone()
        .into_iter()
        .map(|(_name, path)| path)
        .collect();

    let chunk_positions: Vec<ChunkPosition> =
        world_controller::SizedWorld::positions_in_square(origin, radius)
            .filter(|&pos| world_controller.world.is_chunk_at_pos(pos))
            .collect();

    for chunk_pos in chunk_positions {
        let origin_block_pos: glam::IVec3 =
            world_controller::SizedWorld::chunk_to_block_pos(chunk_pos);

        let _ = world_controller
            .world
            .decorate_chunk(chunk_pos, |chunk, pos| {
                if unsafe { !chunk.block_exposed(pos) } {
                    return;
                }

                let block: u8 = unsafe { chunk.block(pos) };
                let screen_pos: glam::Vec3 = iso_proj.0.world_to_screen(origin_block_pos + pos);
                let texture: Handle<Image> =
                    assets.image(asset_server.clone(), &image_paths[block as usize]);
                let transform: Transform = Transform::from_xyz(
                    screen_pos.x as f32,
                    (-screen_pos.y + screen_pos.z) as f32,
                    0.0,
                );

                commands.spawn((Sprite::from_image(texture), transform));
            })
            .map_err(|e| eprintln!("Tried to access unloaded chunk! {:?}", e));
    }
}
