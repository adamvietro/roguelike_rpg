use crate::prelude::*;
use std::collections::HashSet;

/// Which sprite sheet/console a tile_render_at glyph is a cell in - the
/// tile-rendering equivalent of IdleSpriteSheet, needed for the same
/// reason: map_render.rs has to route each tile's draw call to whichever
/// console actually holds that glyph's font (MAP_TILE_CONSOLE's trio for
/// `MapTiles`, the plain dungeonfont ones for `Dungeon`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSpriteSheet {
    Dungeon,
    MapTiles,
}

/// Flat brightness multiplier applied to real-texture WALL tiles only
/// (see tile_render_at) - confirmed needed in real play, 2026-09-08:
/// wall and floor variants at the same brightness read as too visually
/// similar for some themes (Sewer specifically). Tune this if a future
/// theme still reads too flat even with it applied.
const WALL_TEXTURE_SHADE: f32 = 0.72;

/// The exact `resources/map_tiles.png` glyph for a Floor/Wall tile,
/// given its theme's starting row (`MapTheme::tile_row`) and its own
/// rolled variant (`Map::tile_variant`) - `None` for any TileType
/// without a variant pool yet (Exit/Counter/Water), so the caller falls
/// through to the old dungeonfont rendering for those. See
/// docs/Map_Tile_Theme_Guide.md for the row layout this encodes: row
/// `base_row + 0` is basic floor, `+1` wall, `+2` themed floor (floor's
/// variant pool spans both `+0` and `+2`, `variant` 0..FLOOR_VARIANT_COUNT
/// picks between them), `+3` special wall (not wired up yet).
fn map_tile_glyph(base_row: u16, tile: TileType, variant: u8) -> Option<FontCharType> {
    let (row_offset, col) = match tile {
        TileType::Floor if variant < MAP_TILE_COLS as u8 => (0, variant as u16),
        TileType::Floor => (2, (variant - MAP_TILE_COLS as u8) as u16),
        TileType::Wall => (1, variant as u16),
        TileType::Exit | TileType::Counter | TileType::Water => return None,
    };
    Some((base_row + row_offset) * MAP_TILE_COLS + col)
}

/// Computes the same ColorPair/glyph/sheet a tile would be drawn with in
/// map_render.rs, for a single point - shared so entity_render can paint
/// the real floor/wall tile underneath a mid-glide entity (see
/// systems/entity_render.rs) instead of duplicating this logic, and so
/// the two never drift apart. Returns None if the tile is out of bounds
/// or has never been seen (nothing should be drawn there).
pub fn tile_render_at(
    map: &Map,
    theme: &dyn MapTheme,
    visible_tiles: &HashSet<Point>,
    pt: Point,
) -> Option<(ColorPair, FontCharType, TileSpriteSheet)> {
    if !map.in_bounds(pt) {
        return None;
    }
    let idx = map_idx(pt.x, pt.y);
    if !(visible_tiles.contains(&pt) || map.revealed_tiles[idx]) {
        return None;
    }
    let visible = visible_tiles.contains(&pt);
    let tile = map.tiles[idx];

    // Real per-tile texture path - see map_tile_glyph's own doc comment
    // for which TileTypes actually have art yet. Every real tile texture
    // is fully opaque, so "color" is mostly just the same visible/
    // remembered brightness multiply Floor/Exit's old plain path already
    // used (WHITE = unchanged, DARK_GRAY = dimmed). Walls additionally
    // get a flat darkening multiply on top of that (WALL_TEXTURE_SHADE) -
    // confirmed needed in real play (2026-09-08, Sewer specifically):
    // without any per-type tint, wall and floor variants painted at the
    // same brightness read as too visually similar when the art itself
    // doesn't have enough inherent contrast, the same "hard to tell
    // floor from wall" complaint the patch/accent generation algorithm
    // was built to fix on the LAYOUT side - this is the color-side half
    // of the same problem. Applied as a post-multiply here rather than
    // baked into the art, so it benefits every theme uniformly (not just
    // the one that surfaced it) without needing new art. Safe against
    // the `_no_bg` near-black cutoff (MAP_TILE_CONSOLE's own gotcha,
    // above) because that check runs on the source texture's own raw
    // color, before this multiply ever applies.
    if let Some(base_row) = theme.tile_row() {
        if let Some(glyph) = map_tile_glyph(base_row, tile, map.tile_variant[idx]) {
            let (wr, wg, wb) = if visible { WHITE } else { DARK_GRAY };
            let base_tint = RGB::from_u8(wr, wg, wb);
            let tint = if tile == TileType::Wall {
                RGB::from_f32(
                    base_tint.r * WALL_TEXTURE_SHADE,
                    base_tint.g * WALL_TEXTURE_SHADE,
                    base_tint.b * WALL_TEXTURE_SHADE,
                )
            } else {
                base_tint
            };
            return Some((ColorPair::new(tint, BLACK), glyph, TileSpriteSheet::MapTiles));
        }
    }

    let glyph = theme.tile_to_render(tile);
    let wall_base = theme.wall_color();

    let color_pair = if tile == TileType::Wall {
        let bg = if visible {
            wall_base
        } else {
            RGB::from_f32(wall_base.r * 0.35, wall_base.g * 0.35, wall_base.b * 0.35)
        };
        let fg = RGB::from_f32(
            (bg.r * 1.4).min(1.0),
            (bg.g * 1.4).min(1.0),
            (bg.b * 1.4).min(1.0),
        );
        ColorPair::new(fg, bg)
    } else if tile == TileType::Counter {
        // The Battle Arena shop's counter - a warm red bar, deliberately
        // distinct from both the theme's wall and floor colors, so shop
        // items sitting on it read as "on a counter" rather than "text
        // embedded in a generic brick wall" (the counter's first version
        // just reused TileType::Wall for this, which looked like the
        // latter).
        let bright = RGB::from_f32(0.55, 0.12, 0.12);
        let fg = if visible {
            bright
        } else {
            RGB::from_f32(bright.r * 0.35, bright.g * 0.35, bright.b * 0.35)
        };
        ColorPair::new(fg, BLACK)
    } else {
        let tint = if visible { WHITE } else { DARK_GRAY };
        ColorPair::new(tint, BLACK)
    };

    Some((color_pair, glyph, TileSpriteSheet::Dungeon))
}

