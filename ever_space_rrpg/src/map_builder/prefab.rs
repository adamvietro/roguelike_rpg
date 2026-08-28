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

pub fn apply_prefab(mb: &mut MapBuilder, rng: &mut RandomNumberGenerator) {
    let mut placement = None;

    let dijkstra_map = DijkstraMap::new(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        &vec![mb.map.point2d_to_index(mb.player_start)],
        &mb.map,
        1024.0,
    );

    let template = match rng.range(0, 3) {
        0 => FORTRESS,
        1 => TURRET,
        2 => BUNKER,
        _ => unreachable!(),
    };

    let mut attempts = 0; // (1)
    while placement.is_none() && attempts < 10 {
        // (2)
        let dimensions = Rect::with_size(
            // (3)
            rng.range(0, SCREEN_WIDTH - template.1),
            rng.range(0, SCREEN_HEIGHT - template.2),
            template.1,
            template.2,
        );

        let mut can_place = false; // (4)
        dimensions.for_each(|pt| {
            // (5)
            let idx = mb.map.point2d_to_index(pt);
            let distance = dijkstra_map.map[idx];
            if distance < 2000.0 && distance > 20.0 && mb.amulet_start != pt {
                // (6)
                can_place = true;
            }
        });

        if can_place {
            // (7)
            placement = Some(Point::new(dimensions.x1, dimensions.y1));
            let points = dimensions.point_set();
            mb.monster_spawns.retain(|pt| !points.contains(pt)); // (8)
        }
        attempts += 1;
    }

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
                    '#' => mb.map.tiles[idx] = TileType::Wall,
                    _ => println!("No idea what to do with [{}]", c), // (17)
                }
                i += 1;
            }
        }
    }
}
