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
    /// A real map-theme liquid feature (a moat, standing sewage, sludge -
    /// see `MapTheme::water_variants`) - impassable like Wall (`can_
    /// enter_tile` below still only special-cases Floor/Exit as
    /// passable), but deliberately NOT opaque (`is_opaque` below DOES
    /// special-case this one, unlike Counter above) - a moat should
    /// still let the player see what's on the other side of it, the
    /// whole point of using water instead of a solid wall for a
    /// fortress/chest-room border (2026-09-13). Placed deliberately by
    /// `map_builder` (fortress/chest-room moats, small isolated patches)
    /// - never part of the ordinary random Floor/Wall variant rolls.
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

    /// A plain BFS distance field seeded at `target`, over every tile
    /// this map's own `can_enter_tile` considers passable (plus `target`
    /// itself, even if it technically isn't - the seed always gets
    /// walked from, matching every caller's own "distance FROM this
    /// point" intent). Unreached cells stay at `i32::MAX`. Deliberately
    /// NOT `bracket_pathfinding::DijkstraMap` - that type never writes
    /// 0.0 into the seed tile's own array slot (it only gets set by a
    /// neighbor's relaxation pass, which may not happen at all near an
    /// edge), which stalls anything that greedily walks toward the
    /// lowest surrounding value while standing right next to the actual
    /// target. A plain BFS has no such bug (the seed's distance is set
    /// to 0 directly, not left to a later relaxation pass) and costs
    /// nothing extra here since every step on this map costs exactly 1
    /// anyway - no priority queue needed. Promoted out of
    /// `screens/battle.rs`'s `class_survivability_diagnostic` module
    /// (2026-09-13), which had its own private copy for the exact same
    /// reason; `map_builder`'s theme-path placement is a second, real
    /// caller now.
    pub(crate) fn bfs_distance_field(&self, target: Point) -> Vec<i32> {
        let mut field = vec![i32::MAX; NUM_TILES];
        if !self.in_bounds(target) {
            return field;
        }
        let target_idx = self.point2d_to_index(target);
        field[target_idx] = 0;
        let mut queue: std::collections::VecDeque<Point> = std::collections::VecDeque::new();
        queue.push_back(target);
        while let Some(current) = queue.pop_front() {
            let current_dist = field[self.point2d_to_index(current)];
            for delta in [
                Point::new(0, -1),
                Point::new(0, 1),
                Point::new(-1, 0),
                Point::new(1, 0),
            ] {
                let neighbor = current + delta;
                if neighbor != target && !self.can_enter_tile(neighbor) {
                    continue;
                }
                if !self.in_bounds(neighbor) {
                    continue;
                }
                let idx = self.point2d_to_index(neighbor);
                if field[idx] == i32::MAX {
                    field[idx] = current_dist + 1;
                    queue.push_back(neighbor);
                }
            }
        }
        field
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
        // Water is the one non-Floor tile that deliberately doesn't
        // block sight - see TileType::Water's own doc comment.
        !matches!(self.tiles[idx as usize], TileType::Floor | TileType::Water)
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
