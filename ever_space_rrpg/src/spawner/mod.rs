use crate::prelude::*;
mod starting_kits;
mod template;
use starting_kits::StartingKits;
use template::Templates;

/// A class's starting Health/Damage/Speed/Defense, plus which glyph the
/// player renders as in the dungeon - matches the same letter used for
/// that class's icon on the class-select screen (see CLASS_ROSTER in
/// main.rs), so the choice you made stays visually consistent in-game.
/// Looked up by class_base_stats at spawn time - previously every class
/// got identical hardcoded numbers/glyph regardless of which was picked.
struct ClassBaseStats {
    health: i32,
    damage: i32,
    speed: i32,
    defense: i32,
    glyph: char,
}

/// Base stats + glyph for a given class name. Rogue/Amazon/Archer don't
/// have designed stats yet (still placeholders - see CLASS_ROSTER in
/// main.rs), so they share Barbarian's numbers for now - only Mage has
/// real numbers so far (Health 10, Defense -1 - a squishier caster). Each
/// class still gets its own glyph even while sharing stats.
fn class_base_stats(class: &str) -> ClassBaseStats {
    match class {
        "Mage" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: -1,
            glyph: 'm',
        },
        "Rogue" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            glyph: 'r',
        },
        "Amazon" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            glyph: 'a',
        },
        "Archer" => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
            glyph: 'b',
        },
        // Barbarian, and a safe fallback for any unrecognized class name.
        _ => ClassBaseStats {
            health: 10,
            damage: 1,
            speed: 6,
            defense: 0,
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
/// placed this level - see Templates::spawn_prefab_weapon. Swords and
/// Staffs share this same single guaranteed slot.
pub fn spawn_prefab_weapon(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_point: Option<Point>,
) {
    let template = Templates::load();
    template.spawn_prefab_weapon(ecs, rng, level, spawn_point);
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
