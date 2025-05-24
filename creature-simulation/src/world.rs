use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::Rng;
use std::sync::Arc;
use rayon::prelude::*;
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
        self.generate_world_with_progress(None)
    }

    pub fn generate_world_with_progress(&self, progress_callback: Option<Box<dyn Fn(f32, &str) + Send + Sync>>) -> WorldMap {
        use std::time::Instant;
        use std::sync::{Arc, Mutex};
        
        let generation_start = Instant::now();
        
        // Pre-allocate with default values for better memory performance
        let mut tiles = vec![vec![Tile {
            biome: BiomeType::Ocean,
            elevation: 0.0,
            temperature: 0.0,
            moisture: 0.0,
            resources: vec![],
        }; WORLD_SIZE]; WORLD_SIZE];

        let total_tiles = WORLD_SIZE * WORLD_SIZE;
        
        // Progress tracking with minimum visible duration for each stage (1.5s total for good UX)
        let stage_info = [
            ("🏔️ Raising mountains from the depths...", 0.15),      // ~225ms
            ("🌋 Sculpting volcanic peaks...", 0.12),               // ~180ms  
            ("🌡️ Setting perfect temperatures...", 0.1),           // ~150ms
            ("❄️ Adding arctic chill to the north...", 0.1),       // ~150ms
            ("💧 Distributing moisture across lands...", 0.12),     // ~180ms
            ("🌊 Filling rivers and lakes...", 0.1),               // ~150ms
            ("🌍 Shaping diverse biomes...", 0.1),                 // ~150ms
            ("🌿 Planting forests and grasslands...", 0.08),       // ~120ms
            ("💎 Scattering precious minerals...", 0.08),          // ~120ms
            ("🍄 Growing mushrooms in caves...", 0.05),            // ~75ms
            ("✨ Adding final magical touches...", 0.1),           // ~150ms
        ];
        
        let total_target_time: f32 = stage_info.iter().map(|(_, duration)| duration).sum();
        let mut cumulative_times = Vec::new();
        let mut cumulative = 0.0;
        for (_, duration) in &stage_info {
            cumulative += duration;
            cumulative_times.push(cumulative / total_target_time);
        }
        
        // Wrap noise generators in Arc for multi-threading
        let elevation_noise = Arc::new(self.elevation_noise);
        let temperature_noise = Arc::new(self.temperature_noise);
        let moisture_noise = Arc::new(self.moisture_noise);
        let seed = self.seed;
        
        // Progress tracking for multi-threaded environment
        let progress_tracker = Arc::new(Mutex::new((0, generation_start)));
        let current_stage = Arc::new(Mutex::new(0));
        let callback_arc = progress_callback.map(Arc::new);
        
        // Multi-threaded generation using parallel chunks
        let chunk_size = 64; // Process 64x64 chunks in parallel
        let chunks_per_side = (WORLD_SIZE + chunk_size - 1) / chunk_size;
        let total_chunks = chunks_per_side * chunks_per_side;
        
        // Generate chunks in parallel
        let chunk_results: Vec<_> = (0..total_chunks).into_par_iter().map(|chunk_idx| {
            let chunk_x = chunk_idx % chunks_per_side;
            let chunk_y = chunk_idx / chunks_per_side;
            
            let start_x = chunk_x * chunk_size;
            let start_y = chunk_y * chunk_size;
            let end_x = (start_x + chunk_size).min(WORLD_SIZE);
            let end_y = (start_y + chunk_size).min(WORLD_SIZE);
            
            let mut chunk_tiles = Vec::new();
            
            // Pre-compute constants to avoid repeated calculations
            let world_size_f32 = WORLD_SIZE as f32;
            
            for x in start_x..end_x {
                for y in start_y..end_y {
                    // Optimized noise generation with fewer function calls
                    let x_f64 = x as f64;
                    let y_f64 = y as f64;
                    
                    // Inline elevation generation for speed
                    let elevation = {
                        const SCALE: f64 = 0.01;
                        let mut elev = 0.0;
                        let mut amplitude = 1.0;
                        let mut frequency = SCALE;
                        
                        // Reduced octaves for speed (4 -> 2)
                        for _ in 0..2 {
                            elev += elevation_noise.get([x_f64 * frequency, y_f64 * frequency]) as f32 * amplitude;
                            amplitude *= 0.5;
                            frequency *= 2.0;
                        }
                        (elev + 1.0) / 2.0
                    };
                    
                    // Optimized temperature generation
                    let temperature = {
                        const SCALE: f64 = 0.005;
                        let latitude_effect = 1.0 - (y as f32 / world_size_f32);
                        let noise_value = temperature_noise.get([x_f64 * SCALE, y_f64 * SCALE]) as f32;
                        (latitude_effect + noise_value * 0.3).clamp(0.0, 1.0)
                    };
                    
                    // Optimized moisture generation
                    let moisture = {
                        const SCALE: f64 = 0.008;
                        let noise_value = moisture_noise.get([x_f64 * SCALE, y_f64 * SCALE]) as f32;
                        (noise_value + 1.0) / 2.0
                    };
                    
                    let biome = Self::determine_biome_fast(elevation, temperature, moisture);
                    let resources = Self::generate_resources_fast(&biome, seed, x, y);
                    
                    chunk_tiles.push((x, y, Tile {
                        biome,
                        elevation,
                        temperature,
                        moisture,
                        resources,
                    }));
                }
            }
            
            // Update progress periodically
            if let Ok(mut tracker) = progress_tracker.try_lock() {
                tracker.0 += chunk_tiles.len();
                let progress = tracker.0 as f32 / total_tiles as f32;
                
                if let Some(ref callback) = callback_arc {
                    let elapsed = tracker.1.elapsed().as_secs_f32();
                    if elapsed >= 0.05 { // Update every 50ms for better responsiveness
                        if let Ok(mut stage) = current_stage.try_lock() {
                            // Use time-based stage progression for better UX
                            let elapsed_total = generation_start.elapsed().as_secs_f32();
                            let target_elapsed = elapsed_total.max(progress * 1.5); // Minimum 1.5s total
                            
                            // Determine stage based on target elapsed time
                            let mut new_stage = 0;
                            let mut cumulative_target = 0.0;
                            for (i, (_, duration)) in stage_info.iter().enumerate() {
                                cumulative_target += duration;
                                if target_elapsed >= cumulative_target {
                                    new_stage = (i + 1).min(stage_info.len() - 1);
                                } else {
                                    break;
                                }
                            }
                            
                            *stage = new_stage;
                            let (stage_message, _) = stage_info[*stage];
                            
                            // Scale progress to match time-based progression
                            let time_progress = (target_elapsed / total_target_time).min(0.7); // Cap at 70% for world gen
                            callback(time_progress, stage_message);
                            tracker.1 = Instant::now();
                        }
                    }
                }
            }
            
            chunk_tiles
        }).collect();
        
        // Assemble results back into the tiles array
        for chunk_tiles in chunk_results {
            for (x, y, tile) in chunk_tiles {
                tiles[x][y] = tile;
            }
        }
        
        // Final progress update
        if let Some(ref callback) = callback_arc {
            callback(1.0, "✨ Adding final magical touches...");
        }

        WorldMap { tiles, seed: self.seed }
    }
    
    // Fast biome determination without method call overhead
    fn determine_biome_fast(elevation: f32, temperature: f32, moisture: f32) -> BiomeType {
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
    
    // Fast resource generation without allocations when possible
    fn generate_resources_fast(biome: &BiomeType, seed: u32, x: usize, y: usize) -> Vec<ResourceType> {
        // Use position-based deterministic generation instead of thread_rng
        let hash = (seed as u64)
            .wrapping_mul(6364136223846793005)
            .wrapping_add((x as u64) << 16 | (y as u64))
            .wrapping_mul(6364136223846793005);
        
        let available_resources = biome.get_resources();
        if available_resources.is_empty() {
            return Vec::new();
        }
        
        let resource_count = ((hash >> 16) % 3 + 1) as usize;
        let resource_count = resource_count.min(available_resources.len());
        
        available_resources.into_iter().take(resource_count).collect()
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