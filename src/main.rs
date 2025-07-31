use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use floralcraft::{
    config,
    config::{Config, load_config},
    player::Player,
    renderer,
    renderer::DrawContext,
    renderer::{ImageMap, IsoProj},
    world_controller::block_dictionary::initialize_dictionary,
    world_controller::make_chunk,
    world_controller::{WorldController, world::World as FloralWorld},
};
use spriso::IsoProjection;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use terrain_data::prelude::AccessError;
use terrain_data::prelude::ChunkPosition;

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
                floralcraft::player::move_player,
                floralcraft::camera::update_camera,
                make_chunk_tasks,
                draw,
                handle_chunk_tasks,
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

    if let Err(e) = initialize_dictionary(Path::new("Blocks.toml")) {
        eprintln!("{:?}", e);
    }

    commands.insert_resource(WorldController::new(&config.world.generation));
    commands.insert_resource(IsoProj(IsoProjection::new::<14, 14>()));
    commands.insert_resource(DirtyChunks::default());
    commands.insert_resource(ChunksBeingGenerated::default());
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

    info!("Finished setup");
}

#[derive(Default, Resource)]
struct DirtyChunks(pub HashSet<ChunkPosition>);

#[derive(Default, Resource)]
struct ChunksBeingGenerated(pub HashSet<ChunkPosition>);

#[derive(Component)]
#[allow(dead_code)]
struct ChunkGenerationTask(pub Task<Result<Option<ChunkPosition>, AccessError>>);

fn make_chunk_tasks(
    mut commands: Commands,
    world_controller: Res<WorldController>,
    config: Res<Config>,
    mut chunks_being_generated: ResMut<ChunksBeingGenerated>,
) {
    let compute_pool = AsyncComputeTaskPool::get();

    let params: &config::WorldGeneration = &config.world.generation;
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let radius: u32 = config.world.render_distance;
    let positions = FloralWorld::positions_in_square(origin, radius);

    for chunk_pos in positions {
        if world_controller.world.is_chunk_at_pos(chunk_pos)
            || chunks_being_generated.0.contains(&chunk_pos)
        {
            continue;
        }

        let world_clone: Arc<FloralWorld> = Arc::clone(&world_controller.world);
        let generator_clone = world_controller.generator.clone_box();
        let params_clone = params.clone();

        let task = compute_pool.spawn(async move {
            make_chunk(world_clone, generator_clone, chunk_pos, params_clone).await
        });

        commands.spawn(ChunkGenerationTask(task));
        chunks_being_generated.0.insert(chunk_pos);
    }
}

fn handle_chunk_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkGenerationTask)>,
    mut chunks_being_generated: ResMut<ChunksBeingGenerated>,
    mut dirty_chunks: ResMut<DirtyChunks>,
) {
    for (entity, mut task) in &mut query {
        if let Some(result) = bevy::tasks::futures_lite::future::block_on(
            bevy::tasks::futures_lite::future::poll_once(&mut task.0),
        ) {
            match result {
                Ok(Some(chunk_pos)) => {
                    dirty_chunks.0.insert(chunk_pos);
                    chunks_being_generated.0.remove(&chunk_pos);
                    commands.entity(entity).despawn();
                }
                Ok(None) => {
                    commands.entity(entity).despawn();
                }
                Err(e) => {
                    eprintln!("Error generating chunk: {:?}", e);
                    commands.entity(entity).despawn();
                }
            }
        }
    }
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

// pub fn try_add_chunk(
//     &self,
//     params: &WorldGeneration,
//     chunk_pos: ChunkPosition,
// ) -> Result<Option<ChunkPosition>, AccessError> {
//     if self.world.add_empty_chunk(chunk_pos).is_err() {
//         return Ok(None); // return if chunk already exists
//     }

//     let mut chunk = self.world.chunk_mut(chunk_pos)?;

//     let origin_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

//     for pos in World::chunk_coords(ChunkPosition::ZERO) {
//         let block: u8 = self.generator.choose_block(origin_block_pos + pos, params);
//         chunk.value_mut().set_block(pos, block)?;
//     }

//     drop(chunk);

//     self.update_exposed_blocks(chunk_pos)?;

//     Ok(Some(chunk_pos))
// }
