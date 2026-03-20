use crate::config::{TILE_H, TILE_W};
use bevy::prelude::*;
use spico::Projector;

pub struct ProjectionPlugin;

impl Plugin for ProjectionPlugin {
    fn build(&self, app: &mut App) {
        const HTW: u32 = TILE_W / 2;
        const HTH: u32 = TILE_H / 2;

        app.insert_resource(ProjectorRes(Projector::new::<HTW, HTH>()))
            .add_systems(Update, project_grid_to_screen);
    }
}

#[derive(Resource)]
pub struct ProjectorRes(pub Projector);

#[derive(Component)]
pub struct GridPosition(pub Vec3);

fn project_grid_to_screen(
    projector: Res<ProjectorRes>,
    mut query: Query<(&GridPosition, &mut Transform), Changed<GridPosition>>,
) {
    for (grid_pos, mut transform) in &mut query {
        let screen_pos = projector.0.grid_to_screen(grid_pos.0);
        transform.translation = vec3(screen_pos.x, screen_pos.z - screen_pos.y, 0.0);
    }
}
