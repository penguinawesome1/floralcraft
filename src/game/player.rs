use bevy::prelude::*;
use glam::Vec3;
use crate::game::physics::Hitbox3D;
use crate::terrain::chunk::ChunkPosition;
use crate::terrain::World;
use crate::config::CONFIG;
use crate::rendering::isometric_projection::PROJECTION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerFrameKey {
    Idle,
    Run,
    Missing,
}

impl PlayerFrameKey {
    /// Takes input string and returns its corresponding key.
    /// Used to take config inputs and convert into keys for rendering images.
    pub fn from_string(s: &str) -> Self {
        match s {
            "idle" => Self::Idle,
            "run" => Self::Run,
            _ => Self::Missing,
        }
    }
}

#[derive(Component)]
pub struct Player {
    pub image_name: PlayerFrameKey,
    pub hitbox: Hitbox3D,
    pub velocity: Vec3,
}

impl Player {
    /// Gets the position of player converted to chunk position.
    ///
    /// # Examples
    ///
    /// ```
    /// use floralcraft::game::physics::Position3D;
    /// use floralcraft::game::player::Player;
    /// use floralcraft::terrain::{ ChunkPosition, CHUNK_WIDTH };
    ///
    /// let mut player: Player = Player::new();
    ///
    /// player.hitbox.pos = Position3D::new((CHUNK_WIDTH as f32) + 1.0, 0.0, 0.0);
    ///
    /// let chunk_pos: ChunkPosition = player.get_chunk_pos();
    ///
    /// assert_eq!(chunk_pos, ChunkPosition::new(0, -1));
    /// ```
    pub fn get_chunk_pos(&self) -> ChunkPosition {
        World::block_to_chunk_pos(PROJECTION.screen_to_world(self.hitbox.pos))
    }

    /// Progresses the physics of player by one frame relative to time passed.
    pub fn update(&mut self, delta_time: f32) {
        self.apply_gravity(delta_time);
        self.update_velocity(delta_time);
        self.apply_friction(delta_time);
        self.update_position(delta_time);
    }

    fn update_position(&mut self, delta_time: f32) {
        self.hitbox.pos.x += self.velocity.x * delta_time;
        self.hitbox.pos.y += self.velocity.y * delta_time * 0.5;
        self.hitbox.pos.z += self.velocity.z * delta_time;
    }

    fn apply_gravity(&mut self, delta_time: f32) {
        let gravity: f32 = CONFIG.player.gravity_per_second * delta_time;
        self.velocity.z -= gravity;
    }

    fn apply_friction(&mut self, delta_time: f32) {
        let friction: f32 = CONFIG.player.friction_per_second.powf(delta_time);

        self.velocity.x *= friction;
        self.velocity.y *= friction;

        if self.velocity.length().abs() < CONFIG.player.stop_threshold {
            self.velocity = Vec3::ZERO;
        }
    }

    fn update_velocity(&mut self, delta_time: f32) {
        let accel: f32 = CONFIG.player.acceleration_per_second * delta_time;

        let mut d_vel: Vec3 = Vec3::ZERO;

        if is_key_down(KeyCode::Up) {
            d_vel.y -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            d_vel.y += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            d_vel.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) {
            d_vel.x += 1.0;
        }

        if d_vel.length_squared() > f32::EPSILON {
            self.velocity += d_vel.normalize() * accel;
        }

        if is_key_pressed(KeyCode::Space) {
            self.velocity.z = CONFIG.player.jump_velocity;
        }

        if self.velocity.x != 0.0 && self.velocity.y != 0.0 {
            // self.image_name = PlayerFrameKey::Run;
        } else {
            self.image_name = PlayerFrameKey::Idle;
        }
    }
}
