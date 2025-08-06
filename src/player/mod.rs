use crate::config::Config;
use crate::world::World;
use bevy::prelude::*;
use spriso::IsoProjection;
use terrain_data::prelude::{BlockPosition, ChunkPosition};

#[derive(Component)]
pub struct Player;

pub fn move_player(
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
    key_input: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
) {
    let mut direction: Vec2 = Vec2::ZERO;

    if key_input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if key_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let move_delta: Vec2 =
        direction.normalize_or_zero() * config.player.player_speed * time.delta_secs();
    player.translation += move_delta.extend(0.0);
}

pub fn player_chunk_pos(player_transform: &Transform, proj: &IsoProjection) -> ChunkPosition {
    let Vec3 { x, y, z: _ } = player_transform.translation;
    let world_pos: BlockPosition = proj.screen_to_world(glam::vec3(x, -y, 0.0));
    World::block_to_chunk_pos(world_pos)
}
