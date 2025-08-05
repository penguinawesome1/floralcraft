use crate::config::{HALF_TILE_HEIGHT, HALF_TILE_WIDTH};
use crate::world::{DirtyChunks, ResWorld, World, block_dictionary::SnugType};
use bevy::tasks::futures_lite::future;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};
use spriso::IsoProjection;
use std::sync::Arc;
use terrain_data::prelude::{BlockPosition, ChunkAccessError, ChunkPosition};

#[derive(Resource)]
pub struct ResIsoProjection(pub Arc<IsoProjection>);

#[derive(Resource)]
pub struct ImageMap {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct ChunkMeshTask(Task<Result<(Mesh, Transform), ChunkAccessError>>);

pub fn make_draw_tasks(
    mut commands: Commands,
    mut dirty_chunks: ResMut<DirtyChunks>,
    world: Res<ResWorld>,
    image_map: Res<ImageMap>,
    texture_atlases: Res<Assets<TextureAtlasLayout>>,
    proj: Res<ResIsoProjection>,
) {
    let compute_pool = AsyncComputeTaskPool::get();

    let layout: &TextureAtlasLayout = texture_atlases.get(&image_map.layout).unwrap();

    for chunk_pos in dirty_chunks.0.drain() {
        let world_clone: Arc<World> = Arc::clone(&world.0);
        let proj_clone: Arc<IsoProjection> = Arc::clone(&proj.0);
        let layout_clone: TextureAtlasLayout = layout.clone();

        let task = compute_pool
            .spawn(async move { draw_chunk(&world_clone, &proj_clone, layout_clone, chunk_pos) });

        commands.spawn(ChunkMeshTask(task));
    }
}

pub fn handle_draw_tasks(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ChunkMeshTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    image_map: Res<ImageMap>,
) {
    for (entity, mut task) in &mut query {
        let Some(result) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        match result {
            Ok((mesh, transform)) => {
                let mesh_handle: Handle<Mesh> = meshes.add(mesh);
                let material_handle: Handle<ColorMaterial> =
                    materials.add(ColorMaterial::from(image_map.image.clone()));

                commands.spawn((
                    Mesh2d(mesh_handle),
                    MeshMaterial2d(material_handle),
                    transform,
                ));
            }
            Err(e) => {
                eprintln!("Error generating chunk mesh: {}", e);
            }
        }

        commands.entity(entity).despawn();
    }
}

fn draw_chunk(
    world: &World,
    proj: &IsoProjection,
    layout: TextureAtlasLayout,
    chunk_pos: ChunkPosition,
) -> Result<(Mesh, Transform), ChunkAccessError> {
    let chunk_origin_pos: BlockPosition = World::chunk_to_block_pos(chunk_pos);
    let chunk_origin_screen_pos: glam::Vec3 = proj.world_to_screen(chunk_origin_pos);
    let render_data = render_data(&world, chunk_pos, chunk_origin_pos)?;
    let mesh: Mesh = render_data_mesh(render_data, &layout, &proj, chunk_origin_screen_pos);
    let transform: Transform = Transform::from_xyz(
        chunk_origin_screen_pos.x,
        chunk_origin_screen_pos.z - chunk_origin_screen_pos.y,
        (chunk_pos.x + chunk_pos.y) as f32,
    );

    Ok((mesh, transform))
}

fn render_data(
    world: &World,
    chunk_pos: ChunkPosition,
    origin_block_pos: BlockPosition,
) -> Result<impl Iterator<Item = (SnugType, BlockPosition)>, ChunkAccessError> {
    let chunk = Arc::new(world.chunk(chunk_pos)?);
    let chunk_clone = Arc::clone(&chunk);

    let render_data = World::chunk_coords(ChunkPosition::ZERO)
        .filter(move |&pos| chunk.is_exposed(pos).unwrap_or(false))
        .map(move |pos| {
            let block: SnugType = chunk_clone.block(pos).unwrap_or(0);
            let global_pos: BlockPosition = origin_block_pos + pos;
            (block, global_pos)
        });

    Ok(render_data)
}

fn render_data_mesh(
    render_data: impl Iterator<Item = (SnugType, BlockPosition)>,
    atlas_layout: &TextureAtlasLayout,
    proj: &IsoProjection,
    chunk_origin_screen_pos: glam::Vec3,
) -> Mesh {
    let mut all_positions: Vec<[f32; 3]> = Vec::new();
    let mut all_uvs: Vec<[f32; 2]> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();

    for (i, (block, block_pos)) in render_data.enumerate() {
        let Some(rect) = atlas_layout.textures.get(block as usize) else {
            continue;
        };

        let screen_pos: glam::Vec3 = proj.world_to_screen(block_pos);
        let local_screen_pos: glam::Vec3 = screen_pos - chunk_origin_screen_pos;
        let center_x: f32 = local_screen_pos.x;
        let center_y: f32 = local_screen_pos.z - local_screen_pos.y;
        let z_index: f32 = (block_pos.x + block_pos.y + block_pos.z) as f32;

        let vertices: [[f32; 3]; 4] = [
            // top left
            [
                center_x - HALF_TILE_WIDTH as f32,
                center_y + HALF_TILE_HEIGHT as f32,
                z_index,
            ],
            // top right
            [
                center_x + HALF_TILE_WIDTH as f32,
                center_y + HALF_TILE_HEIGHT as f32,
                z_index,
            ],
            // bottom right
            [
                center_x + HALF_TILE_WIDTH as f32,
                center_y - HALF_TILE_HEIGHT as f32,
                z_index,
            ],
            // bottom left
            [
                center_x - HALF_TILE_WIDTH as f32,
                center_y - HALF_TILE_HEIGHT as f32,
                z_index,
            ],
        ];
        all_positions.extend_from_slice(&vertices);

        let min_x: f32 = rect.min.x as f32 / atlas_layout.size.x as f32;
        let min_y: f32 = rect.min.y as f32 / atlas_layout.size.y as f32;
        let max_x: f32 = rect.max.x as f32 / atlas_layout.size.x as f32;
        let max_y: f32 = rect.max.y as f32 / atlas_layout.size.y as f32;

        let uvs: [[f32; 2]; 4] = [
            [min_x, min_y], // top left
            [max_x, min_y], // top right
            [max_x, max_y], // bottom right
            [min_x, max_y], // bottom left
        ];

        all_uvs.extend_from_slice(&uvs);

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let base_index: u32 = (i * 4) as u32;
        all_indices.extend(indices.iter().map(|&index| index + base_index));
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, all_positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, all_uvs)
    .with_inserted_indices(Indices::U32(all_indices))
}
