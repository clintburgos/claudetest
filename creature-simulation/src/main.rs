mod biome;
mod world;
mod render;
mod environment;
mod optimization;
mod optimized_systems;
mod loading;

use bevy::prelude::*;
use std::time::Instant;
use world::{WorldGenerator, WorldMap, WORLD_SIZE};
use render::RenderPlugin;
use environment::EnvironmentPlugin;
use optimized_systems::{OptimizationPlugin, start_world_generation, optimized_render_world_tiles};
use loading::LoadingPlugin;

fn main() {
    let app_start = Instant::now();
    println!("⏱️ TIMING: Application startup began at {:?}", app_start);
    
    let plugin_setup_start = Instant::now();
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Creature Simulation".into(),
            resolution: (1200.0, 800.0).into(),
            ..default()
        }),
        ..default()
    }));
    
    let default_plugins_time = plugin_setup_start.elapsed();
    println!("⏱️ TIMING: Default plugins setup took: {:?}", default_plugins_time);
    
    let custom_plugins_start = Instant::now();
    app.add_plugins(RenderPlugin);
    app.add_plugins(EnvironmentPlugin);
    app.add_plugins(OptimizationPlugin);
    app.add_plugins(LoadingPlugin);
    
    let custom_plugins_time = custom_plugins_start.elapsed();
    println!("⏱️ TIMING: Custom plugins setup took: {:?}", custom_plugins_time);
    
    let systems_setup_start = Instant::now();
    app.add_systems(Startup, (setup_camera, start_world_generation));
    app.add_systems(Update, optimized_render_world_tiles);
    
    let systems_setup_time = systems_setup_start.elapsed();
    println!("⏱️ TIMING: Systems setup took: {:?}", systems_setup_time);
    
    let total_setup_time = app_start.elapsed();
    println!("⏱️ TIMING: Total app setup took: {:?}", total_setup_time);
    println!("⏱️ TIMING: Starting app.run()...");
    
    app.run();
}

fn setup_camera(mut commands: Commands, mut loading_state: ResMut<loading::LoadingState>) {
    let camera_setup_start = Instant::now();
    info!("⏱️ TIMING: Setting up camera at {:?}", camera_setup_start);
    commands.spawn(Camera2dBundle::default());
    let camera_setup_time = camera_setup_start.elapsed();
    info!("⏱️ TIMING: Camera setup took: {:?}", camera_setup_time);
    
    // Initial loading progress
    loading_state.progress = 0.1;
    loading_state.current_message = "📷 Setting up camera systems...".to_string();
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
