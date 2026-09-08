use crate::prelude::*;
mod empty;
mod rooms;
use rooms::RoomsArchitect;
mod automata;
use automata::CellularAutomataArchitect;
mod drunkard;
use drunkard::DrunkardsWalkArchitect;
mod prefab;
use prefab::{apply_chest, apply_prefab};
mod themes;
pub use themes::*;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

/// What kind of scenery a theme's battle arena should get, beyond the
/// floor/wall tint every theme has. New themes pick whichever fits - a
/// theme isn't limited to these two forever, this just covers what's
/// implemented in battle_tick so far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleScenery {
    /// A handful of large tree/foliage silhouettes scattered around the
    /// arena (outdoor themes).
    ScatteredTrees,
    /// Thick stone walls running down the left/right sides, so the arena
    /// reads as an enclosed room (indoor/dungeon themes).
    RoomWalls,
}

pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
    /// Base color for this theme's floor tiles - used both for the dungeon
    /// view (future use) and to tint the battle screen's arena background.
    fn floor_color(&self) -> RGB;
    /// Base color for this theme's wall tiles - used to frame the battle
    /// screen's arena background.
    fn wall_color(&self) -> RGB;
    /// Which battle arena scenery this theme uses.
    fn battle_scenery(&self) -> BattleScenery;
    /// This theme's starting row on the shared `resources/map_tiles.png`
    /// atlas - `None` for a theme still on the old single-glyph
    /// `tile_to_render` rendering (see docs/Map_Tile_Theme_Guide.md).
    /// Every migrated theme owns a fixed 4-row block there (floor/wall/
    /// themed-floor/special-wall, MAP_TILE_COLS columns each) - the same
    /// "one shared sheet, more rows" shape CHARACTER_IDLE_CONSOLE/
    /// ENEMY_IDLE_CONSOLE already use, just keyed by theme instead of by
    /// class/enemy name. Defaults to `None` so a brand new MapTheme impl
    /// doesn't have to know about this system until it actually gets
    /// real tile art.
    fn tile_row(&self) -> Option<u16> {
        None
    }
    /// Which placement style floor variant `variant` (1..FLOOR_VARIANT_
    /// COUNT - variant 0 is always the plain default, never patched or
    /// scattered) should use - see `VariantStyle`. Defaults to `Patch`
    /// for every variant, matching the original "blocks of leaves,
    /// blocks of moss" design. Override per-variant for anything that
    /// reads as a discrete point fixture rather than a spreadable ground
    /// cover - confirmed needed in real play (2026-09-08): Dungeon's
    /// "torchlight glow" patched as a whole region looked like a wall of
    /// torches, which no real dungeon would have. See
    /// docs/Map_Tile_Theme_Guide.md for the full row-9-12 breakdown of
    /// which cells need which style, theme by theme.
    fn floor_variant_style(&self, _variant: u8) -> VariantStyle {
        VariantStyle::Patch
    }
}

/// How a non-default floor variant gets placed on the map - see
/// `MapTheme::floor_variant_style` and `MapBuilder::assign_tile_variants`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantStyle {
    /// A contiguous, randomly-placed, randomly-sized region gets stamped
    /// with this variant - natural for anything that reads as a
    /// spreadable ground cover (a dirt patch, moss, a puddle, an algae
    /// bloom).
    Patch,
    /// Individual tiles scattered sparsely, one at a time, never as a
    /// contiguous block - natural for a discrete point fixture that
    /// would look absurd repeated in bulk right next to itself (a torch,
    /// a drain grate, a pile of bones, a pillar base).
    Scatter,
}

/// Every dungeon-crawl `MapTheme`, in one place - `MapBuilder::new` picks
/// one at random from exactly this list. Add a new theme here (plus its
/// own `MapTheme` impl in `themes.rs`) and it's automatically in the
/// random pool - no range/match bookkeeping needed anywhere else, unlike
/// the old `rng.range(0, 2)` + match this replaced (2026-09-08, when
/// Sewer became the third theme and hand-updating a hardcoded range for
/// every new one stopped being worth it).
fn dungeon_theme_pool() -> Vec<Box<dyn MapTheme>> {
    vec![DungeonTheme::new(), ForestTheme::new(), SewerTheme::new()]
}

/// Columns on `resources/map_tiles.png` - every theme's 4-row block uses
/// this many columns per row, matching the 16-cell (4x4) template in
/// docs/Map_Tile_Theme_Guide.md.
pub const MAP_TILE_COLS: u16 = 4;
/// How many distinct floor textures a migrated theme's FLOOR pool has -
/// row 0 (basic floor) and row 2 (themed floor) combined, MAP_TILE_COLS
/// each, picked randomly from either (see docs/Map_Tile_Theme_Guide.md's
/// confirmed row semantics).
pub const FLOOR_VARIANT_COUNT: u8 = MAP_TILE_COLS as u8 * 2;
/// How many distinct wall textures a migrated theme's WALL pool has -
/// row 1 (basic wall) only for now; row 3 (special wall) joins this pool
/// once its placement logic exists (deferred - see the guide doc).
pub const WALL_VARIANT_COUNT: u8 = MAP_TILE_COLS as u8;
/// Percent chance (0-99) an individual Wall tile swaps from the theme's
/// default wall texture (variant 0) to a random accent variant - see
/// MapBuilder::assign_tile_variants. Deliberately a MINORITY so the
/// default stays the dominant wall texture.
const WALL_ACCENT_CHANCE_PCT: i32 = 30;
/// How many floor texture patches get stamped per generated map - see
/// MapBuilder::assign_tile_variants.
const FLOOR_PATCH_COUNT_MIN: i32 = 6;
const FLOOR_PATCH_COUNT_MAX: i32 = 12;
/// Radius (tiles) of one floor texture patch - see
/// MapBuilder::assign_tile_variants.
const FLOOR_PATCH_RADIUS_MIN: i32 = 2;
const FLOOR_PATCH_RADIUS_MAX: i32 = 4;
/// Percent chance (0-99) an individual Floor tile swaps to a random
/// Scatter-style variant (see `MapTheme::floor_variant_style`) - much
/// lower than WALL_ACCENT_CHANCE_PCT on purpose: these are meant to be
/// rare, discrete highlights (a torch, a grate) across a whole floor
/// area, not a dense accent the way wall rubble/rock can be along a
/// boundary.
const FLOOR_SCATTER_CHANCE_PCT: i32 = 6;

const NUM_ROOMS: usize = 20;
pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub monster_spawns: Vec<Point>,
    pub player_start: Point,
    pub amulet_start: Point,
    pub theme: Box<dyn MapTheme>,
    /// Guaranteed enemy spawn points from a placed prefab's 'M'
    /// markers (see map_builder::prefab). Spawned directly via
    /// spawn_prefab_enemies rather than mixed into the general
    /// monster_spawns lottery, so a prefab's guards are always actually
    /// monsters - previously they were just added to monster_spawns,
    /// where the normal weighted pick could just as easily hand that
    /// point an item instead.
    pub prefab_enemy_spawns: Vec<Point>,
    /// Guaranteed weapon spawn point from a placed prefab's 'S' marker,
    /// if a prefab placed successfully this level (see
    /// map_builder::prefab - placement can fail, so this may be None).
    /// Shared by Swords and Staffs (see spawn_prefab_weapon) - a level
    /// gets one or the other here, not both. Neither family spawns in the
    /// general ambient pool at all - see Template::prefab_only.
    pub prefab_weapon_spawn: Option<Point>,
    /// Guaranteed loot-chest spawn point from the chest room's 'C'
    /// marker, if it placed successfully this level (see
    /// map_builder::prefab::apply_chest - placement can fail, same
    /// best-effort attempt loop as apply_prefab). Unlike the
    /// Fortress/Turret/Bunker roll above, the chest room is always
    /// attempted, not one-of-three-random - see MapBuilder::new.
    pub prefab_chest_spawn: Option<Point>,
    /// Guard spawn points from the chest room's 'M' markers (1 or 2,
    /// decided at placement time) - see
    /// spawner::spawn_prefab_chest_guards, which always picks this
    /// dungeon level's single toughest non-boss enemy for these, unlike
    /// prefab_enemy_spawns' weighted pool above.
    pub prefab_chest_guard_spawns: Vec<Point>,
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator) -> Self {
        let mut architect: Box<dyn MapArchitect> = match rng.range(0, 3) {
            0 => Box::new(DrunkardsWalkArchitect {}),
            1 => Box::new(RoomsArchitect {}),
            _ => Box::new(CellularAutomataArchitect {}),
        };
        let mut mb = architect.new(rng);
        apply_prefab(&mut mb, rng);
        apply_chest(&mut mb, rng);

        let mut pool = dungeon_theme_pool();
        let pick = rng.random_slice_index(&pool).unwrap();
        mb.theme = pool.swap_remove(pick);
        mb.assign_tile_variants(rng);

        mb
    }

    /// Assigns a texture variant to every Floor/Wall tile on the map,
    /// once, and stores it in `map.tile_variant` - see that field's own
    /// doc comment for why this is done once at generation time rather
    /// than picked fresh on every draw. A no-op if the active theme has
    /// no real tile art yet (`theme.tile_row()` is `None`) -
    /// `tile_variant` just stays at its default 0 for every tile, which
    /// is harmless since nothing reads it in that case anyway (see
    /// components::tile_render_at). Must be called AFTER every tile is
    /// in its final TileType for this map/theme - anything that changes
    /// a tile's TileType afterward (there isn't any today) would leave
    /// that tile with a stale variant rolled for its old type.
    ///
    /// Deliberately NOT a uniform per-tile random pick across the whole
    /// variant pool - confirmed too noisy in practice (floor and wall
    /// became hard to tell apart at a glance, since a per-tile roll has
    /// no larger-scale pattern for the eye to key off). Instead: every
    /// tile starts on its theme's plain default look (variant 0 - tile
    /// #1/#5 in the theme's own 16-cell template, see
    /// docs/Map_Tile_Theme_Guide.md, which is why those two specifically
    /// have to be the theme's plain/unremarkable texture), then two
    /// different rules layer real variety on top of that base:
    fn assign_tile_variants(&mut self, rng: &mut RandomNumberGenerator) {
        if self.theme.tile_row().is_none() {
            return;
        }
        for variant in self.map.tile_variant.iter_mut() {
            *variant = 0;
        }

        // Wall accents: a scattered MINORITY of individual wall tiles
        // swap to a different wall variant - deliberately per-tile, not
        // a patch, since a wall is usually only one tile thick and has
        // no width for a "block" to read differently from a scatter
        // anyway. Trees (or whatever variant 0 is) stay the dominant
        // wall texture; rock/rubble/thicket read as occasional accents.
        for idx in 0..self.map.tiles.len() {
            if self.map.tiles[idx] == TileType::Wall
                && rng.range(0, 100) < WALL_ACCENT_CHANCE_PCT
            {
                self.map.tile_variant[idx] = rng.range(1, WALL_VARIANT_COUNT);
            }
        }

        // Non-default floor variants split into two placement styles per
        // MapTheme::floor_variant_style - see VariantStyle's own doc
        // comment for why one style doesn't fit every kind of variant
        // (confirmed in real play, 2026-09-08: Dungeon's "torchlight
        // glow" patched as a whole region looked like a wall of torches).
        let patch_variants: Vec<u8> = (1..FLOOR_VARIANT_COUNT)
            .filter(|&v| self.theme.floor_variant_style(v) == VariantStyle::Patch)
            .collect();
        let scatter_variants: Vec<u8> = (1..FLOOR_VARIANT_COUNT)
            .filter(|&v| self.theme.floor_variant_style(v) == VariantStyle::Scatter)
            .collect();

        // Floor patches: a handful of contiguous, randomly-placed,
        // randomly-sized blobs, each stamped with a single Patch-style
        // floor variant - "blocks of leaves, blocks of moss" rather than
        // noise. The opposite shape from the wall accents above: floors
        // are big open areas where a whole natural-looking REGION of one
        // texture reads right, the way real terrain has patches of
        // different ground cover rather than pixel-scattered variety.
        let floor_tiles: Vec<usize> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, t)| **t == TileType::Floor)
            .map(|(i, _)| i)
            .collect();
        if floor_tiles.is_empty() {
            return;
        }
        if !patch_variants.is_empty() {
            let patch_count = rng.range(FLOOR_PATCH_COUNT_MIN, FLOOR_PATCH_COUNT_MAX + 1);
            for _ in 0..patch_count {
                let seed_idx = floor_tiles[rng.random_slice_index(&floor_tiles).unwrap()];
                let seed = self.map.index_to_point2d(seed_idx);
                let variant = patch_variants[rng.random_slice_index(&patch_variants).unwrap()];
                let radius = rng.range(FLOOR_PATCH_RADIUS_MIN, FLOOR_PATCH_RADIUS_MAX + 1);
                for y in (seed.y - radius)..=(seed.y + radius) {
                    for x in (seed.x - radius)..=(seed.x + radius) {
                        let pt = Point::new(x, y);
                        if !self.map.in_bounds(pt) {
                            continue;
                        }
                        let dx = (x - seed.x) as f32;
                        let dy = (y - seed.y) as f32;
                        if (dx * dx + dy * dy).sqrt() > radius as f32 {
                            continue;
                        }
                        let idx = map_idx(x, y);
                        if self.map.tiles[idx] == TileType::Floor {
                            self.map.tile_variant[idx] = variant;
                        }
                    }
                }
            }
        }

        // Floor scatter: individual, sparse tiles for Scatter-style
        // variants (a torch, a grate, a bone pile) - same shape as the
        // wall accents above, just floor-side and much rarer
        // (FLOOR_SCATTER_CHANCE_PCT), since these are discrete highlights
        // across a whole floor area rather than a dense boundary accent.
        // Runs after patches, so a scattered accent can still land on top
        // of a patch (a torch on a mossy patch of floor is perfectly
        // plausible).
        if !scatter_variants.is_empty() {
            for &idx in &floor_tiles {
                if rng.range(0, 100) < FLOOR_SCATTER_CHANCE_PCT {
                    self.map.tile_variant[idx] =
                        scatter_variants[rng.random_slice_index(&scatter_variants).unwrap()];
                }
            }
        }
    }

    fn fill(&mut self, tile: TileType) {
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    /// Builds the Battle Arena's shop map: a small hand-built walled room
    /// (not one of the three random MapArchitect layouts - a shop is a
    /// fixed, deliberate layout, not something worth randomizing) with an
    /// items row along the top, the player starting in the middle, and a
    /// stairs point (returned via the usual `amulet_start` field, same
    /// convention every other level already uses for "the special point
    /// that becomes a TileType::Exit tile") along the bottom.
    ///
    /// Returns the MapBuilder, exactly `item_count` floor points for
    /// items (left to right, centered on the counter), one point for the
    /// shopkeeper NPC, and a reveal rectangle (x, y, width, height) -
    /// State::start_arena only marks tiles inside that rectangle as
    /// revealed/visible, leaving the rest of the underlying 80x50 grid
    /// (a fixed size every Map uses, not something one MapBuilder can
    /// resize on its own) completely unrevealed. Since nothing renders
    /// for a tile that's neither visible nor ever revealed, this is what
    /// actually makes the map itself read as small - a camera trick
    /// alone couldn't do this, because the camera has no way to tell
    /// "the edge of this small room" apart from "more world offscreen."
    ///
    /// The shop's own interior is a fixed 10 (wide) x 6 (tall) - not
    /// scaled by item count - split top to bottom into: row 0 the
    /// shopkeeper (decorative, one tile, centered) - genuinely behind
    /// the counter, safe to see because start_arena's reveal covers this
    /// whole area rather than relying on real shadowcasting, which a
    /// Counter tile would otherwise block sight past just like a Wall
    /// does. Row 1 the counter itself (TileType::Counter - impassable,
    /// items sit on it, centered within the 10-wide row). Rows 2-5, four
    /// rows of open walkable floor - the player starts at the top of
    /// this (row 2, directly below the row-1 item at the same column) and
    /// the stairs sit at the bottom (row 5).
    pub fn new_arena_shop(
        rng: &mut RandomNumberGenerator,
        item_count: usize,
    ) -> (Self, Vec<Point>, Point, i32, i32, i32, i32) {
        const INTERIOR_W: i32 = 10;
        const INTERIOR_H: i32 = 6;
        const ROOM_W: i32 = INTERIOR_W + 2;
        const ROOM_H: i32 = INTERIOR_H + 2;

        // The reveal rectangle - "the overall map" as the player will
        // actually experience it, centered on the same point as the
        // room itself. ~40x20 comfortably frames the small 12x8 room
        // with a visible ring of backdrop, without being anywhere close
        // to the full 80x50 grid every dungeon floor uses.
        const REVEAL_W: i32 = 40;
        const REVEAL_H: i32 = 20;

        let room_x0 = (SCREEN_WIDTH - ROOM_W) / 2;
        let room_y0 = (SCREEN_HEIGHT - ROOM_H) / 2;
        let reveal_x0 = (SCREEN_WIDTH - REVEAL_W) / 2;
        let reveal_y0 = (SCREEN_HEIGHT - REVEAL_H) / 2;

        let mut mb = Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            // Forest, not dungeon brick, for the shop's backdrop -
            // walking bark/mossy fill reads as deliberate scenery
            // framing a small clearing, since the reveal boundary above
            // means the player will actually see the edge of it rather
            // than endless brick.
            theme: ForestTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        };
        mb.fill(TileType::Wall);
        for y in (room_y0 + 1)..(room_y0 + ROOM_H - 1) {
            for x in (room_x0 + 1)..(room_x0 + ROOM_W - 1) {
                let idx = map_idx(x, y);
                mb.map.tiles[idx] = TileType::Floor;
            }
        }

        let interior_x = |col: i32| room_x0 + 1 + col;
        let interior_y = |row: i32| room_y0 + 1 + row;

        // The counter row - TileType::Counter, not Floor: impassable
        // (can_enter_tile only allows Floor/Exit) so the player can
        // never step onto it, and rendered as a distinct warm bar (see
        // components::tile_render_at) rather than plain floor or wall
        // brick.
        for col in 0..INTERIOR_W {
            let idx = map_idx(interior_x(col), interior_y(1));
            mb.map.tiles[idx] = TileType::Counter;
        }

        let shopkeeper_point = Point::new(interior_x(INTERIOR_W / 2), interior_y(0));
        mb.player_start = Point::new(interior_x(INTERIOR_W / 2), interior_y(2));
        mb.amulet_start = Point::new(interior_x(INTERIOR_W / 2), interior_y(5));

        // Items are centered within the fixed 10-wide counter rather
        // than always starting at column 0 - a shop selling only 3-4
        // things sits in the middle of the counter instead of bunched
        // against the left wall. item_count is expected to be at most
        // 7 (1 weapon + 1 potion stack + up to 5 ability stacks), which
        // always fits inside 10 with room to spare.
        let items_start_col = ((INTERIOR_W - item_count as i32) / 2).max(0);
        let item_points: Vec<Point> = (0..item_count as i32)
            .map(|i| Point::new(interior_x(items_start_col + i), interior_y(1)))
            .collect();

        mb.assign_tile_variants(rng);

        (
            mb,
            item_points,
            shopkeeper_point,
            reveal_x0,
            reveal_y0,
            REVEAL_W,
            REVEAL_H,
        )
    }

    /// Builds the Battle Arena's combat map for one wave: a single open
    /// circular clearing (no interior walls to block sight or movement -
    /// the player is meant to see and reach every enemy) with the player
    /// starting at the center and `enemy_count` spawn points scattered
    /// around a thin ring near the circle's edge, so enemies visibly
    /// appear at the edge of the clearing rather than already being on
    /// top of the player. Also returns a boss spawn point (the reachable
    /// tile farthest from the player's start, the same
    /// "find_most_distant" convention every dungeon floor already uses
    /// for its own boss/exit point) - this is only actually used once
    /// wave 3 clears (see State::arena_spawn_boss_on_current_map), since
    /// the boss appears on this same map rather than a freshly built one.
    ///
    /// Returns the MapBuilder, up to `enemy_count` spawn points (fewer
    /// if the edge ring somehow has less room than that - it doesn't, at
    /// this circle's size, for any of the 5/5/3 wave counts), the boss
    /// spawn point, and a reveal rectangle (x, y, width, height) - see
    /// new_arena_shop's doc comment for why a reveal rectangle rather
    /// than the whole 80x50 map is what actually keeps this map's
    /// footprint small on screen.
    pub fn new_arena_wave(
        rng: &mut RandomNumberGenerator,
        enemy_count: usize,
    ) -> (Self, Vec<Point>, Point, i32, i32, i32, i32) {
        const RADIUS: i32 = 10;
        const REVEAL_W: i32 = 34;
        const REVEAL_H: i32 = 26;

        let center = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
        let reveal_x0 = (SCREEN_WIDTH - REVEAL_W) / 2;
        let reveal_y0 = (SCREEN_HEIGHT - REVEAL_H) / 2;

        let mut mb = Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: ForestTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        };
        mb.fill(TileType::Wall);
        // A circular clearing, not a rectangular room - there's no
        // separate wall ring to draw here: the surrounding Wall/forest
        // fill (already everywhere from the fill above) forms the
        // boundary on its own, wherever a tile falls outside RADIUS of
        // center.
        for y in (center.y - RADIUS - 1)..=(center.y + RADIUS + 1) {
            for x in (center.x - RADIUS - 1)..=(center.x + RADIUS + 1) {
                let dx = (x - center.x) as f32;
                let dy = (y - center.y) as f32;
                if (dx * dx + dy * dy).sqrt() <= RADIUS as f32 {
                    mb.map.tiles[map_idx(x, y)] = TileType::Floor;
                }
            }
        }

        mb.player_start = center;

        // A thin ring of floor tiles near the circle's own edge - the
        // "perimeter" enemies spawn on, so they visibly appear at the
        // edge of the clearing rather than already being on top of the
        // player. Same distance check as the fill above, just looking
        // for tiles close to RADIUS rather than anywhere inside it.
        let mut perimeter: Vec<Point> = Vec::new();
        for y in (center.y - RADIUS)..=(center.y + RADIUS) {
            for x in (center.x - RADIUS)..=(center.x + RADIUS) {
                let dx = (x - center.x) as f32;
                let dy = (y - center.y) as f32;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist <= RADIUS as f32 && dist >= RADIUS as f32 - 1.5 {
                    perimeter.push(Point::new(x, y));
                }
            }
        }

        // Distinct random points, same pick-and-remove pattern
        // spawn_monsters (below) already uses - capped at however many
        // perimeter tiles actually exist, though at this circle's size
        // (~60 edge tiles) that cap never actually bites for any of the
        // 5/5/3 wave counts.
        let mut candidates = perimeter;
        let spawn_count = enemy_count.min(candidates.len());
        let mut enemy_spawns = Vec::with_capacity(spawn_count);
        for _ in 0..spawn_count {
            let idx = rng.random_slice_index(&candidates).unwrap();
            enemy_spawns.push(candidates[idx]);
            candidates.remove(idx);
        }

        let boss_spawn = mb.find_most_distant();
        mb.assign_tile_variants(rng);

        (
            mb,
            enemy_spawns,
            boss_spawn,
            reveal_x0,
            reveal_y0,
            REVEAL_W,
            REVEAL_H,
        )
    }

    fn find_most_distant(&self) -> Point {
        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );

        const UNREACHABLE: &f32 = &f32::MAX;
        self.map.index_to_point2d(
            dijkstra_map
                .map
                .iter()
                .enumerate()
                .filter(|(_, dist)| *dist < UNREACHABLE)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }

    fn build_random_rooms(&mut self, rng: &mut RandomNumberGenerator) {
        while self.rooms.len() < NUM_ROOMS {
            let room = Rect::with_size(
                rng.range(1, SCREEN_WIDTH - 10),
                rng.range(1, SCREEN_HEIGHT - 10),
                rng.range(2, 10),
                rng.range(2, 10),
            );
            let mut overlap = false;
            for r in self.rooms.iter() {
                if r.intersect(&room) {
                    overlap = true;
                }
            }
            if !overlap {
                room.for_each(|p| {
                    if p.x > 0 && p.x < SCREEN_WIDTH && p.y > 0 && p.y < SCREEN_HEIGHT {
                        let idx = map_idx(p.x, p.y);
                        self.map.tiles[idx] = TileType::Floor;
                    }
                });

                self.rooms.push(room)
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        use std::cmp::{max, min};
        for x in min(x1, x2)..=max(x1, x2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        use std::cmp::{max, min};
        for y in min(y1, y2)..=max(y1, y2) {
            if let Some(idx) = self.map.try_idx(Point::new(x, y)) {
                self.map.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn build_corridors(&mut self, rng: &mut RandomNumberGenerator) {
        let mut rooms = self.rooms.clone();
        rooms.sort_by(|a, b| a.center().x.cmp(&b.center().x));

        for (i, room) in rooms.iter().enumerate().skip(1) {
            let prev = rooms[i - 1].center();
            let new = room.center();

            if rng.range(0, 2) == 1 {
                self.apply_horizontal_tunnel(prev.x, new.x, prev.y);
                self.apply_vertical_tunnel(prev.y, new.y, new.x);
            } else {
                self.apply_vertical_tunnel(prev.y, new.y, prev.x);
                self.apply_horizontal_tunnel(prev.x, new.x, new.y);
            }
        }
    }

    fn spawn_monsters(&self, start: &Point, rng: &mut RandomNumberGenerator) -> Vec<Point> {
        const NUM_MONSTERS: usize = 50;
        let mut spawnable_tiles: Vec<Point> = self
            .map
            .tiles
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                **t == TileType::Floor
                    && DistanceAlg::Pythagoras.distance2d(*start, self.map.index_to_point2d(*idx))
                        > 10.0
            })
            .map(|(idx, _)| self.map.index_to_point2d(idx))
            .collect();

        let mut spawns = Vec::new();
        // Capped at however many candidate tiles actually exist, rather
        // than always looping a fixed NUM_MONSTERS times. A small or
        // tightly-packed cave can easily have fewer than 50 tiles more
        // than 10 units from the player's start - previously this kept
        // looping anyway, and random_slice_index on an empty slice
        // returns None, which the unconditional .unwrap() below turned
        // into an outright panic once spawnable_tiles ran out. Spawning
        // fewer monsters on a small map is the correct behavior, not a
        // bug to route around - CellularAutomataArchitect's connectivity
        // fix (see automata.rs's cull_unreachable_areas) made this
        // reachable a lot more often, by shrinking the reachable floor
        // area on some maps, but the underlying fragility was already
        // here regardless of map size.
        let spawn_count = NUM_MONSTERS.min(spawnable_tiles.len());
        for _ in 0..spawn_count {
            let target_index = rng.random_slice_index(&spawnable_tiles).unwrap();
            spawns.push(spawnable_tiles[target_index].clone());
            spawnable_tiles.remove(target_index);
        }
        spawns
    }
}
