use crate::position::GridPosition;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player);
        app.add_systems(Update, apply_gravity);
    }
}

pub type PlayerMovedFilter = (
    With<Player>,
    Or<(Added<GridPosition>, Changed<GridPosition>)>,
);

#[derive(Component)]
pub struct Player;

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        GridPosition(Vec3::default()),
        Sprite::from_image(asset_server.load("player/idle.png")),
        Transform::from_scale(Vec3::splat(0.1)),
        Visibility::default(),
    ));
}

fn apply_gravity(mut query: Query<&mut GridPosition, With<Player>>) {
    for mut pos in &mut query {
        pos.0.z -= 0.01;
    }
}
