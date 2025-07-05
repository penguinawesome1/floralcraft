use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use floralcraft::{
    config::CONFIG,
    rendering::assets::{ Assets, ImageKey },
    terrain::{ chunk::ChunkPosition, World },
    terrain_management::{ WorldLogic, MouseTargets },
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

// fn get_target_solid(
//     q_window: Query<&Window, With<PrimaryWindow>>,
//     world_logic: &ResMut<WorldLogic>
// ) -> Option<glam::IVec3> {
//     let window: &Window = q_window.single().unwrap();
//     let mouse_pos_bevy: Vec2 = window.cursor_position()?;
//     let mouse_pos: glam::Vec2 = glam::Vec2::new(mouse_pos_bevy.x, mouse_pos_bevy.y);
//     let screen_height: u32 = window.physical_height();
//     let mouse_targets: MouseTargets = world_logic.find_mouse_targets(mouse_pos, screen_height)?;
//     Some(mouse_targets.solid)
// }

fn draw_blocks(
    mut commands: Commands,
    mut assets: ResMut<Assets>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    world_logic: ResMut<WorldLogic>
) {
    // let target_solid: Option<glam::IVec3> = get_target_solid(q_window, &world_logic);

    let origin: ChunkPosition = ChunkPosition::new(0, 0);
    let chunk_positions = World::positions_in_square(origin, CONFIG.world.render_distance).filter(
        |&chunk_pos| world_logic.world.is_chunk_at_pos(chunk_pos)
    );

    for world_pos in World::global_coords_in_chunks(chunk_positions) {
        let Some(data) = world_logic.block_render_data(world_pos, None) else {
            continue; // continue if no block render data accessed
        };

        let texture: Handle<Image> = assets.image(
            asset_server.clone(),
            ImageKey::Block(data.block)
        );
        let hover: f32 = (data.is_target as i32 as f32) * (CONFIG.world.target_hover_height as f32);

        commands.spawn((
            Sprite::from_image(texture),
            Transform::from_xyz(data.pos.x as f32, -data.pos.y + data.pos.z + hover, 0.0),
        ));
    }
}

fn update_world_logic(mut world_logic: ResMut<WorldLogic>) {
    world_logic.update(ChunkPosition::new(0, 0));
}
