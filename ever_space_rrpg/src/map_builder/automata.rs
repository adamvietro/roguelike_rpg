use super::MapArchitect;
use crate::prelude::*;

pub struct CellularAutomataArchitect {}

impl MapArchitect for CellularAutomataArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
            prefab_enemy_spawns: Vec::new(),
            prefab_weapon_spawn: None,
            prefab_chest_spawn: None,
            prefab_chest_guard_spawns: Vec::new(),
        };
        self.random_noise_map(rng, &mut mb.map);
        for _ in 0..10 {
            self.iteration(&mut mb.map);
        }
        let start = self.find_start(&mb.map);
        // Cellular automata smoothing has no built-in guarantee the
        // result is a single connected cave - unlike RoomsArchitect
        // (every room chained together via build_corridors) or
        // DrunkardsWalkArchitect (which already re-checks reachability
        // after every carve, see its own DijkstraMap culling step this
        // mirrors), nothing here ever confirmed every Floor tile could
        // actually be reached. On an unlucky roll, the smoothing could
        // produce two or more separate caves - the player would still
        // reach the stairs fine (find_most_distant below already only
        // measures reachable distance via its own DijkstraMap), but a
        // second, disconnected cave elsewhere on the map - fully
        // wall-locked, no corridor to it - could still exist, be
        // walkable-looking, and get real monster/item spawns place into
        // it that the player could never actually reach.
        //
        // Walling off anything NOT reachable from `start` closes that
        // gap: every remaining Floor tile is now guaranteed connected to
        // player_start by the time monster spawning or amulet placement
        // ever runs.
        self.cull_unreachable_areas(&mut mb.map, &start);
        mb.monster_spawns = mb.spawn_monsters(&start, rng);
        mb.player_start = start;
        mb.amulet_start = mb.find_most_distant();
        mb
    }
}

impl CellularAutomataArchitect {
    fn random_noise_map(&mut self, rng: &mut RandomNumberGenerator, map: &mut Map) {
        map.tiles.iter_mut().for_each(|t| {
            // (1)
            let roll = rng.range(0, 100); // (2)
            if roll > 55 {
                // (3)
                *t = TileType::Floor; // (4)
            } else {
                *t = TileType::Wall;
            }
        });
    }

    fn count_neighbors(&self, x: i32, y: i32, map: &Map) -> usize {
        let mut neighbors = 0;
        for iy in -1..=1 {
            for ix in -1..=1 {
                if !(ix == 0 && iy == 0) && map.tiles[map_idx(x + ix, y + iy)] == TileType::Wall {
                    neighbors += 1;
                }
            }
        }

        neighbors
    }

    fn iteration(&mut self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone(); // (5)
        for y in 1..SCREEN_HEIGHT - 1 {
            // (6)
            for x in 1..SCREEN_WIDTH - 1 {
                let neighbors = self.count_neighbors(x, y, map); // (7)
                let idx = map_idx(x, y);
                if neighbors > 4 || neighbors == 0 {
                    // (8)
                    new_tiles[idx] = TileType::Wall;
                } else {
                    new_tiles[idx] = TileType::Floor;
                }
            }
        }
        map.tiles = new_tiles;
    }

    fn find_start(&self, map: &Map) -> Point {
        let center = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2); // (9)
        let closest_point = map
            .tiles
            .iter() // (10)
            .enumerate() // (11)
            .filter(|(_, t)| **t == TileType::Floor) // (12)
            .map(|(idx, _)| {
                (
                    idx,
                    DistanceAlg::Pythagoras.distance2d(
                        // (13)
                        center,
                        map.index_to_point2d(idx),
                    ),
                )
            })
            .min_by(|(_, distance), (_, distance2)| distance.partial_cmp(&distance2).unwrap()) // (14)
            .map(|(idx, _)| idx) // (15)
            .unwrap(); // (16)
        map.index_to_point2d(closest_point) // (17)
    }

    /// Walls off every Floor tile NOT reachable from `start`, using the
    /// same DijkstraMap-based approach DrunkardsWalkArchitect already
    /// uses for the same reason - see its own drunkard-loop culling
    /// step. Cellular automata smoothing can produce more than one
    /// disconnected cave; this guarantees exactly one remains by the
    /// time the caller starts placing spawns/the amulet in it.
    fn cull_unreachable_areas(&mut self, map: &mut Map, start: &Point) {
        let dijkstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![map.point2d_to_index(*start)],
            map,
            1024.0,
        );
        const UNREACHABLE: f32 = f32::MAX;
        for (idx, distance) in dijkstra_map.map.iter().enumerate() {
            if map.tiles[idx] == TileType::Floor && *distance >= UNREACHABLE {
                map.tiles[idx] = TileType::Wall;
            }
        }
    }
}
