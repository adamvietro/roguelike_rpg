use crate::prelude::*;

const NUM_TILES: usize = (SCREEN_WIDTH * SCREEN_HEIGHT) as usize;

#[derive(Copy, Clone, PartialEq)]
pub enum TileType {
    Wall,
    Floor,
    Exit,
    /// The Battle Arena shop's counter - impassable exactly like Wall
    /// (can_enter_tile/is_opaque below only special-case Floor/Exit, so
    /// this is blocked/opaque automatically, no extra logic needed), but
    /// rendered as a distinct warm bar (see tile_render_at) instead of
    /// theme brick, so shop items don't read as text stuck in a wall.
    Counter,
    /// A real map-theme "special wall" feature (see
    /// docs/Map_Tile_Theme_Guide.md) - impassable exactly like Wall, same
    /// zero-extra-logic reasoning as Counter above (can_enter_tile/
    /// is_opaque only special-case Floor/Exit as passable/see-through).
    /// Not placed by any map generator yet - deliberate, targeted
    /// placement (a river, a lone obstacle) is deferred to a later
    /// design pass; this variant exists now so the render/data-model
    /// side is ready whenever that placement logic arrives.
    Water,
}

pub fn map_idx(x: i32, y: i32) -> usize {
    ((y * SCREEN_WIDTH) + x) as usize
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub revealed_tiles: Vec<bool>,
    /// Which specific texture (within its TileType's variant pool) each
    /// tile shows, for a theme with real per-tile art (see
    /// MapTheme::tile_row / docs/Map_Tile_Theme_Guide.md) - meaningless
    /// (left at 0) for a theme still on the old single-glyph rendering,
    /// and for any TileType without a variant pool. Rolled once at map-
    /// generation time (see map_builder::assign_tile_variants) and
    /// stored here rather than re-rolled on every draw, so a given
    /// tile's texture stays the same from frame to frame instead of
    /// flickering between variants.
    pub tile_variant: Vec<u8>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            tiles: vec![TileType::Floor; NUM_TILES],
            revealed_tiles: vec![false; NUM_TILES],
            tile_variant: vec![0; NUM_TILES],
        }
    }

    pub fn in_bounds(&self, point: Point) -> bool {
        point.x >= 0 && point.x < SCREEN_WIDTH && point.y >= 0 && point.y < SCREEN_HEIGHT
    }

    pub fn try_idx(&self, point: Point) -> Option<usize> {
        if !self.in_bounds(point) {
            None
        } else {
            Some(map_idx(point.x, point.y))
        }
    }

    pub fn can_enter_tile(&self, point: Point) -> bool {
        self.in_bounds(point)
            && (self.tiles[map_idx(point.x, point.y)] == TileType::Floor
                || self.tiles[map_idx(point.x, point.y)] == TileType::Exit)
    }

    fn valid_exit(&self, loc: Point, delta: Point) -> Option<usize> {
        let destination = loc + delta;
        if self.in_bounds(destination) {
            if self.can_enter_tile(destination) {
                let idx = self.point2d_to_index(destination);
                Some(idx)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(SCREEN_WIDTH, SCREEN_HEIGHT)
    }

    fn in_bounds(&self, point: Point) -> bool {
        self.in_bounds(point)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] != TileType::Floor
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let location = self.index_to_point2d(idx);

        if let Some(idx) = self.valid_exit(location, Point::new(-1, 0)) {
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(1, 0)) {
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(0, -1)) {
            exits.push((idx, 1.0))
        }
        if let Some(idx) = self.valid_exit(location, Point::new(0, 1)) {
            exits.push((idx, 1.0))
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        DistanceAlg::Pythagoras.distance2d(self.index_to_point2d(idx1), self.index_to_point2d(idx2))
    }
}
