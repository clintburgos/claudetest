use bevy::prelude::*;
use rand::Rng;
use crate::world::{WorldMap, WORLD_SIZE};
use crate::environment::{EnvironmentSprite, SwayAnimation, EnvironmentType, get_environment_elements};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_world_tiles, handle_camera_movement));
    }
}

#[derive(Component)]
pub struct WorldTile {
    pub x: usize,
    pub y: usize,
}

pub const TILE_SIZE: f32 = 4.0;
const CAMERA_SPEED: f32 = 300.0;

fn render_world_tiles(
    mut commands: Commands,
    world_map: Option<Res<WorldMap>>,
    existing_tiles: Query<Entity, With<WorldTile>>,
    existing_environment: Query<Entity, With<EnvironmentSprite>>,
) {
    if let Some(world_map) = world_map {
        if world_map.is_changed() {
            // Clear existing tiles and environment
            for entity in existing_tiles.iter() {
                commands.entity(entity).despawn();
            }
            for entity in existing_environment.iter() {
                commands.entity(entity).despawn();
            }

            // Render new tiles
            for x in 0..WORLD_SIZE {
                for y in 0..WORLD_SIZE {
                    let tile = &world_map.tiles[x][y];
                    let color = tile.biome.get_color();
                    
                    // Spawn base tile
                    commands.spawn((
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
                    ));

                    // Spawn environment elements
                    let environment_elements = get_environment_elements(&tile.biome, x, y);
                    for element_type in environment_elements {
                        spawn_environment_element(&mut commands, element_type, x, y);
                    }
                }
            }
        }
    }
}

fn spawn_environment_element(
    commands: &mut Commands,
    element_type: EnvironmentType,
    tile_x: usize,
    tile_y: usize,
) {
    let mut rng = rand::thread_rng();
    
    // Calculate base position
    let base_x = (tile_x as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
    let base_y = (tile_y as f32 - WORLD_SIZE as f32 / 2.0) * TILE_SIZE;
    
    // Add small random offset within the tile
    let offset_x = rng.gen_range(-TILE_SIZE * 0.3..TILE_SIZE * 0.3);
    let offset_y = rng.gen_range(-TILE_SIZE * 0.3..TILE_SIZE * 0.3);
    
    let position = Vec3::new(base_x + offset_x, base_y + offset_y, 1.0);
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
            x: tile_x,
            y: tile_y,
        },
    ));

    // Add swaying animation if appropriate
    if element_type.should_sway() {
        let (amplitude, frequency) = element_type.get_sway_properties();
        let phase_offset = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
        
        entity_commands.insert(SwayAnimation {
            amplitude,
            frequency,
            phase_offset,
            original_rotation: 0.0,
        });
    }
}

fn handle_camera_movement(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
            camera_transform.translation += direction * CAMERA_SPEED * time.delta_seconds();
        }
    }
}