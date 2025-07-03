use bevy::prelude::*;
use floralcraft::{
    rendering::assets::{ Assets, ImageKey },
    terrain::block::BlockPosition,
    terrain::chunk::{ Chunk, ChunkPosition },
    terrain_management::WorldLogic,
    config::CONFIG,
    terrain::World,
};

fn main() {
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
        .add_systems(Startup, setup)
        .add_systems(Update, update_world_logic)
        .add_systems(Update, draw_blocks)
        .run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(WorldLogic::new());
    commands.insert_resource(Assets::new());
    commands.spawn(Camera2d);
    info!("Setup complete!");
}

fn draw_blocks(
    mut commands: Commands,
    mut assets: ResMut<Assets>,
    asset_server: Res<AssetServer>,
    world_logic: ResMut<WorldLogic>
) {
    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let chunk_positions = World::positions_in_square(origin, CONFIG.world.render_distance);
    for chunk_pos in chunk_positions {
        if !world_logic.world.is_chunk_at_pos(chunk_pos) {
            continue;
        }

        let chunk_block_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);

        for pos in Chunk::chunk_coords() {
            let world_pos: BlockPosition = chunk_block_pos + pos;
            if let Some(data) = world_logic.block_render_data(world_pos, None) {
                let texture: Handle<Image> = assets.image(
                    asset_server.clone(),
                    ImageKey::Block(data.block)
                );
                commands.spawn((
                    Sprite::from_image(texture),
                    Transform::from_xyz(data.pos.x as f32, -data.pos.y + data.pos.z, 0.0),
                ));
            }
        }
    }
}

fn update_world_logic(mut world_logic: ResMut<WorldLogic>) {
    world_logic.update(ChunkPosition::new(0, 0));
}
