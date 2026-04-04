use crate::{
    config::{Config, TILE_H, TILE_W},
    position::{GridPosition, ProjectorRes},
    renderer::SpriteAssets,
    world::{CHUNK_V, World as AeWorld, dictionary::BlockType},
};
use aether::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::camera::primitives::MeshAabb;
use bevy::ecs::system::SystemParam;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use spico::Projector;
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Resource, Default)]
pub struct RenderedChunks(pub HashSet<IVec2>);

#[derive(Component)]
pub struct DrawTask(Task<(ChunkPos, Mesh)>);

#[derive(Resource, Default)]
pub struct DrawsQueued(HashSet<ChunkPos>);

#[derive(SystemParam)]
pub struct DrawingContext<'w, 's> {
    commands: Commands<'w, 's>,
    world: Res<'w, AeWorld>,
    config: Res<'w, Config>,
    projector: Res<'w, ProjectorRes>,
    rendered_chunks: Res<'w, RenderedChunks>,
    draws_queued: ResMut<'w, DrawsQueued>,
    sprite_assets: Res<'w, SpriteAssets>,
    layouts: Res<'w, Assets<TextureAtlasLayout>>,
}

pub fn dispatch_chunk_drawing(ctx: DrawingContext) {
    let DrawingContext {
        mut commands,
        world,
        config,
        projector,
        rendered_chunks,
        mut draws_queued,
        sprite_assets,
        layouts,
    } = ctx;

    let player_chunk_pos = ChunkPos::new(0, 0);
    let task_pool = AsyncComputeTaskPool::get();
    let layout = Arc::new(layouts.get(&sprite_assets.layout).unwrap().clone());
    let mut spawns_left = 20;

    for pos in AeWorld::spiral_around(player_chunk_pos, config.world.render_distance) {
        if spawns_left <= 0 {
            return;
        }

        if rendered_chunks.0.contains(&pos)
            || draws_queued.0.contains(&pos)
            || !world.contains(&pos)
        {
            continue;
        }

        draws_queued.0.insert(pos);

        let proj_handle = projector.0.clone();
        let layout_handle = layout.clone();
        let world_handle = world.clone();

        let task = task_pool.spawn(async move {
            let mesh = build_chunk_mesh(world_handle, layout_handle, proj_handle, pos);
            (pos, mesh)
        });

        commands.spawn(DrawTask(task));
        spawns_left -= 1;
    }
}

pub fn poll_chunk_drawing(
    mut commands: Commands,
    mut rendered_chunks: ResMut<RenderedChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    sprite_assets: Res<SpriteAssets>,
    tasks: Query<(Entity, &mut DrawTask)>,
    mut draws_queued: ResMut<DrawsQueued>,
) {
    for (entity, mut task) in tasks {
        let Some((pos, mesh)) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        let origin_pos = AeWorld::to_block(&pos);

        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(sprite_assets.material.clone()),
            GridPosition(origin_pos.as_vec3()),
        ));

        rendered_chunks.0.insert(pos);
        draws_queued.0.remove(&pos);
        commands.entity(entity).despawn();
    }
}

/// # Panics
///
/// Panics if the chunk is not loaded at the given [`ChunkPos`] before the function is called.
fn build_chunk_mesh(
    world: AeWorld,
    layout: Arc<TextureAtlasLayout>,
    projector: Arc<Projector>,
    chunk_pos: ChunkPos,
) -> Mesh {
    let mut positions = Vec::with_capacity(CHUNK_V * 4);
    let mut uvs = Vec::with_capacity(CHUNK_V * 4);
    let mut indices = Vec::with_capacity(CHUNK_V * 6);
    let origin_pos = AeWorld::to_block(&chunk_pos);

    let mut push_quad = |screen_pos: Vec2, depth: f32, block: BlockType| {
        let start_idx = positions.len() as u32;
        let Vec2 { x, y } = screen_pos;
        let rect = layout.textures[block as usize];
        let atlas_size = layout.size.as_vec2();
        let min = rect.min.as_vec2() / atlas_size;
        let max = rect.max.as_vec2() / atlas_size;

        positions.extend_from_slice(&[
            [x, y, depth],
            [x + TILE_W as f32, y, depth],
            [x + TILE_W as f32, y + TILE_H as f32, depth],
            [x, y + TILE_H as f32, depth],
        ]);

        uvs.extend_from_slice(&[
            [min.x, max.y],
            [max.x, max.y],
            [max.x, min.y],
            [min.x, min.y],
        ]);

        indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    };

    for pos in AeWorld::blocks_in(chunk_pos) {
        let local_pos = pos - origin_pos;
        let block = world.block(&pos).unwrap();

        if block == 0 || !world.is_exposed(&pos).unwrap_or(false) {
            continue;
        }

        let p_3d = projector.grid_to_screen(local_pos.as_vec3());
        let screen_pos = vec2(p_3d.x, p_3d.z - p_3d.y);
        let depth = local_pos.element_sum() as f32 * 0.1;

        push_quad(screen_pos, depth, block);
    }

    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_aabb();
    mesh
}
