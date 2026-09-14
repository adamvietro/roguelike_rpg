use crate::prelude::*;

const FORTRESS: (&str, i32, i32) = (
    "
------------
---######---
---#----#---
---#-M--#---
-###----###-
--M--S---M--
-###----###-
---#----#---
---#--M-#---
---######---
------------
",
    12,
    11,
);

const TURRET: (&str, i32, i32) = (
    "
--------
--##-##-
--#-M-#-
----S---
--#-M-#-
--##-##-
--------
    ",
    8,
    7,
);

const BUNKER: (&str, i32, i32) = (
    "
----------
----##----
---#M-#---
--#----#--
-#------#-
--M-S--M--
-#------#-
--#----#--
---#--#---
----##----
    ",
    10,
    10,
);

// The door sits on the GUARDS' row, not the chest's - the chest's own
// row stays fully walled in on the entrance side, so the only path in is
// past the guards first, not a straight line to the chest that happens
// to have a guard standing nearby. See apply_chest's own doc comment.
const CHEST_ROOM: (&str, i32, i32) = (
    "
--------
--####--
--#MM---
--#-C#--
--####--
--------
",
    8,
    6,
);

/// One random-placement attempt loop, shared by apply_prefab and
/// apply_chest below: up to 10 tries at a `width`x`height` rectangle
/// that's reachable from the player's start (a real Dijkstra distance,
/// not just "somewhere on the map"), far enough away to not trivially
/// stumble onto (distance > 20.0), not so far it's on an unreachable
/// island (distance < 2000.0), and never overlapping the amulet/exit
/// point. Returns the rectangle's top-left corner, and - as a side
/// effect on success - clears any rolled `monster_spawns` that would
/// have landed inside it, so a prefab room's own hand-placed 'M'/'S'/'C'
/// markers below aren't double-booked with the general ambient spawn
/// pool.
fn find_prefab_placement(
    mb: &mut MapBuilder,
    rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> Option<Point> {
    let dijkstra_map = DijkstraMap::new(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        &vec![mb.map.point2d_to_index(mb.player_start)],
        &mb.map,
        1024.0,
    );

    let mut attempts = 0;
    while attempts < 10 {
        let dimensions = Rect::with_size(
            rng.range(0, SCREEN_WIDTH - width),
            rng.range(0, SCREEN_HEIGHT - height),
            width,
            height,
        );

        let mut can_place = false;
        dimensions.for_each(|pt| {
            let idx = mb.map.point2d_to_index(pt);
            let distance = dijkstra_map.map[idx];
            if distance < 2000.0 && distance > 20.0 && mb.amulet_start != pt {
                can_place = true;
            }
        });

        if can_place {
            let points = dimensions.point_set();
            mb.monster_spawns.retain(|pt| !points.contains(pt));
            return Some(Point::new(dimensions.x1, dimensions.y1));
        }
        attempts += 1;
    }
    None
}

pub fn apply_prefab(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
    let pick = rng.range(0, 3);
    let template = match pick {
        0 => FORTRESS,
        1 => TURRET,
        2 => BUNKER,
        _ => unreachable!(),
    };
    // Only the FORTRESS template's own wall ring becomes a moat - see
    // MapTheme::fortress_moat_variant's own doc comment. `None` for
    // Turret/Bunker, or for a theme with no fortress moat configured
    // (Dungeon, deliberately - 2026-09-13), keeps every '#' a plain Wall
    // exactly as before.
    let moat_variant = if pick == 0 {
        mb.theme.fortress_moat_variant()
    } else {
        None
    };

    let placement = find_prefab_placement(mb, rng, template.1, template.2);

    if let Some(placement) = placement {
        // (9)
        let string_vec: Vec<char> = template
            .0
            .chars()
            .filter(|a| *a != '\r' && *a != '\n')
            .collect(); // (10)
        let mut i = 0; // (11)
        for ty in placement.y..placement.y + template.2 {
            // (12)
            for tx in placement.x..placement.x + template.1 {
                let idx = map_idx(tx, ty);
                let c = string_vec[i]; // (13)
                match c {
                    // (14)
                    'M' => {
                        // (15)
                        // Pushed to a dedicated list, NOT mb.monster_spawns -
                        // that general list feeds a weighted lottery that
                        // picks any template (enemy OR item) for a given
                        // point, so a guard position had no actual
                        // guarantee of getting a monster. spawn_prefab_enemies
                        // (see spawner/template.rs) spawns these directly
                        // instead.
                        mb.map.tiles[idx] = TileType::Floor;
                        mb.prefab_enemy_spawns.push(Point::new(tx, ty));
                    }
                    'S' => {
                        // Guaranteed weapon spawn (Sword or Staff) - see
                        // Template::prefab_only / spawn_prefab_weapon.
                        mb.map.tiles[idx] = TileType::Floor;
                        mb.prefab_weapon_spawn = Some(Point::new(tx, ty));
                    }
                    '-' => mb.map.tiles[idx] = TileType::Floor, // (16)
                    '#' => match moat_variant {
                        Some(variant) => {
                            mb.map.tiles[idx] = TileType::Water;
                            mb.map.tile_variant[idx] = variant;
                        }
                        None => mb.map.tiles[idx] = TileType::Wall,
                    },
                    _ => panic!("FORTRESS/TURRET/BUNKER template has an unrecognized marker [{}]", c),
                }
                i += 1;
            }
        }
    }
}

/// Always-attempted (not a one-of-three random pick like apply_prefab
/// above) placement of a single guaranteed loot chest, guarded by 1-2 of
/// this dungeon level's toughest ordinary enemy - see
/// spawner::spawn_prefab_chest_guards. Uses the same
/// find_prefab_placement as apply_prefab, just against a single fixed
/// room instead of a random pick of three. "Always place a chest" only
/// means "always attempt it, don't roll whether to" - placement can still
/// rarely fail to find room on a very cramped map, the same best-effort
/// guarantee apply_prefab's own weapon/guard markers already have.
pub fn apply_chest(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
    let template = CHEST_ROOM;
    // See MapTheme::chest_moat_variant's own doc comment - `None` for a
    // theme with no chest moat configured keeps every '#' a plain Wall
    // exactly as before.
    let moat_variant = mb.theme.chest_moat_variant();

    let placement = find_prefab_placement(mb, rng, template.1, template.2);

    if let Some(placement) = placement {
        let string_vec: Vec<char> = template
            .0
            .chars()
            .filter(|a| *a != '\r' && *a != '\n')
            .collect();
        let mut guard_slots: Vec<Point> = Vec::new();
        let mut i = 0;
        for ty in placement.y..placement.y + template.2 {
            for tx in placement.x..placement.x + template.1 {
                let idx = map_idx(tx, ty);
                let c = string_vec[i];
                match c {
                    'M' => {
                        mb.map.tiles[idx] = TileType::Floor;
                        guard_slots.push(Point::new(tx, ty));
                    }
                    'C' => {
                        mb.map.tiles[idx] = TileType::Floor;
                        mb.prefab_chest_spawn = Some(Point::new(tx, ty));
                    }
                    '-' => mb.map.tiles[idx] = TileType::Floor,
                    '#' => match moat_variant {
                        Some(variant) => {
                            mb.map.tiles[idx] = TileType::Water;
                            mb.map.tile_variant[idx] = variant;
                        }
                        None => mb.map.tiles[idx] = TileType::Wall,
                    },
                    _ => panic!("CHEST_ROOM template has an unrecognized marker [{}]", c),
                }
                i += 1;
            }
        }

        // 1 or 2 guards, picked from the room's two 'M' slots - the
        // guard TYPE is deliberately NOT randomized (see
        // spawn_prefab_chest_guards), just how many of the two slots
        // actually get filled.
        let guard_count = rng.range(1, 3) as usize;
        if guard_count >= guard_slots.len() {
            mb.prefab_chest_guard_spawns = guard_slots;
        } else {
            let idx = rng.random_slice_index(&guard_slots).unwrap();
            mb.prefab_chest_guard_spawns = vec![guard_slots[idx]];
        }
    }
}
