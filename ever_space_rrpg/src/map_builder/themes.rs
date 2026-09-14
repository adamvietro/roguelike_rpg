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

    /// Row 1 (of 3) on `resources/battle_backgrounds.png` - see
    /// `MapTheme::battle_background_row`'s own doc comment.
    fn battle_background_row(&self) -> Option<u16> {
        Some(1)
    }

    /// Explicit even though it matches `MapTheme::end_scene_theme`'s own
    /// default - consistency with every other per-theme override in this
    /// file, and resilience if that default ever changes.
    fn end_scene_theme(&self) -> EndSceneTheme {
        EndSceneTheme::Dungeon
    }

    /// Dungeon's thin top wall band leaves real headroom Forest's
    /// tree/fence perimeter doesn't - screenshot-verified against the
    /// real art (a candidate back row as high as 0.9 clipped the window
    /// sill/torch/crate near the top; 1.5/1.9 sits clear of all of them).
    fn enemy_formation_rows(&self) -> (f32, f32) {
        (1.5, 1.9)
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

    /// Row 0 (of 3) on `resources/battle_backgrounds.png` - see
    /// `MapTheme::battle_background_row`'s own doc comment.
    fn battle_background_row(&self) -> Option<u16> {
        Some(0)
    }

    fn end_scene_theme(&self) -> EndSceneTheme {
        EndSceneTheme::Forest
    }

    /// Variant 4 (Dirt path, cell 9) / 5 (Path fork, cell 10) - see
    /// `MapTheme::path_variants`'s own doc comment. Previously left on
    /// the default `Patch` treatment like every other floor variant,
    /// which is exactly what produced a random circular blob instead of
    /// a real connected line (2026-09-13 - "the path tiles should be in
    /// a line").
    fn path_variants(&self) -> Option<(u8, u8)> {
        Some((4, 5))
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

    /// Row 2 (of 3) on `resources/battle_backgrounds.png` - see
    /// `MapTheme::battle_background_row`'s own doc comment.
    fn battle_background_row(&self) -> Option<u16> {
        Some(2)
    }

    fn end_scene_theme(&self) -> EndSceneTheme {
        EndSceneTheme::Sewer
    }

    /// Sewer's thin top pipe/wall band leaves the same real headroom
    /// Dungeon's does - screenshot-verified clear at 1.5/1.9 with room
    /// to spare (no nearby fixtures at all near the top, unlike
    /// Dungeon's window/torch/crate).
    fn enemy_formation_rows(&self) -> (f32, f32) {
        (1.5, 1.9)
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
