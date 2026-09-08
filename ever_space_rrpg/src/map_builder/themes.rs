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
            // Not placed by any map generator yet - see TileType::Water's
            // own doc comment. Only reached if that ever changes before
            // this theme has a real tile_row, or for a theme that never
            // gets one.
            TileType::Water => to_cp437('~'),
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

    /// Dungeon is the second theme migrated to real tile art (2026-09-08)
    /// - owns rows 4-7 of resources/map_tiles.png. See
    /// docs/Map_Tile_Theme_Guide.md for the full row-by-row breakdown.
    fn tile_row(&self) -> Option<u16> {
        Some(4)
    }

    /// Every one of Dungeon's row-3 (themed floor) cells - drain grate,
    /// bones/debris, cracked pillar base, torchlight glow - is a
    /// discrete point fixture, not a spreadable ground cover. Confirmed
    /// needed in real play (2026-09-08): torchlight glow patched as a
    /// whole region looked like a wall of torches. All four Scatter,
    /// none Patch.
    fn floor_variant_style(&self, variant: u8) -> VariantStyle {
        match variant {
            4..=7 => VariantStyle::Scatter,
            _ => VariantStyle::Patch,
        }
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
            // Not placed by any map generator yet - see TileType::Water's
            // own doc comment. Only reached if that ever changes before
            // this theme has a real tile_row, or for a theme that never
            // gets one.
            TileType::Water => to_cp437('~'),
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

    /// Forest is the first theme migrated to real tile art (2026-09-08) -
    /// owns rows 0-3 of resources/map_tiles.png. See
    /// docs/Map_Tile_Theme_Guide.md for the full row-by-row breakdown.
    fn tile_row(&self) -> Option<u16> {
        Some(0)
    }
}

impl ForestTheme {
    pub fn new() -> Box<dyn MapTheme> {
        Box::new(Self {})
    }
}

pub struct SewerTheme {}

impl MapTheme for SewerTheme {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType {
        match tile_type {
            TileType::Floor => to_cp437(','),
            TileType::Wall => to_cp437('%'),
            TileType::Exit => to_cp437('>'),
            TileType::Counter => to_cp437('▄'),
            // Not placed by any map generator yet - see TileType::Water's
            // own doc comment. Only reached if that ever changes before
            // this theme has a real tile_row, or for a theme that never
            // gets one.
            TileType::Water => to_cp437('~'),
        }
    }

    fn floor_color(&self) -> RGB {
        // Murky, damp stone-green.
        RGB::from_f32(0.16, 0.20, 0.17)
    }

    fn wall_color(&self) -> RGB {
        // Rust-brown against the greenish floor.
        RGB::from_f32(0.30, 0.22, 0.16)
    }

    fn battle_scenery(&self) -> BattleScenery {
        BattleScenery::RoomWalls
    }

    /// Sewer is the third theme migrated to real tile art (2026-09-08) -
    /// owns rows 9-12 of resources/map_tiles.png, NOT 8-11 - row 8 is
    /// this sheet's permanently forbidden row (see
    /// docs/Map_Tile_Theme_Guide.md's glyph-32 gotcha), so the third
    /// theme's block skips it entirely rather than starting there.
    fn tile_row(&self) -> Option<u16> {
        Some(9)
    }

    /// Row-3 (themed floor) cells 9-10 (drainage grate, rubble/debris)
    /// are discrete point fixtures - Scatter, same reasoning as
    /// Dungeon's own override. Cells 11-12 (shallow puddle, glowing
    /// fungus/algae patch) genuinely read as spreadable ground cover -
    /// left as the default Patch.
    fn floor_variant_style(&self, variant: u8) -> VariantStyle {
        match variant {
            4 | 5 => VariantStyle::Scatter,
            _ => VariantStyle::Patch,
        }
    }
}

impl SewerTheme {
    pub fn new() -> Box<dyn MapTheme> {
        Box::new(Self {})
    }
}
