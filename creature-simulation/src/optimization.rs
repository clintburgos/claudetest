use bevy::prelude::*;
use std::collections::HashMap;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use std::sync::{Arc, Mutex};
use crate::world::{WorldMap, WorldGenerator, WORLD_SIZE};
use crate::environment::EnvironmentType;

// === CHUNK SYSTEM ===
pub const CHUNK_SIZE: usize = 32;
pub const RENDER_DISTANCE: f32 = 200.0; // Reduced for testing

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub loaded_chunks: HashMap<(i32, i32), ChunkData>,
    pub active_chunks: Vec<(i32, i32)>,
}

#[derive(Default)]
pub struct ChunkData {
    pub entities: Vec<Entity>,
    pub is_loaded: bool,
}

// === LOD SYSTEM ===
#[derive(Component)]
pub struct LODLevel(pub u8); // 0 = highest detail, 3 = lowest

// === SPATIAL HASHING ===
#[derive(Resource)]
pub struct SpatialHash {
    pub cell_size: f32,
    pub grid: HashMap<(i32, i32), Vec<Entity>>,
}

impl Default for SpatialHash {
    fn default() -> Self {
        Self {
            cell_size: 64.0, // Adjust based on typical interaction radius
            grid: HashMap::new(),
        }
    }
}

impl SpatialHash {
    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let cell = self.world_to_cell(position);
        self.grid.entry(cell).or_default().push(entity);
    }
    
    pub fn remove(&mut self, entity: Entity, position: Vec3) {
        let cell = self.world_to_cell(position);
        if let Some(entities) = self.grid.get_mut(&cell) {
            entities.retain(|&e| e != entity);
        }
    }
    
    pub fn get_nearby(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let mut nearby = Vec::new();
        let min_cell = self.world_to_cell(position - Vec3::splat(radius));
        let max_cell = self.world_to_cell(position + Vec3::splat(radius));
        
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.grid.get(&(x, y)) {
                    nearby.extend(entities);
                }
            }
        }
        nearby
    }
    
    pub fn clear(&mut self) {
        self.grid.clear();
    }
    
    fn world_to_cell(&self, position: Vec3) -> (i32, i32) {
        ((position.x / self.cell_size) as i32, (position.y / self.cell_size) as i32)
    }
}

// === COMPRESSED WORLD DATA ===
#[derive(Resource)]
pub struct CompressedWorldData {
    pub biomes: Vec<u8>, // 4 bits per biome
    pub elevation_samples: Vec<f32>, // Sparse sampling
    pub temperature_samples: Vec<f32>,
    pub moisture_samples: Vec<f32>,
    pub sample_resolution: usize, // Every Nth tile is sampled
}

impl CompressedWorldData {
    pub fn from_world_map(world_map: &WorldMap) -> Self {
        let sample_resolution = 8; // Sample every 8th tile
        let mut biomes = Vec::new();
        let mut elevation_samples = Vec::new();
        let mut temperature_samples = Vec::new();
        let mut moisture_samples = Vec::new();

        // Pack biomes (2 per byte)
        for x in 0..WORLD_SIZE {
            for y in (0..WORLD_SIZE).step_by(2) {
                let biome1 = world_map.tiles[x][y].biome.to_id();
                let biome2 = if y + 1 < WORLD_SIZE {
                    world_map.tiles[x][y + 1].biome.to_id()
                } else {
                    0
                };
                let packed = (biome2 << 4) | biome1;
                biomes.push(packed);
            }
        }

        // Sample environmental data sparsely
        for x in (0..WORLD_SIZE).step_by(sample_resolution) {
            for y in (0..WORLD_SIZE).step_by(sample_resolution) {
                let tile = &world_map.tiles[x][y];
                elevation_samples.push(tile.elevation);
                temperature_samples.push(tile.temperature);
                moisture_samples.push(tile.moisture);
            }
        }

        Self {
            biomes,
            elevation_samples,
            temperature_samples,
            moisture_samples,
            sample_resolution,
        }
    }

    pub fn get_biome(&self, x: usize, y: usize) -> u8 {
        let index = x * WORLD_SIZE + y;
        let byte_index = index / 2;
        let is_upper_nibble = index % 2 == 1;
        
        if byte_index >= self.biomes.len() {
            return 0;
        }
        
        let packed_byte = self.biomes[byte_index];
        if is_upper_nibble {
            (packed_byte >> 4) & 0xF
        } else {
            packed_byte & 0xF
        }
    }

    pub fn get_elevation(&self, x: usize, y: usize) -> f32 {
        let sample_x = (x / self.sample_resolution).min(self.elevation_samples.len() - 1);
        let sample_y = (y / self.sample_resolution).min(self.elevation_samples.len() - 1);
        // Simple lookup - could be improved with interpolation
        self.elevation_samples.get(sample_x * sample_y).copied().unwrap_or(0.0)
    }
}

// === SHARED ANIMATION STATE ===
#[derive(Resource)]
pub struct SharedAnimationState {
    pub wind_time: f32,
    pub wind_strength: f32,
    pub wind_direction: Vec2,
}

impl Default for SharedAnimationState {
    fn default() -> Self {
        Self {
            wind_time: 0.0,
            wind_strength: 1.0,
            wind_direction: Vec2::new(1.0, 0.0),
        }
    }
}

// === INSTANCED RENDERING ===
#[derive(Component)]
pub struct InstancedSprites {
    pub element_type: EnvironmentType,
    pub positions: Vec<Vec3>,
    pub rotations: Vec<f32>,
    pub scales: Vec<Vec2>,
}

// === ASYNC WORLD GENERATION ===
#[derive(Component)]
pub struct WorldGenerationTask {
    pub task: Task<WorldMap>,
    pub progress_tracker: Arc<Mutex<(f32, String)>>,
}

// === UTILITY FUNCTIONS ===
pub fn calculate_visible_chunks(camera_pos: Vec3) -> Vec<(i32, i32)> {
    let tile_size = 4.0; // From render.rs
    let chunk_x = (camera_pos.x / (CHUNK_SIZE as f32 * tile_size)) as i32;
    let chunk_y = (camera_pos.y / (CHUNK_SIZE as f32 * tile_size)) as i32;
    let render_chunks = (RENDER_DISTANCE / (CHUNK_SIZE as f32 * tile_size)) as i32 + 1;
    
    let mut visible_chunks = Vec::new();
    for x in (chunk_x - render_chunks)..=(chunk_x + render_chunks) {
        for y in (chunk_y - render_chunks)..=(chunk_y + render_chunks) {
            visible_chunks.push((x, y));
        }
    }
    visible_chunks
}

pub fn world_to_chunk_coord(world_x: usize, world_y: usize) -> (i32, i32) {
    ((world_x / CHUNK_SIZE) as i32, (world_y / CHUNK_SIZE) as i32)
}

pub fn chunk_to_world_bounds(chunk_x: i32, chunk_y: i32) -> (usize, usize, usize, usize) {
    // Calculate bounds in i32 first to handle negative coordinates properly
    let start_x_i32 = chunk_x * CHUNK_SIZE as i32;
    let start_y_i32 = chunk_y * CHUNK_SIZE as i32;
    let end_x_i32 = (chunk_x + 1) * CHUNK_SIZE as i32;
    let end_y_i32 = (chunk_y + 1) * CHUNK_SIZE as i32;
    
    // Clamp to valid world bounds and convert to usize
    let start_x = start_x_i32.max(0).min(WORLD_SIZE as i32) as usize;
    let start_y = start_y_i32.max(0).min(WORLD_SIZE as i32) as usize;
    let end_x = end_x_i32.max(0).min(WORLD_SIZE as i32) as usize;
    let end_y = end_y_i32.max(0).min(WORLD_SIZE as i32) as usize;
    
    (start_x, start_y, end_x, end_y)
}