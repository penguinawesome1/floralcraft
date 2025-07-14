use bevy::prelude::*;
use floralcraft::{
    config,
    config::{ Config, load_config },
    renderer,
    world_controller,
    world_controller::WorldController,
};
use floralcraft_terrain::ChunkPosition;
use isometric_projection::IsometricProjection;
use thiserror::Error;

#[derive(Resource)]
pub struct IsoProjWrapper(IsometricProjection);

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)] ConfigCliError(#[from] config::CliError),
    #[error(transparent)] BlockCliError(#[from] block_dictionary::CliError),
}

fn main() -> Result<(), CliError> {
    let config: Config = load_config("Config.toml")?;

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: String::from("Floralcraft"),
                    ..default()
                }),
                ..default()
            })
        )
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
    world_controller.update(&config.world.generation);
}

fn draw_blocks(
    mut commands: Commands,
    mut assets: ResMut<renderer::Assets>,
    asset_server: Res<AssetServer>,
    world_controller: Res<WorldController>,
    iso_proj: Res<IsoProjWrapper>,
    config: Res<Config>
) {
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = 0;

    let chunk_positions = world_controller::SizedWorld
        ::positions_in_square(origin, radius)
        .filter(|&pos| world_controller.world.is_chunk_at_pos(pos));

    let valid_block_pos = world_controller::SizedWorld
        ::coords_in_chunks(chunk_positions)
        .filter_map(|pos| unsafe {
            world_controller.world
                .block(pos)
                .ok()
                .map(|block| (pos, block))
        });

    let image_paths: Vec<(String, String)> = config.assets.blocks.clone().into_iter().collect();

    for (pos, block) in valid_block_pos {
        unsafe {
            match world_controller.world.block_exposed(pos) {
                Ok(is_exposed) => {
                    if !is_exposed {
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("Tried to print unloaded chunk! {:?}", e);
                    continue;
                }
            }
        }

        let screen_pos = iso_proj.0.world_to_screen(pos);

        commands.spawn((
            Sprite::from_image(
                assets.image(asset_server.clone(), image_paths[block as usize].1.as_str())
            ),
            Transform::from_xyz(screen_pos.x as f32, (-screen_pos.y + screen_pos.z) as f32, 0.0),
        ));
    }
}
