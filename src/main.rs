use bevy::prelude::*;
use floralcraft::{
    camera::update_camera,
    config::{ConfigPlugin, ConfigSet},
    player::PlayerPlugin,
    renderer::{RendererPlugin, RendererSet},
    world::{
        ResWorld, World,
        chunk_generation::{GenerationPlugin, GenerationSet},
        chunk_selection::choose_chunks_to_generate,
        hover_block::{HoverBlock, update_hover_block},
        interaction::InteractionPlugin,
    },
};
use std::sync::Arc;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Floralcraft".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .configure_sets(Startup, ConfigSet.before(GenerationSet))
        .configure_sets(Startup, ConfigSet.before(RendererSet))
        .add_plugins(ConfigPlugin)
        .add_plugins(RendererPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(GenerationPlugin)
        .add_plugins(InteractionPlugin)
        .add_systems(Startup, load_resources)
        .add_systems(
            Update,
            (choose_chunks_to_generate, update_camera, update_hover_block).chain(),
        )
        .run();
}

fn load_resources(mut commands: Commands) {
    commands.insert_resource(HoverBlock::default());
    commands.insert_resource(ResWorld(Arc::new(World::default())));

    commands.spawn(Camera2d);
}
