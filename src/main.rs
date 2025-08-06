use bevy::prelude::*;
use floralcraft::{
    camera::update_camera,
    config::{
        Config, HALF_TILE_HEIGHT, HALF_TILE_WIDTH, NUM_BLOCKS, TILE_HEIGHT, TILE_WIDTH, WorldMode,
        load_config,
    },
    player::{Player, move_player, player_chunk_pos},
    renderer::{
        ChunkMaterial, DrawTaskPool, ImageMap, ResIsoProjection, handle_draw_tasks, make_draw_tasks,
    },
    world::{
        DirtyChunks, ResWorld, World,
        block_dictionary::initialize_dictionary,
        block_generator::{FlatGenerator, NormalGenerator, SkyblockGenerator},
        chunk_generation::{
            ChunkTaskPool, ChunksTasksInTransit, PendingChunks, ResGenerator, handle_chunk_tasks,
            make_chunk_tasks,
        },
    },
};
use spriso::IsoProjection;
use std::path::Path;
use std::sync::Arc;
use terrain_data::prelude::ChunkPosition;

fn main() {
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
        .add_systems(Startup, (load_configs, load_assets, load_resources).chain())
        .add_systems(
            Update,
            (
                choose_pending_chunks,
                make_chunk_tasks,
                handle_chunk_tasks,
                move_player,
                update_camera,
                make_draw_tasks,
                handle_draw_tasks,
            )
                .chain(),
        )
        .run();
}

fn choose_pending_chunks(
    mut pending_chunks: ResMut<PendingChunks>,
    config: Res<Config>,
    player: Single<&mut Transform, With<Player>>,
    proj: Res<ResIsoProjection>,
) {
    let origin: ChunkPosition = player_chunk_pos(&player, &proj.0);
    let radius: u32 = config.world.render_distance;
    let positions = World::positions_in_square(origin, radius);

    pending_chunks.0.extend(positions);
}

fn load_configs(mut commands: Commands) {
    commands.insert_resource(load_config("Config.toml").unwrap_or_else(|e| {
        eprintln!("Error loading config: {}", e);
        panic!();
    }));

    if let Err(e) = initialize_dictionary(Path::new("Blocks.toml")) {
        eprintln!("{:?}", e);
    }
}

fn load_assets(
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
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

    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("player/idle.png")),
    ));
}

fn load_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Config>,
    image_map: Res<ImageMap>,
) {
    commands.insert_resource(ChunkTaskPool::default());
    commands.insert_resource(ChunksTasksInTransit::default());
    commands.insert_resource(DrawTaskPool::default());
    commands.insert_resource(PendingChunks::default());
    commands.insert_resource(DirtyChunks::default());
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
    commands.insert_resource(ChunkMaterial(
        materials.add(ColorMaterial::from(image_map.image.clone())),
    ));

    commands.spawn(Camera2d);
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
