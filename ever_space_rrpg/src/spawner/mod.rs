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

/// Base stats + glyph for a given class name - every class in
/// CLASS_ROSTER (screens/title.rs) now has its own designed numbers;
/// only an unrecognized class name falls through to the generic
/// fallback at the bottom. Each class still gets its own glyph.
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
            speed: 7,
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
            health: 12,
            damage: 1,
            speed: 6,
            defense: 0,
            evasion: 5,
            glyph: 'a',
        },
        "Hunter" => ClassBaseStats {
            health: 12,
            damage: 1,
            speed: 8,
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
            // Was 'D' - collided with Deathblow's own already-finalized
            // icon (Barbarian), so playing Debug rendered the player's
            // own map/portrait sprite as Deathblow's icon instead of a
            // distinct look. Moved to a genuinely free codepoint - see
            // Dungeon_Font_Glyph_to_Cell_Map.md.
            glyph: 'N',
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
    // Pre-seed an already-expired MovingAnimation, exactly like
    // spawn_entity does for every enemy (see spawner/template.rs) and
    // for the same reason - except this now matters for the player too,
    // where it never used to: camera_render_offset (components.rs)
    // reads the player's own gliding_position every frame to decide
    // whether the camera should be panning, so the player's very first
    // move of a level is no longer purely cosmetic the way the old
    // camera-snaps-instantly design left it. Without this, that first
    // step would still work, but as a genuinely new component being
    // attached (a structural ECS change other systems can read one
    // frame late), it could show the same one-frame hiccup enemies used
    // to show on their own first step, before template.rs's spawn_entity
    // got this same fix. Pre-seeding closes that gap here too.
    commands.add_component(
        player,
        MovingAnimation {
            start: pos,
            end: pos,
            elapsed_ms: MOVE_ANIM_DURATION_MS,
        },
    );
    // See idle_frames_for_class's own doc comment - pulls real walk-cycle
    // frames from resources/character_idle.png for every class in
    // CLASS_ROSTER (a placeholder row of the same dungeon portrait
    // repeated 5x for any class real art hasn't arrived for yet, baked
    // into the sheet itself - no branching needed here).
    commands.add_component(player, idle_frames_for_class(class, to_cp437(stats.glyph)));
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

/// Spawns the toughest non-boss enemy for `level` at each of a chest's
/// guard positions - see Templates::spawn_prefab_chest_guards.
pub fn spawn_prefab_chest_guards(ecs: &mut World, level: usize, spawn_points: &[Point]) {
    let template = Templates::load();
    template.spawn_prefab_chest_guards(ecs, level, spawn_points);
}

/// Spawns a guaranteed loot chest entity at `pos` - see components::Chest
/// and systems/movement.rs, which handles the player walking onto it.
/// Not a template-driven Item (a chest isn't usable/carryable, just an
/// interactive prop), so this pushes its components directly, the same
/// way spawn_amulet_of_yala below does for the other non-template special
/// entity in this game.
pub fn spawn_chest(ecs: &mut World, pos: Point) {
    ecs.push((
        Chest,
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('c'),
        },
        Name("Treasure Chest".to_string()),
    ));
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

/// Spawns the level's guaranteed boss at `spawn_point` (always
/// MapBuilder::amulet_start - see Templates::spawn_boss for why that
/// point in particular). Silently does nothing if no boss template is
/// defined for this level yet.
pub fn spawn_boss(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_point: Point,
) {
    let template = Templates::load();
    template.spawn_boss(ecs, rng, level, spawn_point);
}

/// Rolls a chance to grant the player a random one-time battle item after
/// a battle victory, filtered to items matching the player's own Class -
/// guaranteed instead of rolled if `enemy` is a boss (see
/// Templates::grant_random_battle_loot). Returns the granted item's
/// display name, if any.
pub fn grant_random_battle_loot(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    player: Entity,
    enemy: Entity,
) -> Option<String> {
    let player_class = entity_class(ecs, player)?;
    let template = Templates::load();
    template.grant_random_battle_loot(ecs, rng, player, enemy, &player_class)
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

/// Places a shop-counter stock marker at `pt` for `quantity` units of
/// `name`, priced at `price` gold per unit - see Templates::spawn_shop_stock_at.
pub fn spawn_shop_stock_at(ecs: &mut World, name: &str, pt: Point, quantity: i32, price: i32) {
    let template = Templates::load();
    template.spawn_shop_stock_at(ecs, name, pt, quantity, price);
}

/// Grants `player` a real copy of `name` via a CommandBuffer - see
/// Templates::spawn_named_item_via_commands. Used by the arena shop's
/// buy handler (systems/player_input.rs), which only has a
/// SubWorld + CommandBuffer to work with, not a real `&mut World`.
pub fn spawn_named_item_via_commands(
    name: &str,
    player: Entity,
    commands: &mut legion::systems::CommandBuffer,
) {
    let template = Templates::load();
    template.spawn_named_item_via_commands(name, player, commands);
}

/// Spawns one exact named enemy via a CommandBuffer - see
/// Templates::spawn_named_enemy_via_commands. Used by the Debug class's
/// "Battle 4" cheat (systems/use_items.rs), which only has a SubWorld +
/// CommandBuffer to work with, not a real `&mut World`.
pub fn spawn_named_enemy_via_commands(
    name: &str,
    pt: Point,
    commands: &mut legion::systems::CommandBuffer,
) -> Option<Entity> {
    let template = Templates::load();
    template.spawn_named_enemy_via_commands(name, pt, commands)
}

/// Whether `name`'s template is a weapon - see Templates::item_is_weapon.
pub fn item_is_weapon(name: &str) -> bool {
    let template = Templates::load();
    template.item_is_weapon(name)
}

/// Which weapon template the arena shop should offer `class` at
/// `level` - see Templates::weapon_name_for_class_level.
pub fn weapon_name_for_class_level(class: &str, level: usize) -> Option<String> {
    let template = Templates::load();
    template.weapon_name_for_class_level(class, level)
}

/// Every ability name (Technique or Effect) defined for `class` - see
/// Templates::class_ability_names_for_class. Used by the Battle Arena shop
/// to roll its 5 random ability slots - deliberately broader than
/// class_technique_names below, since the shop sells out-of-combat
/// abilities too, not just in-battle Techniques.
pub fn class_ability_names_for_class(class: &str) -> Vec<String> {
    let template = Templates::load();
    template.class_ability_names_for_class(class)
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

/// Every distinct out-of-combat ability name defined for `class`,
/// regardless of ownership, in template.ron's own file order - see
/// Templates::effect_names_for_class. Used by components::ability_bar_slots
/// (via systems/hud.rs) to show a class's full out-of-combat roster on
/// the Ability Bar with unowned ones greyed out - the exact same
/// always-show-the-roster convention class_technique_names already
/// established for the battle menu.
pub fn class_effect_names(class: &str) -> Vec<String> {
    let template = Templates::load();
    template.effect_names_for_class(class)
}

/// Every distinct universal usable item name (no class restriction),
/// regardless of ownership, in template.ron's own file order - see
/// Templates::universal_item_names. Used by components::item_bar_slots
/// (via systems/hud.rs) to show the full roster of possible universal
/// items on the Item Bar with unowned ones greyed out - the same
/// always-show-the-roster convention class_effect_names/
/// class_technique_names already established for the other two bars.
pub fn universal_item_names() -> Vec<String> {
    let template = Templates::load();
    template.universal_item_names()
}

/// The glyph a template named `name` renders as - see
/// Templates::glyph_for_name. Used by the Ability Bar (systems/hud.rs) to
/// show an icon for a roster slot even when the player doesn't currently
/// own any copies of it (so there's no carried entity to read a Render
/// component from).
pub fn glyph_for_item_name(name: &str) -> Option<char> {
    let template = Templates::load();
    template.glyph_for_name(name)
}

/// The Description text a template named `name` carries, if any - see
/// Templates::description_for_name. Used by the Ability Bar so hovering
/// an unowned (greyed-out) roster slot still shows a real description,
/// not nothing.
pub fn description_for_item_name(name: &str) -> Option<String> {
    let template = Templates::load();
    template.description_for_name(name)
}
