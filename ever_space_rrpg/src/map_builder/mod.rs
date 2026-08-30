use crate::prelude::*;
mod empty;
mod rooms;
use rooms::RoomsArchitect;
mod automata;
use automata::CellularAutomataArchitect;
mod drunkard;
use drunkard::DrunkardsWalkArchitect;
mod prefab;
use prefab::apply_prefab;
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
}

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

        mb.theme = match rng.range(0, 2) {
            0 => DungeonTheme::new(),
            _ => ForestTheme::new(),
        };

        mb
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
    /// Returns the MapBuilder, plus exactly 11 floor points for items (in
    /// a fixed left-to-right order: index 0 is the weapon slot, 1..=5 are
    /// the 5 potion slots, 6..=10 are the 5 ability slots), plus one
    /// point for the shopkeeper NPC. The caller (State::start_arena /
    /// State::enter_arena_shop) decides which named item template goes
    /// at each item point and what to render at the shopkeeper point -
    /// this function only lays out the room.
    ///
    /// Interior rows, top to bottom: 0 the shopkeeper (decorative, one
    /// tile, centered) - genuinely behind the counter now, safe to see
    /// because State::start_arena marks this whole map fully visible
    /// (no fog of war) rather than relying on real shadowcasting, which
    /// a Counter tile would otherwise block sight past just like a
    /// Wall does. 1 the counter itself (TileType::Counter - impassable,
    /// items sit on it). 2 the player's start position, directly below
    /// the row-1 item at the same column (so the player starts already
    /// able to buy that one item without moving first) - the only row
    /// items can ever be bought from, since row 1 is impassable. 3 a
    /// walkable gap. 4 the stairs.
    pub fn new_arena_shop(_rng: &mut RandomNumberGenerator) -> (Self, Vec<Point>, Point) {
        // Interior dimensions (inside the walls). 13 columns gives 11
        // item columns (interior cols 1..=11) plus a 1-tile buffer on
        // each side; 5 rows fits the keeper/counter/player-start/gap/
        // stairs layout described above.
        const INTERIOR_W: i32 = 13;
        const INTERIOR_H: i32 = 5;
        const ROOM_W: i32 = INTERIOR_W + 2;
        const ROOM_H: i32 = INTERIOR_H + 2;

        let x0 = (SCREEN_WIDTH - ROOM_W) / 2;
        let y0 = (SCREEN_HEIGHT - ROOM_H) / 2;

        let mut mb = Self {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: DungeonTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
        };
        mb.fill(TileType::Wall);
        for y in (y0 + 1)..(y0 + ROOM_H - 1) {
            for x in (x0 + 1)..(x0 + ROOM_W - 1) {
                let idx = map_idx(x, y);
                mb.map.tiles[idx] = TileType::Floor;
            }
        }

        let interior_x = |col: i32| x0 + 1 + col;
        let interior_y = |row: i32| y0 + 1 + row;

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
        mb.amulet_start = Point::new(interior_x(INTERIOR_W / 2), interior_y(4));

        let item_points: Vec<Point> = (0..11)
            .map(|i| Point::new(interior_x(1 + i), interior_y(1)))
            .collect();

        (mb, item_points, shopkeeper_point)
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
