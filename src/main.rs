use macroquad::prelude::*;
use floralcraft::game::player::Player;
use floralcraft::game::physics::{ Position3D, Position2D };
use floralcraft::rendering::Renderer;
use floralcraft::terrain::chunk::{ ChunkPosition, Chunk };
use floralcraft::terrain_management::{ WorldLogic, MouseTargets };
use floralcraft::config::CONFIG;

#[macroquad::main("Floralcraft")]
async fn main() {
    let mut camera: Camera2D = Camera2D::default();
    let mut world_logic: WorldLogic = WorldLogic::new();
    let mut player: Player = Player::new();
    let mut renderer: Renderer = Renderer::new().await.expect("failed to initialize assets");

    loop {
        clear_background(BLACK);

        let delta_time: f32 = get_frame_time();

        // player update
        player.update(delta_time);
        let origin: ChunkPosition = player.get_chunk_pos();
        let player_pos: Position3D = player.hitbox.pos;

        // inputs
        let zoom: f32 = CONFIG.player.camera_zoom;
        let mouse_pos_vec: Vec2 = camera.screen_to_world(mouse_position().into());
        let mouse_pos: Position2D = Position2D::new(mouse_pos_vec.x, mouse_pos_vec.y);
        let is_left_click: bool = is_mouse_button_pressed(MouseButton::Left);
        let is_right_click: bool = is_mouse_button_pressed(MouseButton::Right);

        // world update
        world_logic.update(
            origin,
            CONFIG.world.render_distance as i32,
            mouse_pos,
            is_left_click,
            is_right_click
        );

        // camera updates
        camera.zoom = Vec2::new(zoom / screen_width(), zoom / screen_height());
        camera.target = Vec2::new(player_pos.x, player_pos.y - player_pos.z);
        set_camera(&camera);

        // rendering
        let radius: i32 = CONFIG.world.render_distance as i32;
        let chunks: Vec<&Chunk> = world_logic.world.chunks_in_square(origin, radius).collect();
        let targets: Option<MouseTargets> = world_logic.find_mouse_targets(mouse_pos);
        renderer.update(&player, &chunks, &world_logic, targets).await;

        set_default_camera();
        next_frame().await;
    }
}
