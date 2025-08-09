use crate::config::TILE_HEIGHT;
use crate::world::CHUNK_DEPTH;
use crate::{
    renderer::ResIsoProjection,
    world::{ResWorld, block_dictionary::definition},
};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use spriso::IsoProjection;
use std::collections::HashSet;
use terrain_data::prelude::BlockPosition;

#[derive(Resource, Default)]
pub struct HoverBlock(pub Option<(BlockPosition, BlockPosition)>);

pub fn update_hover_block(
    mut hover_block: ResMut<HoverBlock>,
    camera: Single<(&Camera, &GlobalTransform)>,
    world: Res<ResWorld>,
    windows: Query<&Window, With<PrimaryWindow>>,
    proj: Res<ResIsoProjection>,
) {
    hover_block.0 = None;

    let Ok(window) = windows.single() else {
        return;
    };
    let (camera, camera_transform) = *camera;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(global_cursor_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    let window_height: u32 = CHUNK_DEPTH as u32 * TILE_HEIGHT / 2;

    let mut gap_pos: Option<BlockPosition> = None;

    for pos in raycast_coords(&proj.0, global_cursor_pos, window_height) {
        let Ok(block) = world.0.block(pos) else {
            continue;
        };

        if !definition(block as usize).is_hoverable() {
            gap_pos = Some(pos);
        } else if let Some(gp) = gap_pos {
            hover_block.0 = Some((gp, pos));
            return;
        }
    }
}

fn raycast_coords(
    proj: &IsoProjection,
    pos: Vec2,
    screen_height: u32,
) -> impl Iterator<Item = BlockPosition> {
    let mut seen: HashSet<BlockPosition> = HashSet::new();

    (0..screen_height).rev().step_by(2).filter_map(move |z| {
        let screen_pos: glam::Vec3 = glam::Vec3::new(pos.x, -pos.y + z as f32, z as f32);
        let world_pos: BlockPosition = proj.screen_to_world(screen_pos);
        if seen.insert(world_pos) {
            Some(world_pos)
        } else {
            None
        }
    })
}
