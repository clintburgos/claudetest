use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashMap;
use crate::world::{WorldMap, WorldGenerator, WORLD_SIZE};
use crate::biome::BiomeType;
use crate::environment::{EnvironmentSprite, SwayAnimation, EnvironmentType, get_environment_elements};
use crate::render::{WorldTile, TILE_SIZE};
use crate::optimization::*;

pub struct OptimizationPlugin;

impl Plugin for OptimizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkManager>()
            .init_resource::<SpatialHash>()
            .init_resource::<SharedAnimationState>()
            .add_systems(Update, (
                update_shared_animation_state,
                update_lod_system,
                optimized_sway_system,
                chunk_management_system,
                check_world_generation_system,
                update_spatial_hash_system,
            ));
    }
}

// === ASYNC WORLD GENERATION ===
pub fn start_world_generation(mut commands: Commands) {
    debug!("Starting world generation...");
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        debug!("World generation task started in background thread");
        let generator = WorldGenerator::new(Some(12345));
        debug!("Generating world map...");
        let world_map = generator.generate_world();
        debug!("World generation completed!");
        world_map
    });
    commands.spawn(WorldGenerationTask(task));
    debug!("World generation task spawned");
}

fn check_world_generation_system(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut WorldGenerationTask)>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(world_map) = future::block_on(future::poll_once(&mut task.0)) {
            debug!("World generation task completed! Converting to compressed format...");
            // Convert to compressed format
            let compressed_data = CompressedWorldData::from_world_map(&world_map);
            debug!("Compressed data created. Inserting resources...");
            commands.insert_resource(compressed_data);
            commands.insert_resource(world_map);
            commands.entity(entity).despawn();
            debug!("World map resource inserted! Ready to render.");
        }
    }
}

// === OPTIMIZED CHUNK RENDERING ===
pub fn optimized_render_world_tiles(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    world_map: Option<Res<WorldMap>>,
    mut chunk_manager: ResMut<ChunkManager>,
    existing_tiles: Query<Entity, With<WorldTile>>,
    existing_environment: Query<Entity, With<EnvironmentSprite>>,
) {
    let Some(world_map) = world_map else { 
        // Only log this occasionally to avoid spam
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 60 == 1 { // Log once per second at 60fps
                debug!("Waiting for world map resource...");
            }
        }
        return; 
    };
    let Ok(camera_transform) = camera_query.get_single() else { 
        debug!("Camera not found!");
        return;
    };

    if world_map.is_changed() {
        debug!("World map changed! Clearing existing entities...");
        // Clear all existing entities
        for entity in existing_tiles.iter() {
            commands.entity(entity).despawn();
        }
        for entity in existing_environment.iter() {
            commands.entity(entity).despawn();
        }
        chunk_manager.loaded_chunks.clear();
        debug!("Cleared {} tiles and {} environment entities", existing_tiles.iter().count(), existing_environment.iter().count());
    }

    // Calculate visible chunks
    debug!("Calculating visible chunks from camera position: {:?}", camera_transform.translation);
    let visible_chunks = calculate_visible_chunks(camera_transform.translation);
    debug!("Found {} visible chunks", visible_chunks.len());
    
    // Unload chunks that are no longer visible
    let mut chunks_to_unload = Vec::new();
    for &chunk_coord in chunk_manager.active_chunks.iter() {
        if !visible_chunks.contains(&chunk_coord) {
            chunks_to_unload.push(chunk_coord);
        }
    }
    
    for chunk_coord in chunks_to_unload {
        if let Some(chunk_data) = chunk_manager.loaded_chunks.get(&chunk_coord) {
            for entity in &chunk_data.entities {
                commands.entity(*entity).despawn();
            }
        }
        chunk_manager.loaded_chunks.remove(&chunk_coord);
    }

    // Update active chunks
    chunk_manager.active_chunks = visible_chunks.clone();

    // Load new chunks
    debug!("Loading new chunks...");
    let mut chunks_loaded = 0;
    for chunk_coord in visible_chunks {
        if !chunk_manager.loaded_chunks.contains_key(&chunk_coord) {
            debug!("Loading chunk {:?}", chunk_coord);
            let entities = render_chunk(&mut commands, &world_map, chunk_coord);
            debug!("Chunk {:?} loaded with {} entities", chunk_coord, entities.len());
            chunk_manager.loaded_chunks.insert(chunk_coord, ChunkData {
                entities,
                is_loaded: true,
            });
            chunks_loaded += 1;
        }
    }
    debug!("Loaded {} new chunks", chunks_loaded);
}

fn render_chunk(
    commands: &mut Commands,
    world_map: &WorldMap,
    chunk_coord: (i32, i32),
) -> Vec<Entity> {
    debug!("Rendering chunk {:?}", chunk_coord);
    let mut entities = Vec::new();
    let (start_x, start_y, end_x, end_y) = chunk_to_world_bounds(chunk_coord.0, chunk_coord.1);
    debug!("Chunk bounds: ({}, {}) to ({}, {})", start_x, start_y, end_x, end_y);
    
    // Skip invalid chunks
    if start_x >= end_x || start_y >= end_y || start_x >= WORLD_SIZE || start_y >= WORLD_SIZE {
        debug!("Skipping invalid chunk {:?}", chunk_coord);
        return entities;
    }

    // Group similar elements for instancing
    let mut instanced_elements: HashMap<EnvironmentType, Vec<(Vec3, f32)>> = HashMap::new();
    
    let mut tiles_processed = 0;

    for x in start_x..end_x {
        for y in start_y..end_y {
            if x >= WORLD_SIZE || y >= WORLD_SIZE { continue; }
            
            let tile = &world_map.tiles[x][y];
            let color = tile.biome.get_color();
            
            // Spawn base tile
            let tile_entity = commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE,
                        (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE,
                        0.0,
                    )),
                    ..default()
                },
                WorldTile { x, y },
                LODLevel(0),
            )).id();
            entities.push(tile_entity);

            // Collect environment elements for instancing
            let environment_elements = get_environment_elements(&tile.biome, x, y);
            for element_type in environment_elements {
                let base_x = (x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
                let base_y = (y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
                let position = Vec3::new(base_x, base_y, 1.0);
                
                instanced_elements.entry(element_type)
                    .or_default()
                    .push((position, 0.0)); // rotation
            }
        }
    }

    // Create instanced sprites
    for (element_type, positions_rotations) in instanced_elements {
        if positions_rotations.len() > 5 { // Only instance if we have enough
            let (positions, rotations): (Vec<_>, Vec<_>) = positions_rotations.into_iter().unzip();
            let scales = vec![element_type.get_size(); positions.len()];
            
            let instanced_entity = commands.spawn((
                InstancedSprites {
                    element_type,
                    positions,
                    rotations,
                    scales,
                },
                LODLevel(0),
            )).id();
            entities.push(instanced_entity);
        } else {
            // Spawn individual sprites for small groups
            for (position, _rotation) in positions_rotations {
                let env_entity = spawn_individual_environment_element(commands, element_type, position);
                entities.push(env_entity);
            }
        }
    }

    entities
}

fn spawn_individual_environment_element(
    commands: &mut Commands,
    element_type: EnvironmentType,
    position: Vec3,
) -> Entity {
    let size = element_type.get_size();
    let color = element_type.get_color();

    let mut entity_commands = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(position),
            ..default()
        },
        EnvironmentSprite {
            element_type,
            x: 0, // Not used in optimized version
            y: 0,
        },
        LODLevel(0),
    ));

    // Add swaying animation if appropriate
    if element_type.should_sway() {
        let (amplitude, frequency) = element_type.get_sway_properties();
        entity_commands.insert(SwayAnimation {
            amplitude,
            frequency,
            phase_offset: 0.0,
            original_rotation: 0.0,
        });
    }

    entity_commands.id()
}

// === LOD SYSTEM ===
fn update_lod_system(
    camera_query: Query<&Transform, With<Camera>>,
    mut lod_query: Query<(&Transform, &mut LODLevel), (Without<Camera>, With<EnvironmentSprite>)>,
) {
    let Ok(camera_transform) = camera_query.get_single() else { return };
    
    for (transform, mut lod) in lod_query.iter_mut() {
        let distance = camera_transform.translation.distance(transform.translation);
        lod.0 = match distance {
            d if d < 100.0 => 0,
            d if d < 300.0 => 1,
            d if d < 600.0 => 2,
            _ => 3,
        };
    }
}

// === OPTIMIZED ANIMATION SYSTEM ===
fn update_shared_animation_state(
    time: Res<Time>,
    mut wind_state: ResMut<SharedAnimationState>,
) {
    wind_state.wind_time += time.delta_seconds();
    // Add some variation to wind strength
    wind_state.wind_strength = 0.8 + 0.2 * (wind_state.wind_time * 0.1).sin();
}

fn optimized_sway_system(
    wind_state: Res<SharedAnimationState>,
    mut query: Query<(&mut Transform, &SwayAnimation, &LODLevel)>,
) {
    for (mut transform, sway, lod) in query.iter_mut() {
        // Skip animation for distant objects
        if lod.0 >= 2 { continue; }
        
        let time_offset = wind_state.wind_time + sway.phase_offset;
        let effective_wind = wind_state.wind_strength * (if lod.0 == 0 { 1.0 } else { 0.5 });
        let sway_amount = (time_offset * sway.frequency).sin() * sway.amplitude * effective_wind;
        transform.rotation = Quat::from_rotation_z(sway.original_rotation + sway_amount);
    }
}

// === SPATIAL HASH SYSTEM ===
fn update_spatial_hash_system(
    mut spatial_hash: ResMut<SpatialHash>,
    environment_query: Query<(Entity, &Transform), (With<EnvironmentSprite>, Changed<Transform>)>,
) {
    // Clear and rebuild spatial hash for changed entities
    for (entity, transform) in environment_query.iter() {
        spatial_hash.insert(entity, transform.translation);
    }
}

// === CHUNK MANAGEMENT ===
fn chunk_management_system(
    camera_query: Query<&Transform, With<Camera>>,
    chunk_manager: Res<ChunkManager>,
) {
    let Ok(_camera_transform) = camera_query.get_single() else { return };
    
    // This system can be expanded to handle more sophisticated chunk loading/unloading
    // such as preloading chunks in the direction of movement, etc.
    
    // For now, the main chunk logic is in optimized_render_world_tiles
}

// === INSTANCED RENDERING SYSTEM ===
fn render_instanced_sprites_system(
    mut commands: Commands,
    instanced_query: Query<(Entity, &InstancedSprites, &LODLevel)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, instanced, lod) in instanced_query.iter() {
        // Skip rendering for very distant LOD levels
        if lod.0 >= 3 { continue; }
        
        // This would be expanded to actually use GPU instancing
        // For now, it's a placeholder for the instanced rendering concept
        // Real implementation would require custom shaders and instance buffers
    }
}