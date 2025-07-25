use crate::config::Config;
use bevy::prelude::*;

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
