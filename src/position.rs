use crate::config::{HALF_TILE_H, HALF_TILE_W};
use bevy::prelude::*;
use spico::Projector;

pub struct ProjectionPlugin;

impl Plugin for ProjectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProjectorRes(Projector::new::<HALF_TILE_W, HALF_TILE_H>()))
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
        let order = grid_pos.0.element_sum() * 0.1;
        let screen_pos = projector.0.grid_to_screen(grid_pos.0);
        transform.translation = vec3(screen_pos.x, screen_pos.z - screen_pos.y, order);
    }
}
