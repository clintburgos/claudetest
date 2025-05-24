mod biome;
mod world;
mod render;

use bevy::prelude::*;
use world::{WorldGenerator, WorldMap};
use render::RenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Creature Simulation".into(),
                resolution: (1200.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RenderPlugin)
        .add_systems(Startup, setup_world)
        .run();
}

fn setup_world(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    
    let generator = WorldGenerator::new(Some(12345));
    let world_map = generator.generate_world();
    
    commands.insert_resource(world_map);
}
