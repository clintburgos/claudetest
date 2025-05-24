use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BiomeType {
    Ocean,
    Coastal,
    Desert,
    Savanna,
    Grasslands,
    Forest,
    TropicalRainforest,
    Mountain,
    Alpine,
    Tundra,
    Wetlands,
    Caves,
    Volcanic,
    Badlands,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Water,
    Wood,
    Stone,
    Fish,
    Berries,
    Herbs,
    Minerals,
    Salt,
    Ice,
    Mushrooms,
    Clay,
    Sulfur,
}

impl BiomeType {
    pub fn get_color(&self) -> Color {
        match self {
            BiomeType::Ocean => Color::srgb(0.0, 0.3, 0.8),
            BiomeType::Coastal => Color::srgb(0.8, 0.8, 0.6),
            BiomeType::Desert => Color::srgb(0.9, 0.8, 0.4),
            BiomeType::Savanna => Color::srgb(0.7, 0.7, 0.3),
            BiomeType::Grasslands => Color::srgb(0.4, 0.8, 0.3),
            BiomeType::Forest => Color::srgb(0.2, 0.6, 0.2),
            BiomeType::TropicalRainforest => Color::srgb(0.1, 0.4, 0.1),
            BiomeType::Mountain => Color::srgb(0.5, 0.5, 0.5),
            BiomeType::Alpine => Color::srgb(0.7, 0.7, 0.8),
            BiomeType::Tundra => Color::srgb(0.8, 0.9, 0.9),
            BiomeType::Wetlands => Color::srgb(0.3, 0.5, 0.4),
            BiomeType::Caves => Color::srgb(0.2, 0.2, 0.2),
            BiomeType::Volcanic => Color::srgb(0.6, 0.2, 0.1),
            BiomeType::Badlands => Color::srgb(0.6, 0.4, 0.3),
        }
    }

    pub fn get_resources(&self) -> Vec<ResourceType> {
        match self {
            BiomeType::Ocean => vec![ResourceType::Water, ResourceType::Fish, ResourceType::Salt],
            BiomeType::Coastal => vec![ResourceType::Water, ResourceType::Fish, ResourceType::Salt, ResourceType::Clay],
            BiomeType::Desert => vec![ResourceType::Stone, ResourceType::Minerals, ResourceType::Salt],
            BiomeType::Savanna => vec![ResourceType::Herbs, ResourceType::Stone],
            BiomeType::Grasslands => vec![ResourceType::Herbs, ResourceType::Berries],
            BiomeType::Forest => vec![ResourceType::Wood, ResourceType::Berries, ResourceType::Herbs],
            BiomeType::TropicalRainforest => vec![ResourceType::Wood, ResourceType::Berries, ResourceType::Water],
            BiomeType::Mountain => vec![ResourceType::Stone, ResourceType::Minerals, ResourceType::Water],
            BiomeType::Alpine => vec![ResourceType::Stone, ResourceType::Ice, ResourceType::Herbs],
            BiomeType::Tundra => vec![ResourceType::Ice, ResourceType::Fish],
            BiomeType::Wetlands => vec![ResourceType::Water, ResourceType::Clay, ResourceType::Fish],
            BiomeType::Caves => vec![ResourceType::Minerals, ResourceType::Stone, ResourceType::Mushrooms],
            BiomeType::Volcanic => vec![ResourceType::Minerals, ResourceType::Sulfur, ResourceType::Stone],
            BiomeType::Badlands => vec![ResourceType::Stone, ResourceType::Minerals],
        }
    }

    pub fn to_id(&self) -> u8 {
        match self {
            BiomeType::Ocean => 0,
            BiomeType::Coastal => 1,
            BiomeType::Desert => 2,
            BiomeType::Savanna => 3,
            BiomeType::Grasslands => 4,
            BiomeType::Forest => 5,
            BiomeType::TropicalRainforest => 6,
            BiomeType::Mountain => 7,
            BiomeType::Alpine => 8,
            BiomeType::Tundra => 9,
            BiomeType::Wetlands => 10,
            BiomeType::Caves => 11,
            BiomeType::Volcanic => 12,
            BiomeType::Badlands => 13,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            0 => BiomeType::Ocean,
            1 => BiomeType::Coastal,
            2 => BiomeType::Desert,
            3 => BiomeType::Savanna,
            4 => BiomeType::Grasslands,
            5 => BiomeType::Forest,
            6 => BiomeType::TropicalRainforest,
            7 => BiomeType::Mountain,
            8 => BiomeType::Alpine,
            9 => BiomeType::Tundra,
            10 => BiomeType::Wetlands,
            11 => BiomeType::Caves,
            12 => BiomeType::Volcanic,
            13 => BiomeType::Badlands,
            _ => BiomeType::Ocean, // Default fallback
        }
    }

    pub fn can_transition_to(&self, other: &BiomeType) -> bool {
        match (self, other) {
            (BiomeType::Ocean, BiomeType::Coastal) => true,
            (BiomeType::Coastal, BiomeType::Ocean | BiomeType::Grasslands | BiomeType::Wetlands) => true,
            (BiomeType::Desert, BiomeType::Savanna | BiomeType::Badlands) => true,
            (BiomeType::Savanna, BiomeType::Desert | BiomeType::Grasslands) => true,
            (BiomeType::Grasslands, BiomeType::Savanna | BiomeType::Forest | BiomeType::Coastal) => true,
            (BiomeType::Forest, BiomeType::Grasslands | BiomeType::Mountain | BiomeType::TropicalRainforest | BiomeType::Wetlands) => true,
            (BiomeType::TropicalRainforest, BiomeType::Forest | BiomeType::Wetlands) => true,
            (BiomeType::Mountain, BiomeType::Forest | BiomeType::Alpine | BiomeType::Volcanic) => true,
            (BiomeType::Alpine, BiomeType::Mountain | BiomeType::Tundra) => true,
            (BiomeType::Tundra, BiomeType::Alpine | BiomeType::Grasslands) => true,
            (BiomeType::Wetlands, BiomeType::Forest | BiomeType::Coastal | BiomeType::TropicalRainforest) => true,
            (BiomeType::Caves, _) => false, // Caves are underground
            (BiomeType::Volcanic, BiomeType::Mountain | BiomeType::Badlands) => true,
            (BiomeType::Badlands, BiomeType::Desert | BiomeType::Volcanic) => true,
            _ => false,
        }
    }
}