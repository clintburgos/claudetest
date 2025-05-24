# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Creature simulation built in Rust using the Bevy game engine. Features auto-generated worlds with multiple biomes where creatures will interact, reproduce, and evolve. The simulation is designed to run quickly with detailed UI in a new window.

## Development Commands

- `cargo check` - Compile check
- `cargo run` - Run the simulation 
- `cargo build --release` - Build optimized version

## Architecture

**Core Modules:**
- `biome.rs` - Biome types, colors, resources, and transition rules
- `world.rs` - World generation using Perlin noise for elevation/temperature/moisture
- `render.rs` - Bevy rendering system for tiles and camera controls
- `main.rs` - App setup and resource initialization

**Dependencies:**
- `bevy = "0.14"` - Game engine for rendering and ECS
- `noise = "0.9"` - Perlin noise for procedural generation
- `rand = "0.8"` - Random number generation

**World Generation System:**
- 1000x1000 tile grid with procedural generation
- Uses multi-octave Perlin noise for realistic terrain
- Biomes determined by elevation, temperature, and moisture values
- Realistic biome transitions matching real-world patterns
- Each tile stores biome type, elevation, temperature, moisture, and resources

**Biome System (14 types):**
1. **Ocean** - Low elevation, water/fish/salt resources
2. **Coastal** - Shore areas, water/fish/salt/clay resources
3. **Desert** - Hot/dry, stone/minerals/salt resources
4. **Savanna** - Hot/moderate moisture, herbs/stone resources
5. **Grasslands** - Moderate temp/moisture, herbs/berries resources
6. **Forest** - Moderate temp/good moisture, wood/berries/herbs resources
7. **TropicalRainforest** - Hot/wet, wood/berries/water resources
8. **Mountain** - High elevation, stone/minerals/water resources
9. **Alpine** - Very high elevation/cold, stone/ice/herbs resources
10. **Tundra** - Very cold, ice/fish resources
11. **Wetlands** - High moisture, water/clay/fish resources
12. **Caves** - Underground systems, minerals/stone/mushrooms resources
13. **Volcanic** - Hot/high elevation, minerals/sulfur/stone resources
14. **Badlands** - Extreme conditions, stone/minerals resources

**Biome Transition Rules:**
- Ocean ↔ Coastal
- Desert ↔ Savanna ↔ Grasslands ↔ Forest
- Mountain ↔ Alpine ↔ Tundra
- Forest ↔ TropicalRainforest ↔ Wetlands
- Mountain ↔ Volcanic ↔ Badlands
- Caves are isolated (underground)

**Resource Types:**
- Water, Wood, Stone, Fish, Berries, Herbs, Minerals, Salt, Ice, Mushrooms, Clay, Sulfur

**Rendering System:**
- Tile-based rendering with 4x4 pixel tiles
- Color-coded biomes for easy identification
- Camera system with WASD/arrow key movement (300 units/second)
- Real-time world rendering using Bevy's sprite system

**Controls:**
- WASD or arrow keys to move camera around the world
- Camera can explore the entire 1000x1000 world

## Future Creature Implementation (Planned)

**Decision-Making Systems:**
- **Needs-based behavior**: Hunger, thirst, safety, reproduction drives
- **Neural networks**: Simple AI that evolves over generations
- **State machines**: Different behaviors (foraging, mating, fleeing, resting)
- **Genetic traits**: Aggression, curiosity, social tendencies encoded in DNA

**World Interactions:**
- **Resource gathering**: Food sources that deplete and regenerate
- **Territory marking**: Scent trails, claimed areas
- **Environmental adaptation**: Seasonal changes, weather effects
- **Tool use**: Simple object manipulation for shelter/food

**Social Behaviors:**
- **Social hierarchies**: Pack behavior, dominance systems
- **Cooperation**: Hunting together, sharing resources
- **Communication**: Simple signals for danger, food location
- **Competition**: Fighting over mates, territory, resources

**Reproduction & Evolution:**
- **Mate selection**: Based on fitness, traits, territory quality
- **Genetic mixing**: Traits combine and mutate in offspring
- **Parental care**: Protecting young affects survival rates
- **Natural selection**: Successful traits become more common

**Death Mechanics:**
- **Age-based**: Natural lifespan with declining abilities
- **Resource-based**: Starvation, dehydration
- **Predation**: Creature vs creature combat
- **Environmental**: Disease, weather extremes, natural disasters

**Advanced Features (Future):**
- **Seasonal cycles**: Affect resource availability and creature behavior
- **Weather systems**: Rain, drought, temperature variations
- **Migration patterns**: Creatures moving between biomes seasonally
- **Ecosystem balance**: Predator-prey relationships, food chains
- **Cultural evolution**: Learned behaviors passed between generations
- **Disease systems**: Illness spreading through populations
- **Territory systems**: Creatures claiming and defending areas
- **Tool creation**: Advanced creatures making simple tools
- **Social learning**: Creatures learning from observing others

## Technical Considerations

**Performance Optimizations (Future):**
- Chunk-based rendering for large worlds
- Level-of-detail systems for distant areas
- Spatial partitioning for creature interactions
- Multi-threading for simulation updates

**UI Framework Choice:**
- **Bevy** chosen for ECS architecture and game simulation capabilities
- Alternative frameworks considered: egui (immediate mode), Tauri (web frontend), iced (reactive), slint (declarative)
- Bevy provides integrated rendering, physics, and entity management

**Data Structures:**
- World stored as 2D vector of Tiles
- Each tile contains biome type and environmental data
- Resource system tracks available materials per biome
- Future creature system will use ECS components

**Performance Optimizations:**
- Chunk-based rendering (32x32 tile chunks) with frustum culling
- Distance-based LOD system (4 levels) for animations and detail
- Spatial hashing for O(1) neighbor queries and creature interactions
- GPU instancing framework for repeated environmental elements
- Compressed world data storage (87.5% memory reduction)
- Shared animation timers with realistic wind simulation
- Multi-threaded world generation (non-blocking startup)
- Integrated optimization pipeline with configurable parameters
- Expected performance: 20-200x improvement, 60+ FPS with thousands of entities

See PERFORMANCE_OPTIMIZATIONS.md for detailed technical documentation.