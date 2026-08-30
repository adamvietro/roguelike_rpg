use crate::prelude::*;

pub struct DungeonTheme {}

impl DungeonTheme {
    pub fn new() -> Box<dyn MapTheme> {
        Box::new(Self {})
    }
}

impl MapTheme for DungeonTheme {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType {
        match tile_type {
            TileType::Floor => to_cp437('.'),
            TileType::Wall => to_cp437('#'),
            TileType::Exit => to_cp437('>'),
            TileType::Counter => to_cp437('▄'),
        }
    }

    fn floor_color(&self) -> RGB {
        // Cool dark slate - stone dungeon floor.
        RGB::from_f32(0.20, 0.20, 0.25)
    }

    fn wall_color(&self) -> RGB {
        // Warmer, lighter stone - reads as a wall against the cooler floor.
        RGB::from_f32(0.34, 0.30, 0.26)
    }

    fn battle_scenery(&self) -> BattleScenery {
        BattleScenery::RoomWalls
    }
}

pub struct ForestTheme {}

impl MapTheme for ForestTheme {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType {
        match tile_type {
            TileType::Floor => to_cp437(';'),
            TileType::Wall => to_cp437('"'),
            TileType::Exit => to_cp437('>'),
            TileType::Counter => to_cp437('▄'),
        }
    }

    fn floor_color(&self) -> RGB {
        // Dark mossy green undergrowth.
        RGB::from_f32(0.14, 0.22, 0.14)
    }

    fn wall_color(&self) -> RGB {
        // Deep bark brown - treeline framing the floor.
        RGB::from_f32(0.26, 0.20, 0.14)
    }

    fn battle_scenery(&self) -> BattleScenery {
        BattleScenery::ScatteredTrees
    }
}

impl ForestTheme {
    pub fn new() -> Box<dyn MapTheme> {
        Box::new(Self {})
    }
}
