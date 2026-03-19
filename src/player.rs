use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

pub fn new(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Transform::from_xyz(1.0, 2.0, 3.0),
        GlobalTransform::default(),
        Sprite::from_image(asset_server.load("player/idle.png")),
    ));
}
