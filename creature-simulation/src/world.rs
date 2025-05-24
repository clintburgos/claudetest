use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use crate::biome::{BiomeType, ResourceType};

pub const WORLD_SIZE: usize = 1000;
pub const CHUNK_SIZE: usize = 16;

#[derive(Component, Debug, Clone)]
pub struct Tile {
    pub biome: BiomeType,
    pub elevation: f32,
    pub temperature: f32,
    pub moisture: f32,
    pub resources: Vec<ResourceType>,
}

#[derive(Resource)]
pub struct WorldMap {
    pub tiles: Vec<Vec<Tile>>,
    pub seed: u32,
}

pub struct WorldGenerator {
    elevation_noise: Perlin,
    temperature_noise: Perlin,
    moisture_noise: Perlin,
    seed: u32,
}

impl WorldGenerator {
    pub fn new(seed: Option<u32>) -> Self {
        let seed = seed.unwrap_or_else(|| rand::thread_rng().gen());
        
        let mut elevation_noise = Perlin::new(seed);
        elevation_noise = elevation_noise.set_seed(seed);
        
        let mut temperature_noise = Perlin::new(seed + 1);
        temperature_noise = temperature_noise.set_seed(seed + 1);
        
        let mut moisture_noise = Perlin::new(seed + 2);
        moisture_noise = moisture_noise.set_seed(seed + 2);

        Self {
            elevation_noise,
            temperature_noise,
            moisture_noise,
            seed,
        }
    }

    pub fn generate_world(&self) -> WorldMap {
        let mut tiles = vec![vec![Tile {
            biome: BiomeType::Ocean,
            elevation: 0.0,
            temperature: 0.0,
            moisture: 0.0,
            resources: vec![],
        }; WORLD_SIZE]; WORLD_SIZE];

        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                let elevation = self.generate_elevation(x, y);
                let temperature = self.generate_temperature(x, y);
                let moisture = self.generate_moisture(x, y);
                
                let biome = self.determine_biome(elevation, temperature, moisture);
                let resources = self.generate_resources(&biome);

                tiles[x][y] = Tile {
                    biome,
                    elevation,
                    temperature,
                    moisture,
                    resources,
                };
            }
        }

        WorldMap { tiles, seed: self.seed }
    }

    fn generate_elevation(&self, x: usize, y: usize) -> f32 {
        let scale = 0.01;
        let octaves = 4;
        let mut elevation = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = scale;

        for _ in 0..octaves {
            elevation += self.elevation_noise.get([x as f64 * frequency, y as f64 * frequency]) as f32 * amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        (elevation + 1.0) / 2.0 // Normalize to 0-1
    }

    fn generate_temperature(&self, x: usize, y: usize) -> f32 {
        let scale = 0.005;
        let latitude_effect = 1.0 - (y as f32 / WORLD_SIZE as f32);
        let noise_value = self.temperature_noise.get([x as f64 * scale, y as f64 * scale]) as f32;
        
        (latitude_effect + noise_value * 0.3).clamp(0.0, 1.0)
    }

    fn generate_moisture(&self, x: usize, y: usize) -> f32 {
        let scale = 0.008;
        let noise_value = self.moisture_noise.get([x as f64 * scale, y as f64 * scale]) as f32;
        
        (noise_value + 1.0) / 2.0
    }

    fn determine_biome(&self, elevation: f32, temperature: f32, moisture: f32) -> BiomeType {
        // Ocean level
        if elevation < 0.3 {
            return BiomeType::Ocean;
        }
        
        // Coastal areas
        if elevation < 0.35 {
            return BiomeType::Coastal;
        }

        // High elevation biomes
        if elevation > 0.8 {
            if temperature < 0.3 {
                return BiomeType::Alpine;
            } else if temperature < 0.7 {
                return BiomeType::Mountain;
            } else {
                return BiomeType::Volcanic;
            }
        }

        // Very high elevation or extreme cold
        if elevation > 0.9 || temperature < 0.1 {
            return BiomeType::Tundra;
        }

        // Temperature and moisture based biomes
        match (temperature, moisture) {
            // Hot and dry
            (t, m) if t > 0.7 && m < 0.3 => BiomeType::Desert,
            // Hot and moderate moisture
            (t, m) if t > 0.7 && m < 0.6 => BiomeType::Savanna,
            // Hot and wet
            (t, m) if t > 0.7 && m >= 0.6 => BiomeType::TropicalRainforest,
            // Moderate temperature, very wet
            (t, m) if t > 0.3 && t <= 0.7 && m > 0.8 => BiomeType::Wetlands,
            // Moderate temperature, moderate moisture
            (t, m) if t > 0.3 && t <= 0.7 && m > 0.4 => BiomeType::Forest,
            // Moderate temperature, low moisture
            (t, m) if t > 0.3 && t <= 0.7 && m <= 0.4 => BiomeType::Grasslands,
            // Cold
            (t, _) if t <= 0.3 => BiomeType::Tundra,
            // Extreme conditions
            (t, m) if t > 0.8 && m < 0.2 => BiomeType::Badlands,
            // Default fallback
            _ => BiomeType::Grasslands,
        }
    }

    fn generate_resources(&self, biome: &BiomeType) -> Vec<ResourceType> {
        let mut rng = rand::thread_rng();
        let available_resources = biome.get_resources();
        let resource_count = rng.gen_range(1..=3.min(available_resources.len()));
        
        available_resources
            .into_iter()
            .take(resource_count)
            .collect()
    }
}