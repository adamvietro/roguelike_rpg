use crate::prelude::*;

/// Which top-level game mode the player picked at the new AdventureSelect
/// screen (see screens/title.rs). Kept as a plain `State` field (like
/// `options_awaiting`/`stats_selected_class`), not a resource - it's
/// screen-navigation state read only by class_select to decide which
/// start function to call (`State::start_game` vs `State::start_arena`),
/// not something any gameplay system needs to see.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdventureMode {
    DungeonCrawl,
    BattleArena,
}

/// How many enemies each of a level's 3 waves spawns, indexed by
/// wave - 1 (wave 1 -> index 0). The level's boss spawns after wave 3
/// clears, not as a 4th wave - see State::handle_arena_kill.
pub const ARENA_WAVE_ENEMY_COUNTS: [u32; 3] = [5, 5, 3];

/// FieldOfView radius applied to every Arena enemy and boss (see
/// State::arena_begin_wave/arena_spawn_boss_on_current_map), overriding
/// the small radius spawn_entity gives ordinary dungeon enemies (6 -
/// tuned for a fog-of-war dungeon crawl, where enemies are meant to only
/// notice a nearby player). An Arena wave's whole point is a small,
/// fully-visible clearing - 30 comfortably covers the ~20-tile diameter
/// of the circular arena (see MapBuilder::new_arena_wave) from any point
/// inside it, so an enemy notices and starts chasing the moment it
/// spawns rather than standing idle at the edge until the player
/// wanders within a dungeon-sized detection range.
pub const ARENA_ENEMY_FOV_RADIUS: i32 = 30;

/// How much gold a Battle Arena run begins with - only matters for the
/// very first shop, since every later shop is entered with whatever gold
/// survived the levels before it.
pub const ARENA_STARTING_GOLD: i32 = 25;

/// Flat gold cost of one Healing Potion.
pub const HEALING_POTION_PRICE: i32 = 5;

/// Flat gold cost of one charge of ANY ability (Technique or Effect,
/// regardless of class) - deliberately not split by class or by
/// Technique-vs-Effect, matching how the shop's own roll doesn't weight
/// one ability over another either (see class_ability_names_for_class).
pub const ABILITY_PRICE: i32 = 12;

/// Gold cost of a weapon, indexed by its (0-indexed) template tier - a
/// weapon at template_level 0 costs WEAPON_TIER_PRICES[0], and so on.
/// Flat across classes at the same tier (every class's tier-0 weapon
/// costs the same 20g, etc.) - deliberately not scaled per class, since
/// base_damage is already roughly symmetric across classes at a given
/// tier in template.ron.
pub const WEAPON_TIER_PRICES: [i32; 3] = [20, 40, 70];

/// Gold an ordinary (non-boss) Arena enemy drops when it dies, regardless
/// of HOW it died (real battle, a ranged strike, or a Trap) - see
/// gold_reward_for_kill.
pub const ENEMY_GOLD_DROP: i32 = 5;

/// Gold an Arena boss drops when it dies - 4x a regular kill, matching
/// how a boss already guarantees a loot drop elsewhere in this project
/// (see Templates::grant_random_battle_loot).
pub const BOSS_GOLD_DROP: i32 = 20;

/// How much gold one kill is worth, in the Battle Arena - shared by every
/// kill path (a real battle victory, a ranged strike, a Trap) so income
/// is consistent no matter how the enemy actually died. Callers are
/// expected to gate this on the player actually having a Gold component
/// at all (see Gold's own doc comment) - a Dungeon Crawl kill should
/// never call this in the first place.
pub fn gold_reward_for_kill(is_boss: bool) -> i32 {
    if is_boss {
        BOSS_GOLD_DROP
    } else {
        ENEMY_GOLD_DROP
    }
}

/// A brief status line the arena shop shows for one interaction - "Not
/// enough gold!" on a failed buy attempt (see player_input.rs's
/// buy_nearby_item). Cleared automatically on the very next keypress
/// (whatever it is) unless that keypress is itself another buy attempt
/// that fails again - see systems/player_input.rs - so a stale message
/// never lingers once the player's moved on. Only ever Some while
/// ShoppingActive; a Dungeon Crawl run never touches this at all.
#[derive(Clone, Debug, PartialEq)]
pub struct ShopMessage(pub String);

/// Tracks progress through a Battle Arena run. Inserted as a resource
/// (`Some(ArenaRun{..})`) only when a Battle Arena run is active -
/// `start_game` (normal dungeon crawl) always inserts `None::<ArenaRun>`
/// instead. Other systems tell an arena run apart from an ordinary
/// dungeon crawl purely by checking this resource's presence, e.g.
/// end_turn's Exit-tile handling below.
#[derive(Clone, Copy, Debug)]
pub struct ArenaRun {
    /// Which of the 3 arena levels this is, 1..=3. Maps onto the
    /// existing dungeon template levels 0..=2 (see `template_level`) -
    /// arena level 1 uses the same Goblin/Goblin Chieftain pool as
    /// dungeon floor 0, arena level 2 uses dungeon floor 1's Orc/Orc
    /// Warlord pool, and so on. While the player is in a shop (before
    /// this level's waves begin), this is already the level the shop is
    /// "preparing you for", not a separate level 0.
    pub level: u8,
    /// Which wave within this level the player is currently on, 1..=3 -
    /// only meaningful while `boss_active` is false. 0 means "no wave
    /// has started yet" (still in the shop).
    pub wave: u8,
    /// True once this level's boss has been spawned (after wave 3
    /// clears) - a kill while this is true means the level itself is
    /// cleared, not just a wave, and routes to the next level's shop (or
    /// final Victory on level 3) instead of the next wave. See
    /// State::handle_arena_kill.
    pub boss_active: bool,
    /// Where this level's boss should spawn, once wave 3 clears - set
    /// each time a wave's arena map is built (State::arena_begin_wave)
    /// and read once, when the boss actually spawns
    /// (State::arena_spawn_boss_on_current_map), since the boss appears
    /// on the SAME map wave 3 was fought on rather than a freshly built
    /// one.
    pub boss_spawn: Point,
}

impl ArenaRun {
    /// A fresh run about to enter `level`'s shop - no wave started yet,
    /// no boss out.
    pub fn new(level: u8) -> Self {
        Self {
            level,
            wave: 0,
            boss_active: false,
            boss_spawn: Point::zero(),
        }
    }

    /// Converts this run's 1..=3 arena level into the 0..=2 index
    /// `template.ron`'s `levels:` sets and spawn_level/spawn_boss expect.
    pub fn template_level(&self) -> usize {
        (self.level - 1) as usize
    }
}

/// Marker resource (`Option<ShoppingActive>`, same "None means not
/// active" convention as Option<Battle>/Option<ArenaRun>) - present only
/// while the player is browsing an arena shop. Its purpose is narrow and
/// deliberate: while Some, systems/movement.rs's normal walk-onto-it
/// auto-pickup is suppressed, and systems/player_input.rs's new buy key
/// (Enter) is enabled instead - so shop items are a deliberate choice,
/// not something the player sweeps up by walking past. Scoped
/// separately from ArenaRun itself (rather than just checking
/// ArenaRun.is_some()) because once wave combat exists, an ArenaRun will
/// still be active during a wave fight, where normal auto-pickup of
/// battle loot should keep working exactly like it does in a dungeon
/// crawl - only the shop screens want this suppressed.
#[derive(Clone, Copy, Debug)]
pub struct ShoppingActive;

/// How many of this item remain on the shop counter - the counter entity
/// itself is a lightweight display/bookkeeping marker (Point + Render +
/// Name + this), NOT a real usable Item (no Effect/Technique/Weapon
/// components) - buying one calls spawner::spawn_named_item_via_commands
/// to grant the player a real, fully-built copy, then decrements this.
/// Reaching 0 removes the counter entity entirely rather than leaving a
/// "0 remaining" marker behind - see player_input.rs's buy_nearby_item.
#[derive(Clone, Copy, Debug)]
pub struct ShopStock(pub i32);

/// Rolls this shop's full stock list - deliberately BEFORE the room
/// itself is built, so MapBuilder::new_arena_shop can size and center
/// the counter around however many distinct items this returns, instead
/// of always reserving a fixed 11 slots regardless of how many are
/// actually used. Returns (name, quantity, unit_price) triples: the
/// class's weapon at `template_level` if one exists (quantity 1, priced
/// by tier - see WEAPON_TIER_PRICES), a Healing Potion stack (quantity 5,
/// HEALING_POTION_PRICE each), then however many distinct abilities came
/// up across 5 random rolls with replacement (ABILITY_PRICE each) -
/// identical rolls are grouped into one stack (3 identical rolls become
/// one entry with quantity 3, not three separate entries), so a class
/// with fewer than 5 distinct abilities still shows a sensible small
/// counter instead of padding it out. The ability pool covers BOTH
/// in-battle Techniques and out-of-combat Effects (see
/// class_ability_names_for_class) - previously it only drew from
/// Techniques, so classes with Effect-based abilities (Rogue's Stealth,
/// Amazon's Throw Spear/Poison Spear, Hunter's Shoot/Freeze Trap, Mage's
/// Invisible Cloak/Ice Armor) could never have those show up in a shop no
/// matter how many times this ran. `unit_price` is the cost of ONE unit -
/// buying decrements ShopStock by 1 per purchase (see player_input.rs's
/// buy_nearby_item), so this is never a whole-stack price.
pub fn roll_arena_shop_items(
    rng: &mut RandomNumberGenerator,
    class: &str,
    template_level: usize,
) -> Vec<(String, i32, i32)> {
    let mut items = Vec::new();

    if let Some(weapon_name) = weapon_name_for_class_level(class, template_level) {
        let price = WEAPON_TIER_PRICES
            .get(template_level)
            .copied()
            .unwrap_or(*WEAPON_TIER_PRICES.last().unwrap());
        items.push((weapon_name, 1, price));
    }
    // No weapon-name match is possible in principle (every playable
    // class has a weapon at levels 0/1/2), but this just leaves the
    // weapon out of the list rather than panicking if template.ron is
    // ever missing one for a new class - same "warn, don't crash"
    // spirit as Templates::spawn_named_item's unknown-name handling.
    // template_level is likewise always 0/1/2 in practice (see
    // ArenaRun::template_level), but the fallback to the last tier's
    // price is cheap insurance against an out-of-range index if that
    // ever changes, rather than panicking on .get().unwrap().

    items.push(("Healing Potion".to_string(), 5, HEALING_POTION_PRICE));

    let abilities = class_ability_names_for_class(class);
    for _ in 0..5 {
        if abilities.is_empty() {
            break;
        }
        let name = abilities[rng.range(0, abilities.len() as i32) as usize].clone();
        match items.iter_mut().find(|(n, _, _)| *n == name) {
            Some((_, count, _)) => *count += 1,
            None => items.push((name, 1, ABILITY_PRICE)),
        }
    }
    // A class with zero defined abilities (shouldn't happen for any of
    // the 5 real classes, all of which have several) just sells no
    // abilities at all rather than panicking on an empty-range
    // rng.range call.

    items
}
