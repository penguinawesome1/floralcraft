use bevy::prelude::*;
use floralcraft::{
    camera::update_camera,
    config::{
        Config, HALF_TILE_HEIGHT, HALF_TILE_WIDTH, NUM_BLOCKS, TILE_HEIGHT, TILE_WIDTH, WorldMode,
        load_config,
    },
    player::Player,
    player::move_player,
    renderer::{ImageMap, ResIsoProjection, handle_draw_tasks, make_draw_tasks},
    world::{
        DirtyChunks, ResWorld, World,
        block_dictionary::initialize_dictionary,
        block_generator::{FlatGenerator, NormalGenerator, SkyblockGenerator},
        chunk_generation::{
            ChunksBeingGenerated, ResGenerator, handle_chunk_tasks, make_chunk_tasks,
        },
    },
};
use spriso::IsoProjection;
use std::path::Path;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                move_player,
                update_camera,
                make_chunk_tasks,
                handle_chunk_tasks,
                make_draw_tasks,
                handle_draw_tasks,
            )
                .chain(),
        )
        .run();

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
) {
    if let Err(e) = initialize_dictionary(Path::new("Blocks.toml")) {
        eprintln!("{:?}", e);
    }

    commands.insert_resource(ResWorld(Arc::new(World::default())));
    commands.insert_resource(ResGenerator(match &config.world.generation.world_mode {
        WorldMode::Normal => Box::new(NormalGenerator::new(&config.world.generation)),
        WorldMode::Flat => Box::new(FlatGenerator),
        WorldMode::Skyblock => Box::new(SkyblockGenerator),
    }));
    commands.insert_resource(ResIsoProjection(Arc::new(IsoProjection::new::<
        HALF_TILE_WIDTH,
        HALF_TILE_HEIGHT,
    >())));
    commands.insert_resource(DirtyChunks::default());
    commands.insert_resource(ChunksBeingGenerated::default());
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("player/idle.png")),
    ));
    commands.insert_resource(ImageMap {
        image: asset_server.load("blocks.png"),
        layout: texture_atlases.add(TextureAtlasLayout::from_grid(
            UVec2::new(TILE_WIDTH, TILE_HEIGHT),
            NUM_BLOCKS,
            1,
            None,
            None,
        )),
    });

    info!("Finished setup");
}

// fn raycast_coords(pos: Vec2, screen_height: u32) -> impl Iterator<Item = BlockPosition> {
//     (0..screen_height as i32)
//         .rev()
//         .map(move |z| {
//             let screen_pos: Vec3 = Vec3::new(pos.x, pos.y + z as f32, z as f32);
// PROJECTION.screen_to_world(screen_pos)
//         })
//         .collect::<HashSet<BlockPosition>>()
//         .into_iter()
// }
