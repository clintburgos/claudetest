use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use crate::biome::BiomeType;
use crate::world::WORLD_SIZE;

#[derive(Component)]
pub struct EnvironmentSprite {
    pub element_type: EnvironmentType,
    pub x: usize,
    pub y: usize,
}

#[derive(Component)]
pub struct SwayAnimation {
    pub amplitude: f32,
    pub frequency: f32,
    pub phase_offset: f32,
    pub original_rotation: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EnvironmentType {
    Tree,
    Grass,
    Rock,
    Cactus,
    Bush,
    Flower,
    Mushroom,
    DeadTree,
}

impl EnvironmentType {
    pub fn get_color(&self) -> Color {
        match self {
            EnvironmentType::Tree => Color::srgb(0.1, 0.5, 0.1),
            EnvironmentType::Grass => Color::srgb(0.3, 0.7, 0.2),
            EnvironmentType::Rock => Color::srgb(0.6, 0.6, 0.6),
            EnvironmentType::Cactus => Color::srgb(0.2, 0.6, 0.2),
            EnvironmentType::Bush => Color::srgb(0.2, 0.4, 0.1),
            EnvironmentType::Flower => Color::srgb(0.9, 0.3, 0.5),
            EnvironmentType::Mushroom => Color::srgb(0.8, 0.7, 0.6),
            EnvironmentType::DeadTree => Color::srgb(0.4, 0.3, 0.2),
        }
    }

    pub fn get_size(&self) -> Vec2 {
        match self {
            EnvironmentType::Tree => Vec2::new(3.0, 4.0),
            EnvironmentType::Grass => Vec2::new(1.0, 2.0),
            EnvironmentType::Rock => Vec2::new(2.0, 1.5),
            EnvironmentType::Cactus => Vec2::new(1.5, 3.0),
            EnvironmentType::Bush => Vec2::new(2.0, 1.5),
            EnvironmentType::Flower => Vec2::new(0.8, 1.0),
            EnvironmentType::Mushroom => Vec2::new(1.0, 1.0),
            EnvironmentType::DeadTree => Vec2::new(2.5, 3.5),
        }
    }

    pub fn should_sway(&self) -> bool {
        matches!(self, 
            EnvironmentType::Tree | 
            EnvironmentType::Grass | 
            EnvironmentType::Bush | 
            EnvironmentType::Flower |
            EnvironmentType::Cactus
        )
    }

    pub fn get_sway_properties(&self) -> (f32, f32) {
        match self {
            EnvironmentType::Tree => (0.05, 1.0),      // Small amplitude, slow frequency
            EnvironmentType::Grass => (0.1, 3.0),      // Medium amplitude, fast frequency
            EnvironmentType::Bush => (0.03, 1.5),      // Small amplitude, medium frequency
            EnvironmentType::Flower => (0.08, 2.5),    // Medium amplitude, fast frequency
            EnvironmentType::Cactus => (0.02, 0.8),    // Very small amplitude, slow frequency
            _ => (0.0, 0.0),
        }
    }
}

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sway_animation_system);
    }
}

pub fn get_environment_elements(biome: &BiomeType, tile_x: usize, tile_y: usize) -> Vec<EnvironmentType> {
    let mut elements = Vec::new();

    // Use tile coordinates as seed for consistent generation
    let seed = ((tile_x * WORLD_SIZE + tile_y) * 12345) as u64;
    let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(seed);

    match biome {
        BiomeType::Forest => {
            if seeded_rng.gen::<f32>() < 0.3 {
                elements.push(EnvironmentType::Tree);
            }
            if seeded_rng.gen::<f32>() < 0.4 {
                elements.push(EnvironmentType::Bush);
            }
            if seeded_rng.gen::<f32>() < 0.2 {
                elements.push(EnvironmentType::Mushroom);
            }
        },
        BiomeType::TropicalRainforest => {
            if seeded_rng.gen::<f32>() < 0.5 {
                elements.push(EnvironmentType::Tree);
            }
            if seeded_rng.gen::<f32>() < 0.6 {
                elements.push(EnvironmentType::Bush);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Flower);
            }
        },
        BiomeType::Grasslands => {
            if seeded_rng.gen::<f32>() < 0.7 {
                elements.push(EnvironmentType::Grass);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Flower);
            }
            if seeded_rng.gen::<f32>() < 0.05 {
                elements.push(EnvironmentType::Rock);
            }
        },
        BiomeType::Savanna => {
            if seeded_rng.gen::<f32>() < 0.5 {
                elements.push(EnvironmentType::Grass);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Tree);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Rock);
            }
        },
        BiomeType::Desert => {
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Cactus);
            }
            if seeded_rng.gen::<f32>() < 0.15 {
                elements.push(EnvironmentType::Rock);
            }
            if seeded_rng.gen::<f32>() < 0.05 {
                elements.push(EnvironmentType::DeadTree);
            }
        },
        BiomeType::Mountain => {
            if seeded_rng.gen::<f32>() < 0.3 {
                elements.push(EnvironmentType::Rock);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::DeadTree);
            }
        },
        BiomeType::Alpine => {
            if seeded_rng.gen::<f32>() < 0.4 {
                elements.push(EnvironmentType::Rock);
            }
        },
        BiomeType::Wetlands => {
            if seeded_rng.gen::<f32>() < 0.6 {
                elements.push(EnvironmentType::Grass);
            }
            if seeded_rng.gen::<f32>() < 0.2 {
                elements.push(EnvironmentType::Bush);
            }
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Mushroom);
            }
        },
        BiomeType::Tundra => {
            if seeded_rng.gen::<f32>() < 0.1 {
                elements.push(EnvironmentType::Rock);
            }
        },
        BiomeType::Badlands => {
            if seeded_rng.gen::<f32>() < 0.2 {
                elements.push(EnvironmentType::Rock);
            }
            if seeded_rng.gen::<f32>() < 0.05 {
                elements.push(EnvironmentType::DeadTree);
            }
        },
        BiomeType::Volcanic => {
            if seeded_rng.gen::<f32>() < 0.25 {
                elements.push(EnvironmentType::Rock);
            }
        },
        BiomeType::Caves => {
            if seeded_rng.gen::<f32>() < 0.3 {
                elements.push(EnvironmentType::Mushroom);
            }
            if seeded_rng.gen::<f32>() < 0.4 {
                elements.push(EnvironmentType::Rock);
            }
        },
        // Ocean and Coastal don't have land-based environment elements
        _ => {}
    }

    elements
}

fn sway_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SwayAnimation)>,
) {
    for (mut transform, sway) in query.iter_mut() {
        let time_offset = time.elapsed_seconds() + sway.phase_offset;
        let sway_amount = (time_offset * sway.frequency).sin() * sway.amplitude;
        transform.rotation = Quat::from_rotation_z(sway.original_rotation + sway_amount);
    }
}