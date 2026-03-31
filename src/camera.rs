use crate::config::Config;
use crate::player::Player;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

pub fn update_camera(
    camera: Single<(&mut Projection, &mut Transform), (With<Camera2d>, Without<Player>)>,
    mouse_wheel: Res<AccumulatedMouseScroll>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let (mut projection, mut cam_trans) = camera.into_inner();

    let target_pos = player
        .translation
        .truncate()
        .extend(cam_trans.translation.z);
    cam_trans.translation.smooth_nudge(
        &target_pos,
        config.player.camera.decay_rate,
        time.delta_secs(),
    );

    if let Projection::Orthographic(ref mut ortho) = *projection {
        let delta_zoom = -mouse_wheel.delta.y * config.player.camera.zoom_speed;
        let multiplicative_zoom = 1.0 + delta_zoom;

        ortho.scale = (ortho.scale * multiplicative_zoom).clamp(0.1, 10.0);
    }
}
