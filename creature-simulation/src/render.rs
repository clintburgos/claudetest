use bevy::prelude::*;
use crate::world::{WorldMap, WORLD_SIZE};

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

const TILE_SIZE: f32 = 4.0;
const CAMERA_SPEED: f32 = 300.0;

fn render_world_tiles(
    mut commands: Commands,
    world_map: Option<Res<WorldMap>>,
    existing_tiles: Query<Entity, With<WorldTile>>,
) {
    if let Some(world_map) = world_map {
        if world_map.is_changed() {
            // Clear existing tiles
            for entity in existing_tiles.iter() {
                commands.entity(entity).despawn();
            }

            // Render new tiles
            for x in 0..WORLD_SIZE {
                for y in 0..WORLD_SIZE {
                    let tile = &world_map.tiles[x][y];
                    let color = tile.biome.get_color();
                    
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
                }
            }
        }
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