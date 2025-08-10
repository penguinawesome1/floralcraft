use crate::renderer::ChunksToRender;
use crate::world::{
    ResWorld, World,
    block_dictionary::{SnugType, definition},
    hover_block::HoverBlock,
};
use bevy::prelude::*;
use terrain_data::prelude::{BlockPosition, ChunkPosition};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, break_and_place);
    }
}

fn break_and_place(
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

    let affected_pos: Option<BlockPosition> =
        handle_block_breaking(&world.0, &mouse_buttons, pos, block)
            .or_else(|| handle_block_placing(&world.0, &mouse_buttons, gap_pos, gap_block));

    let Some(change_pos) = affected_pos else {
        return;
    };

    update_surrounding_exposed(&world.0, change_pos);

    let chunk_pos: ChunkPosition = World::block_to_chunk_pos(change_pos);
    chunks_to_render.0.push(chunk_pos);
}

fn handle_block_breaking(
    world: &World,
    mouse_buttons: &ButtonInput<MouseButton>,
    pos: BlockPosition,
    block: SnugType,
) -> Option<BlockPosition> {
    if mouse_buttons.just_pressed(MouseButton::Left) && definition(block as usize).is_breakable() {
        world.set_block(pos, 0).unwrap();
        Some(pos)
    } else {
        None
    }
}

fn handle_block_placing(
    world: &World,
    mouse_buttons: &ButtonInput<MouseButton>,
    pos: BlockPosition,
    block: SnugType,
) -> Option<BlockPosition> {
    if mouse_buttons.just_pressed(MouseButton::Right) && definition(block as usize).is_replaceable()
    {
        world.set_block(pos, 2).unwrap();
        Some(pos)
    } else {
        None
    }
}

fn update_surrounding_exposed(world: &World, pos: BlockPosition) {
    for update_pos in World::block_offsets(pos).chain([pos]) {
        let Ok(block) = world.block(update_pos) else {
            continue;
        };

        let is_exposed: bool = definition(block as usize).is_visible()
            && World::block_offsets(update_pos).any(|adj_pos| match world.block(adj_pos) {
                Ok(adj_block) => definition(adj_block as usize).is_transparent(),
                _ => false,
            });

        world.set_is_exposed(update_pos, is_exposed).unwrap();
    }
}
