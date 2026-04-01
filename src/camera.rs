use crate::config::Config;
use crate::player::Player;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

type CameraData = (&'static mut Projection, &'static mut Transform);
type CameraFilter = (With<Camera2d>, Without<Player>);

#[derive(SystemParam)]
pub struct MainCamera<'w, 's> {
    pub query: Single<'w, 's, CameraData, CameraFilter>,
}

pub fn update_camera(
    mut camera: MainCamera,
    mouse_wheel: Res<AccumulatedMouseScroll>,
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let (projection, transform) = &mut *camera.query;

    let target_pos = player
        .translation
        .truncate()
        .extend(transform.translation.z);

    transform.translation.smooth_nudge(
        &target_pos,
        config.player.camera.decay_rate,
        time.delta_secs(),
    );

    if let Projection::Orthographic(ortho) = &mut **projection {
        let delta_zoom = -mouse_wheel.delta.y * config.player.camera.zoom_speed;
        let multiplicative_zoom = 1.0 + delta_zoom;

        ortho.scale = (ortho.scale * multiplicative_zoom).clamp(0.1, 10.0);
    }
}
