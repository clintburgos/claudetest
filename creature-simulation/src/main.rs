mod biome;
mod world;
mod render;
mod environment;
mod optimization;
mod optimized_systems;

use bevy::prelude::*;
use world::{WorldGenerator, WorldMap, WORLD_SIZE};
use render::RenderPlugin;
use environment::EnvironmentPlugin;
use optimized_systems::{OptimizationPlugin, start_world_generation, optimized_render_world_tiles};

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
        .add_plugins(EnvironmentPlugin)
        .add_plugins(OptimizationPlugin)
        .add_systems(Startup, (setup_camera, start_world_generation))
        .add_systems(Update, optimized_render_world_tiles)
        .run();
}

fn setup_camera(mut commands: Commands) {
    debug!("Setting up camera...");
    commands.spawn(Camera2dBundle::default());
    debug!("Camera spawned!");
}

// Simple fallback render system to test if the basic rendering works
fn fallback_simple_render(
    mut commands: Commands,
    world_map: Option<Res<WorldMap>>,
    existing_tiles: Query<Entity, (With<render::WorldTile>, Without<optimization::LODLevel>)>,
) {
    let Some(world_map) = world_map else { return };
    
    if !world_map.is_changed() { return }
    
    // Only render if we have no tiles yet and it's a small test area
    if existing_tiles.iter().count() > 0 { return }
    
    info!("Fallback render: creating a small test area");
    
    // Render a small 10x10 area around the center for testing
    let center_x = WORLD_SIZE / 2;
    let center_y = WORLD_SIZE / 2;
    
    for x in (center_x - 5)..(center_x + 5) {
        for y in (center_y - 5)..(center_y + 5) {
            let tile = &world_map.tiles[x][y];
            let color = tile.biome.get_color();
            
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(render::TILE_SIZE, render::TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        (x as f32 - WORLD_SIZE as f32 / 2.0) * render::TILE_SIZE,
                        (y as f32 - WORLD_SIZE as f32 / 2.0) * render::TILE_SIZE,
                        0.0,
                    )),
                    ..default()
                },
                render::WorldTile { x, y },
            ));
        }
    }
    info!("Fallback render: created 100 test tiles");
}
