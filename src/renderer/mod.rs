use crate::config::{Config, HALF_TILE_HEIGHT, HALF_TILE_WIDTH, TILE_HEIGHT, TILE_WIDTH};
use crate::world::{ResWorld, World, block_dictionary::SnugType};
use bevy::tasks::AsyncComputeTaskPool;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bevy_async_task::AsyncReceiver;
use bevy_async_task::AsyncTask;
use spriso::IsoProjection;
use std::collections::VecDeque;
use std::sync::Arc;
use terrain_data::prelude::{BlockPosition, ChunkAccessError, ChunkPosition};

const MAX_TASKS_PER_FRAME: usize = 5;

#[derive(Resource)]
pub struct ResIsoProjection(pub Arc<IsoProjection>);

#[derive(Resource)]
pub struct ImageMap {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<ColorMaterial>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DrawTaskPool(
    pub VecDeque<AsyncReceiver<Result<(Mesh, Transform, ChunkPosition), ChunkAccessError>>>,
);

#[derive(Component)]
pub struct ChunkPositionComponent(pub ChunkPosition);

#[derive(Resource, Default)]
pub struct ChunksToRender(pub Vec<ChunkPosition>);

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct RendererSet;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_renderer_resources.in_set(RendererSet))
            .add_systems(Update, (make_draw_tasks, handle_draw_tasks).chain());
    }
}

fn setup_renderer_resources(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    config: Res<Config>,
) {
    let image_map: ImageMap = ImageMap {
        image: asset_server.load("blocks.png"),
        layout: texture_atlases.add(TextureAtlasLayout::from_grid(
            UVec2::new(TILE_WIDTH, TILE_HEIGHT),
            config.world.num_blocks,
            1,
            None,
            None,
        )),
    };

    commands.insert_resource(ChunkMaterial(
        materials.add(ColorMaterial::from(image_map.image.clone())),
    ));
    commands.insert_resource(image_map);
    commands.insert_resource(ResIsoProjection(Arc::new(IsoProjection::new::<
        HALF_TILE_WIDTH,
        HALF_TILE_HEIGHT,
    >())));
    commands.insert_resource(DrawTaskPool::default());
    commands.insert_resource(ChunksToRender::default());
}

fn make_draw_tasks(
    mut draw_task_pool: ResMut<'_, DrawTaskPool>,
    mut chunks_to_render: ResMut<ChunksToRender>,
    world: Res<ResWorld>,
    image_map: Res<ImageMap>,
    texture_atlases: Res<Assets<TextureAtlasLayout>>,
    proj: Res<ResIsoProjection>,
) {
    let layout: &TextureAtlasLayout = texture_atlases.get(&image_map.layout).unwrap();

    for chunk_pos in chunks_to_render.0.drain(..) {
        let world_clone: Arc<World> = Arc::clone(&world.0);
        let proj_clone: Arc<IsoProjection> = Arc::clone(&proj.0);
        let layout_clone: TextureAtlasLayout = layout.clone();

        let (fut, receiver) =
            AsyncTask::new(draw_chunk(world_clone, proj_clone, layout_clone, chunk_pos)).split();

        draw_task_pool.push_back(receiver);
        AsyncComputeTaskPool::get().spawn(fut).detach();
    }
}

fn handle_draw_tasks(
    mut draw_task_pool: ResMut<'_, DrawTaskPool>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    chunk_material: Res<ChunkMaterial>,
    query: Query<(Entity, &ChunkPositionComponent)>,
) {
    for _ in 0..MAX_TASKS_PER_FRAME {
        let Some(mut receiver) = draw_task_pool.0.pop_front() else {
            return;
        };

        let Some(v) = receiver.try_recv() else {
            draw_task_pool.0.push_back(receiver);
            continue;
        };

        match v {
            Ok((mesh, transform, chunk_pos)) => {
                let mesh_handle: Handle<Mesh> = meshes.add(mesh);

                undraw_chunk(&mut commands, query, chunk_pos);

                commands.spawn((
                    Mesh2d(mesh_handle),
                    MeshMaterial2d(chunk_material.0.clone()),
                    transform,
                    ChunkPositionComponent(chunk_pos),
                ));
            }
            Err(e) => {
                eprintln!("Error generating chunk mesh: {}", e);
            }
        }
    }
}

fn undraw_chunk(
    commands: &mut Commands,
    query: Query<(Entity, &ChunkPositionComponent)>,
    pos: ChunkPosition,
) {
    if let Some((entity, _)) = query
        .iter()
        .find(|(_, chunk_component)| chunk_component.0 == pos)
    {
        commands.entity(entity).despawn();
    }
}

async fn draw_chunk(
    world: Arc<World>,
    proj: Arc<IsoProjection>,
    layout: TextureAtlasLayout,
    pos: ChunkPosition,
) -> Result<(Mesh, Transform, ChunkPosition), ChunkAccessError> {
    let chunk_origin_pos: BlockPosition = World::chunk_to_block_pos(pos);
    let chunk_origin_screen_pos: glam::Vec3 = proj.world_to_screen(chunk_origin_pos);
    let render_data = render_data(&world, pos, chunk_origin_pos)?;
    let mesh: Mesh = render_data_mesh(render_data, &layout, &proj, chunk_origin_screen_pos);
    let transform: Transform = Transform::from_xyz(
        chunk_origin_screen_pos.x,
        chunk_origin_screen_pos.z - chunk_origin_screen_pos.y,
        (pos.x + pos.y) as f32,
    );

    Ok((mesh, transform, pos))
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
