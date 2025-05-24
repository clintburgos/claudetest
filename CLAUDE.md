# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Creature simulation built in Rust using the Bevy game engine. Features auto-generated worlds with multiple biomes where creatures will interact, reproduce, and evolve.

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

**World Generation:**
- 1000x1000 tile grid with procedural generation
- Biomes determined by elevation, temperature, and moisture values
- 14 biome types: Ocean, Coastal, Desert, Savanna, Grasslands, Forest, TropicalRainforest, Mountain, Alpine, Tundra, Wetlands, Caves, Volcanic, Badlands
- Each biome has specific resource types and color coding

**Controls:**
- WASD or arrow keys to move camera around the world

## Future Implementation Ideas

Saved for later implementation:
- Needs-based creature decision making (hunger, thirst, safety, reproduction)
- Neural networks that evolve over generations
- Social behaviors (cooperation, hierarchies, communication)
- Environmental interactions (tool use, territory marking)
- Genetic traits and mutation system
- Age-based death and resource-based survival