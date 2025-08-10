use crate::config::Config;
use crate::renderer::ResIsoProjection;
use crate::world::{ResWorld, block_dictionary::definition};
use bevy::prelude::*;
use terrain_data::prelude::BlockPosition;

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerWorldPos(pub glam::Vec3);

#[derive(Resource, Default)]
pub struct PlayerWorldVel(pub glam::Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_player, setup_player_resources).chain())
            .add_systems(Update, (move_player, prevent_player_collision).chain());
    }
}

fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("player/idle.png")),
    ));
}

fn setup_player_resources(mut commands: Commands) {
    commands.insert_resource(PlayerWorldPos(glam::vec3(0.0, 0.0, 99.0)));
    commands.insert_resource(PlayerWorldVel::default());
}

fn move_player(
    mut player: Single<&mut Transform, With<Player>>,
    mut player_world_pos: ResMut<PlayerWorldPos>,
    mut player_world_vel: ResMut<PlayerWorldVel>,
    time: Res<Time>,
    key_input: Res<ButtonInput<KeyCode>>,
    config: Res<Config>,
    proj: Res<ResIsoProjection>,
) {
    // move player

    player_world_pos.0 += player_world_vel.0;
    let glam::Vec3 { x, y, z } = proj.0.world_float_to_screen(player_world_pos.0);
    player.translation = vec3(x, z - y, 99.0);

    // friction

    let friction: f32 = config.player.friction_per_second * time.delta_secs();
    player_world_vel.0.x *= friction;
    player_world_vel.0.y *= friction;

    // walking

    let mut direction: glam::Vec2 = glam::Vec2::ZERO;

    if key_input.pressed(KeyCode::KeyW) {
        direction.x -= 1.0;
        direction.y -= 1.0;
    }
    if key_input.pressed(KeyCode::KeyS) {
        direction.x += 1.0;
        direction.y += 1.0;
    }
    if key_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
        direction.y += 1.0;
    }
    if key_input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
        direction.y -= 1.0;
    }

    let vel_delta: glam::Vec2 =
        direction.normalize_or_zero() * config.player.player_speed * time.delta_secs();
    player_world_vel.0 += vel_delta.extend(0.0);

    // gravity

    player_world_vel.0.z -= config.player.gravity_per_second * time.delta_secs();

    // jumping

    if key_input.just_pressed(KeyCode::Space) {
        player_world_vel.0.z = config.player.jump_velocity;
    }
}

fn prevent_player_collision(
    mut player_world_pos: ResMut<PlayerWorldPos>,
    mut player_world_vel: ResMut<PlayerWorldVel>,
    world: Res<ResWorld>,
) {
    let player_block_pos: BlockPosition = player_world_pos.0.as_ivec3();

    let plus_offsets: [glam::IVec3; 3] = [
        glam::ivec3(1, 0, 0),
        glam::ivec3(0, 1, 0),
        glam::ivec3(0, 0, 1),
    ];

    for i in 0..3 {
        if player_world_vel.0[i] > 0.0 {
            if let Ok(block) = world.0.block(player_block_pos + plus_offsets[i]) {
                if definition(block as usize).is_collidable() {
                    player_world_pos.0[i] = player_world_pos.0[i].trunc();
                    player_world_vel.0[i] = 0.0;
                }
            }
        } else if player_world_vel.0[i] < 0.0 {
            if let Ok(block) = world.0.block(player_block_pos - plus_offsets[i]) {
                if definition(block as usize).is_collidable() {
                    player_world_pos.0[i] = player_world_pos.0[i].trunc();
                    player_world_vel.0[i] = 0.0;
                }
            }
        }
    }
}
