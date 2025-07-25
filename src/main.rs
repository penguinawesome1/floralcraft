use bevy::prelude::*;
use floralcraft::{
    config,
    config::{Config, load_config},
    player::Player,
    renderer,
    renderer::{DrawContext, ImageMap, IsoProj},
    world_controller::{DirtyChunks, SizedWorld, WorldController},
};
use floralcraft_terrain::ChunkPosition;
use spriso::IsoProjection;

fn main() -> Result<(), config::CliError> {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Floralcraft".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(load_config("Config.toml")?)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_world_controller,
                draw,
                floralcraft::player::move_player,
                floralcraft::camera::update_camera,
            )
                .chain(),
        )
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    const TILE_WIDTH: u32 = 28;

    commands.insert_resource(WorldController::new(&config.world.generation));
    commands.insert_resource(IsoProj(IsoProjection::new::<14, 14>()));
    commands.insert_resource(DirtyChunks::default());
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("player/idle.png")),
    ));
    commands.insert_resource(ImageMap {
        texture: asset_server.load("blocks.png"),
        layout: texture_atlases.add(TextureAtlasLayout::from_grid(
            UVec2::splat(TILE_WIDTH),
            5,
            1,
            None,
            None,
        )),
    });
}

fn update_world_controller(
    mut world_controller: ResMut<WorldController>,
    config: Res<Config>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    let params: &config::WorldGeneration = &config.world.generation;
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = config.world.render_distance;
    let positions = SizedWorld::positions_in_square(origin, radius);
    world_controller.update(params, &mut dirty_chunks, positions);
}

fn draw(
    mut draw_context: DrawContext,
    world_controller: Res<WorldController>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    for chunk_pos in dirty_chunks.0.drain() {
        renderer::draw_chunk(&mut draw_context, &world_controller, chunk_pos);
    }
}
