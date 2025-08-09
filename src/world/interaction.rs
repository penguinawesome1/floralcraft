use crate::renderer::ChunksToRender;
use crate::world::{
    ResWorld, World,
    block_dictionary::{SnugType, definition},
    hover_block::HoverBlock,
};
use bevy::prelude::*;
use terrain_data::prelude::ChunkPosition;

pub fn break_and_place(
    mut chunks_to_render: ResMut<ChunksToRender>,
    world: Res<ResWorld>,
    hover_block: Res<HoverBlock>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    let Some((gap_pos, pos)) = hover_block.0 else {
        return;
    };

    let gap_block: SnugType = world.0.block(gap_pos).unwrap();
    let block: SnugType = world.0.block(pos).unwrap();

    if mouse_buttons.just_pressed(MouseButton::Left) && definition(block as usize).is_breakable() {
        world.0.set_block(pos, 0).unwrap();
    } else if mouse_buttons.just_pressed(MouseButton::Right)
        && definition(gap_block as usize).is_replaceable()
    {
        world.0.set_block(gap_pos, 2).unwrap();
    } else {
        return;
    }

    for update_pos in World::block_offsets(pos).chain([pos]) {
        let Ok(block) = world.0.block(update_pos) else {
            continue;
        };

        let is_exposed: bool = definition(block as usize).is_visible()
            && World::block_offsets(update_pos).any(|adj_pos| match world.0.block(adj_pos) {
                Ok(adj_block) => definition(adj_block as usize).is_transparent(),
                _ => false,
            });

        let _ = world.0.set_is_exposed(update_pos, is_exposed);
    }

    let chunk_pos: ChunkPosition = World::block_to_chunk_pos(pos);
    chunks_to_render.0.push(chunk_pos);
}
