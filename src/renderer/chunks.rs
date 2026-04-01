use crate::{
    config::{Config, TILE_H, TILE_W},
    player::PlayerMovedFilter,
    position::{GridPosition, ProjectorRes},
    renderer::SpriteAssets,
    world::{CHUNK_V, World as AeWorld, dictionary::BlockType},
};
use aether::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use spico::Projector;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub struct RenderedChunks(pub HashSet<IVec2>);

#[derive(SystemParam)]
pub struct ChunkRenderer<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<ColorMaterial>>,
    pub layouts: Res<'w, Assets<TextureAtlasLayout>>,
    pub sprite_assets: Res<'w, SpriteAssets>,
    pub world: Res<'w, AeWorld>,
}

pub fn draw_chunks(
    mut renderer: ChunkRenderer,
    mut rendered_chunks: ResMut<RenderedChunks>,
    config: Res<Config>,
    projector: Res<ProjectorRes>,
    query: Query<&GridPosition, PlayerMovedFilter>,
) {
    let radius = config.world.render_distance;

    for player_pos in &query {
        let player_chunk_pos = AeWorld::to_chunk(player_pos.0.as_ivec3());

        for pos in AeWorld::square_around(player_chunk_pos, radius) {
            if rendered_chunks.0.contains(&pos) || !renderer.world.contains(&pos) {
                continue;
            }

            spawn_chunk(&mut renderer, &projector.0, pos);
            rendered_chunks.0.insert(pos);
        }
    }
}

fn spawn_chunk(ctx: &mut ChunkRenderer, projector: &Projector, chunk_pos: ChunkPos) {
    let layout = ctx.layouts.get(&ctx.sprite_assets.layout).unwrap();
    let mesh = build_chunk_mesh(&ctx.world, layout, projector, chunk_pos);
    let origin_pos = AeWorld::to_block(&chunk_pos);

    ctx.commands.spawn((
        Mesh2d(ctx.meshes.add(mesh)),
        MeshMaterial2d(
            ctx.materials
                .add(ColorMaterial::from(ctx.sprite_assets.texture.clone())),
        ),
        GridPosition(origin_pos.as_vec3()),
        Visibility::default(),
    ));
}

fn build_chunk_mesh(
    world: &AeWorld,
    layout: &TextureAtlasLayout,
    projector: &Projector,
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

        let Ok(block) = world.block(&pos) else {
            continue;
        };

        if block == 0 || !world.is_exposed(&pos).unwrap_or(false) {
            continue;
        }

        let p_3d = projector.grid_to_screen(local_pos.as_vec3());
        let screen_pos = vec2(p_3d.x, p_3d.z - p_3d.y);
        let depth = local_pos.element_sum() as f32 * 0.1;

        push_quad(screen_pos, depth, block);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
