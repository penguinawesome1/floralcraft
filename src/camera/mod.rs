use crate::{config::Config, player::Player};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub fn update_camera(
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut evr_scroll: EventReader<MouseWheel>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
    config: Res<Config>,
) {
    let Vec3 { x, y, .. } = player.translation;
    let direction: Vec3 = Vec3::new(x, y, camera.translation.z);

    camera.translation.smooth_nudge(
        &direction,
        config.player.camera_decay_rate,
        time.delta_secs(),
    );

    for ev in evr_scroll.read() {
        let zoom_amount: f32 = match ev.unit {
            MouseScrollUnit::Line => config.player.camera_zoom_speed,
            MouseScrollUnit::Pixel => config.player.camera_zoom_speed * 0.1,
        };

        let scroll_direction: f32 = ev.y.signum();
        let curr_scale: Vec3 = camera.scale;

        camera.scale -= scroll_direction * zoom_amount * curr_scale;
    }
}
