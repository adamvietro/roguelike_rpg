use crate::prelude::*;
mod starting_kits;
mod template;
use starting_kits::StartingKits;
use template::Templates;

/// A class's starting Health/Damage/Speed/Defense/Evasion, plus which
/// glyph the player renders as in the dungeon - matches the same letter
/// used for that class's icon on the class-select screen (see
/// CLASS_ROSTER in screens/title.rs), so the choice you made stays visually
/// consistent in-game. Looked up by class_base_stats at spawn time -
/// previously every class got identical hardcoded numbers/glyph
/// regardless of which was picked.
struct ClassBaseStats {
    health: i32,
    damage: i32,
    speed: i32,
    defense: i32,
    /// Base percent chance to fully evade an incoming attack - see
    /// components::Evasion. 0 for every class except Rogue so far.
    evasion: i32,
    glyph: char,
}

/// Base stats + glyph for a given class name. Amazon/Archer don't have
/// designed stats yet (still placeholders - see CLASS_ROSTER in screens/title.rs),
/// so they share the fallback numbers for now - Barbarian, Mage, and
/// Rogue all have real, distinct numbers. Each class still gets its own
/// glyph even while sharing stats.
fn class_base_stats(class: &str) -> ClassBaseStats {
    match class {
        "Barbarian" => ClassBaseStats {
            health: 15,
            damage: 1,
            speed: 6,
            defense: 0,
            evasion: 0,
            glyph: '@',
        },
        "Mage" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: -1,
            evasion: 0,
            glyph: 'm',
        },
        "Rogue" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 10,
            defense: 0,
            evasion: 10,
            glyph: 'r',
        },
        "Amazon" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            evasion: 0,
            glyph: 'a',
        },
        "Archer" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            evasion: 0,
            glyph: 'B',
        },
        // Not in CLASS_ROSTER - never appears on the normal class-select
        // screen, only reachable via the hidden D-key shortcut there
        // (see screens/title.rs's class_select). Deliberately overpowered stats
        // plus the three instant-win/lose/next-level items (see
        // resources/starting_kits.ron) for quickly testing late-game
        // states without a real playthrough.
        "Debug" => ClassBaseStats {
            health: 100,
            damage: 5,
            speed: 10,
            defense: 0,
            evasion: 50,
            glyph: 'D',
        },
        // Safe fallback for any unrecognized class name.
        _ => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            evasion: 0,
            glyph: '@',
        },
    }
}

pub fn spawn_player(ecs: &mut World, pos: Point, class: &str) -> Entity {
    let stats = class_base_stats(class);
    let mut commands = legion::systems::CommandBuffer::new(ecs);
    let player = commands.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437(stats.glyph),
        },
        Health {
            current: stats.health,
            max: stats.health,
        },
        FieldOfView::new(8),
        Damage(stats.damage),
    ));
    commands.add_component(player, CanAttack);
    commands.add_component(player, CanDefend);
    commands.add_component(player, CanFlee);
    commands.add_component(player, Speed(stats.speed));
    commands.add_component(player, Class(class.to_string()));
    commands.add_component(player, Defense(stats.defense));
    commands.add_component(player, Evasion(stats.evasion));
    commands.flush(ecs);
    player
}

pub fn spawn_level(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_points: &[Point],
) {
    let template = Templates::load();
    template.spawn_entities(ecs, rng, level, spawn_points);
}

/// Spawns guaranteed enemies at a prefab's guard positions - see
/// Templates::spawn_prefab_enemies for why this is separate from
/// spawn_level's general pool.
pub fn spawn_prefab_enemies(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_points: &[Point],
) {
    let template = Templates::load();
    template.spawn_prefab_enemies(ecs, rng, level, spawn_points);
}

/// Spawns a guaranteed weapon at a prefab's treasure position, if one
/// placed this level - see Templates::spawn_prefab_weapon. Filtered to
/// `player_class` (or unrestricted), so a Mage never finds a Sword here.
pub fn spawn_prefab_weapon(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_point: Option<Point>,
    player_class: &str,
) {
    let template = Templates::load();
    template.spawn_prefab_weapon(ecs, rng, level, spawn_point, player_class);
}

/// Rolls a chance to grant the player a random one-time battle item after
/// a battle victory, filtered to items matching the player's own Class.
/// Returns the granted item's display name, if any.
pub fn grant_random_battle_loot(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    player: Entity,
) -> Option<String> {
    let player_class = entity_class(ecs, player)?;
    let template = Templates::load();
    template.grant_random_battle_loot(ecs, rng, player, &player_class)
}

/// Grants `player` their class's guaranteed starting inventory - see
/// resources/starting_kits.ron. Call once, right after spawn_player, when
/// a new run begins. Classes without a defined kit there simply start
/// with nothing extra, same as every class did before this system existed.
pub fn grant_starting_items(ecs: &mut World, player: Entity, class: &str) {
    let kits = StartingKits::load();
    let items = kits.items_for(class);
    if items.is_empty() {
        return;
    }
    let template = Templates::load();
    for item_name in items {
        template.spawn_named_item(ecs, player, item_name);
    }
}

pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item,
        AmuletOfYala,
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('|'),
        },
        Name("Amulet of Yala".to_string()),
    ));
}

/// Every distinct technique name defined for `class` in template.ron,
/// regardless of ownership - see Templates::technique_names_for_class.
/// Used by battle::available_actions to show a class's full technique
/// roster with unowned ones greyed out.
pub fn class_technique_names(class: &str) -> Vec<String> {
    let template = Templates::load();
    template.technique_names_for_class(class)
}
