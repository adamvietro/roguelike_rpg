pub use crate::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Render {
    pub color: ColorPair,
    pub glyph: FontCharType,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Player {
    pub map_level: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Enemy;

/// Marks an Enemy entity as a level boss - guards the Exit/Amulet tile
/// (see spawner::spawn_boss / Templates::spawn_boss, which places one at
/// MapBuilder::amulet_start on every level). Doesn't change any combat
/// mechanics by itself yet - stats/abilities are a separate pass - this
/// just identifies the entity for that future work and for anything
/// (rendering, HUD, AI) that wants to treat a boss differently later.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Boss;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weapon;

/// A placed hazard entity - Amazon's Trap (see ProvidesEffect::PlaceTrap /
/// systems/use_items.rs, which spawns one of these at the player's
/// position) and systems/traps.rs, which checks every enemy's position
/// against these each monster turn. Deals `damage` to the first enemy
/// that steps onto its tile, then is removed - single use. The player is
/// never affected by their own (or any) trap; only entities with an
/// Enemy component are checked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trap {
    pub damage: i32,
}

/// A placed hazard entity - Hunter's Freeze Trap (see
/// ProvidesEffect::PlaceFreezeTrap / systems/use_items.rs, which spawns
/// one of these at the player's position). Checked by the same
/// systems/traps.rs pass that already checks Trap above, against every
/// enemy's position each monster turn - instead of damage, the first
/// enemy that steps onto its tile is given a Frozen status for `turns`
/// turns, then this hazard is removed - single use, same as Trap. The
/// player is never affected by their own (or any) freeze trap; only
/// entities with an Enemy component are checked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreezeTrap {
    pub turns: i32,
}

/// An enemy currently frozen by a Freeze Trap (see FreezeTrap /
/// ProvidesEffect::PlaceFreezeTrap) - fully inert for `turns_remaining`
/// more player turns: systems/chasing.rs skips it entirely (no movement,
/// so no chasing AND no bumping into the player to start a battle
/// either, since that's the same mechanism), and entity_render tints it
/// blue for the duration (see tinted_color). Ticked down once per
/// completed player turn in systems/end_turn.rs, the same cadence
/// Invisible/Stealthed already use, and removed once it reaches zero.
/// Deliberately dungeon-view only - a frozen enemy that the player walks
/// into anyway still starts a perfectly normal battle, with no
/// carry-over status once inside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frozen {
    pub turns_remaining: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmuletOfYala;

// --- Out-of-combat item effects ---------------------------------------------
//
// One component + data enum, mirroring Technique/TechniqueEffect below -
// replaces the old approach of a separate marker component per effect
// (ProvidesHealing, ProvidesDungeonMap, ProvidesInvisibility) with one
// generic Effect component whose meaning is data, not a distinct Rust
// type. An item has at most one such effect.

/// An out-of-combat item's effect, applied when used from the dungeon-view
/// item list (see systems/use_items.rs) - set from `Template.effect` in
/// template.ron.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub enum ProvidesEffect {
    /// Restore `0` HP immediately - the i32 is the amount healed.
    Healing(i32),
    /// Reveal the entire map immediately.
    MagicMap,
    /// Become Invisible (see components::Invisible) for `0` player turns -
    /// the i32 is the move count.
    Invisibility(i32),
    /// Gain `defense_bonus` extra Defense for the next `attacks` enemy
    /// hits taken (see components::IceArmored) - applied immediately and
    /// persists across dungeon exploration and into battle, unlike the
    /// old in-battle-only Shield technique this replaces. "Mages buff
    /// before battle."
    IceArmor { defense_bonus: i32, attacks: i32 },
    /// Become Stealthed (see components::Stealthed) for `0` player moves.
    /// Unlike Invisibility, which blocks a bump-into-enemy from starting a
    /// battle at all, Stealth lets the battle start but grants it an
    /// ambush bonus (forced first turn + 3x damage on the opening action -
    /// see systems/player_input.rs and battle::Battle::sneak_attack).
    Stealth(i32),
    /// Debug-class item: instantly ends the run as a win, exactly as if
    /// the Amulet of Yala had been claimed (see systems/end_turn.rs,
    /// which sets TurnState::Victory the same way on the real win
    /// condition).
    DebugWin,
    /// Debug-class item: instantly ends the run as a loss, exactly as if
    /// the player's Health had hit 0 (see systems/end_turn.rs).
    DebugLose,
    /// Debug-class item: instantly triggers the same level-transition
    /// State::advance_level runs when stepping on a real Exit tile (see
    /// systems/end_turn.rs / main.rs's TurnState::NextLevel dispatch).
    DebugNextLevel,
    /// Amazon's Throw Spear: damages the nearest currently-visible (in the
    /// player's FieldOfView - real line-of-sight, not just distance, see
    /// systems/fov.rs) enemy for the player's normal attack damage plus
    /// this flat bonus - entirely outside battle, no fight starts. See
    /// systems/use_items.rs. Does nothing (but is still consumed, same as
    /// every other item used at a moment it has nothing to affect) if no
    /// enemy is currently visible.
    RangedStrike(i32),
    /// Amazon's Trap: places a Trap entity (see components::Trap) at the
    /// user's current position, dealing `0` damage to the first enemy
    /// that steps onto it - see systems/traps.rs. The i32 is the trap's
    /// damage, not a duration/count like most other effects here.
    PlaceTrap(i32),
    /// Hunter's Freeze Trap: places a FreezeTrap entity (see
    /// components::FreezeTrap) at the user's current position - same
    /// spot/spirit as PlaceTrap above, but the first enemy that steps
    /// onto it is given a Frozen status for `0` turns instead of taking
    /// damage (see components::Frozen / systems/traps.rs). The i32 is
    /// the freeze duration, not a damage amount.
    PlaceFreezeTrap(i32),
}

/// Marks an Item entity with its out-of-combat effect. Granted via normal
/// floor spawns, starting kits, or battle loot, and consumed on use in
/// systems/use_items.rs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Effect(pub ProvidesEffect);

// --- One-time battle items -------------------------------------------------
//
// A single component + data enum replaces what used to be one marker
// component (ProvidesDeathblow, ProvidesQuickAttack, ...) and one
// near-identical `carried_*` lookup function PER technique, all baked to
// the Barbarian class specifically. The mechanical shape of a technique is
// now data (TechniqueEffect), not a Rust type - so a new class's technique
// that reuses an existing shape (e.g. a Mage spell that's also "multiply
// damage") is a template.ron entry only, no code change. A genuinely new
// mechanic still needs a new variant here, but only ONE new match arm
// (in battle::apply_player_technique) instead of a new component, a new
// BattleAction variant, a new carried_* function, AND a new main.rs match
// arm like before.

/// One battle technique's mechanical effect. Attached to an Item entity via
/// `Technique`, set from `Template.technique` in template.ron - see
/// spawner/template.rs. Interpreted in one place: battle::apply_player_technique.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
pub enum TechniqueEffect {
    /// Attack right now for `multiplier`x normal attack damage.
    DamageMultiplier(i32),
    /// Attack for a fixed `amount` plus any carried weapon damage (unlike
    /// DamageMultiplier, which scales off your normal attack rather than
    /// adding a flat base) - e.g. Fireball: 2 base damage, more with a
    /// better staff.
    FlatDamage(i32),
    /// Attack `hits` times in a row, each for full normal attack damage.
    MultiHit(i32),
    /// Skip your attack this turn; the next hit the enemy lands on you has
    /// `chance_percent` chance to reflect `multiplier`x your normal attack
    /// damage back at them.
    Counter {
        chance_percent: i32,
        multiplier: i32,
    },
    /// Wound the enemy for `damage` at the start of each of the next
    /// `turns` rounds.
    DamageOverTime { damage: i32, turns: i32 },
    /// Restore `amount` HP to yourself right now. Not used by any current
    /// Barbarian technique - included so a Mage healing spell has
    /// somewhere to plug in without another battle.rs change.
    Heal { amount: i32 },
    /// Gain `chance_percent` additional chance to fully evade an incoming
    /// attack (stacking with the entity's base Evasion, see
    /// components::Evasion) for the next `turns` enemy attacks faced -
    /// see battle::resolve_enemy_attack / Battle::dodge_bonus.
    Evade { chance_percent: i32, turns: i32 },
    /// Reduce the enemy's outgoing damage by a random amount between
    /// `min_reduction` and `max_reduction` (inclusive) for the next
    /// `attacks` enemy attacks actually faced (an evaded attack doesn't
    /// consume a charge - see battle::resolve_enemy_attack). Amazon's
    /// Battle Cry - deliberately random rather than the flat numbers
    /// every other effect in this project uses, per design.
    WarCry {
        min_reduction: i32,
        max_reduction: i32,
        attacks: i32,
    },
    /// Deal `initial` damage (plus carried weapon damage, same shape as
    /// FlatDamage) right now, THEN also apply a DamageOverTime-style
    /// wound of `dot_damage` per turn for the next `dot_turns` rounds -
    /// Amazon's Poison Spear. Reuses the exact same Battle::enemy_dot /
    /// tick_dot machinery Rend/Burn/Garrote already use for the DOT half;
    /// only the "also hit immediately" half is new.
    PoisonStrike {
        initial: i32,
        dot_damage: i32,
        dot_turns: i32,
    },
    /// Roll `chance_percent` right now; on success, the enemy is unable
    /// to attack for its next `turns` turns (see Battle::enemy_stunned /
    /// battle::resolve_enemy_attack, which skips the attack entirely and
    /// ticks this down - no Defend/Ice Armor/Counter gets consumed on a
    /// stunned turn, same as a fully-evaded one). On failure, nothing
    /// happens - the technique is still consumed either way, same as
    /// every other one-time item. Hunter's Stun.
    Stun { chance_percent: i32, turns: i32 },
    /// Guaranteed (no roll), unlike Stun above, but only for the enemy's
    /// very next single attack (see resolve_enemy_attack, via the same
    /// Battle::enemy_stunned field Stun uses - narratively different
    /// ("playing dead" vs. a hard stun) but mechanically identical: the
    /// enemy skips N of its own attacks). Hunter's Feint - used instead
    /// of attacking that round, to sit out the enemy's next hit for
    /// free rather than gambling on Defend.
    Feint,
    /// Strikes EVERY enemy currently in the battle, `hits` times each,
    /// for (normal attack damage + `bonus_damage`) per hit - the first
    /// class of technique that hits more than one target, now that a
    /// battle can hold more than one enemy (see battle::MAX_BATTLE_ENEMIES).
    /// In a solo fight this behaves exactly like MultiHit(hits) against
    /// the one enemy present (bonus_damage lets Whirlwind's own "+2 per
    /// hit" flavor exist without a separate variant). Each class's own
    /// AOE (Flurry/Whirlwind/Blizzard/Javelin Volley/Arrow Volley) is
    /// this same variant with different hits/bonus_damage - see
    /// resources/template.ron.
    AoeMultiHit { hits: i32, bonus_damage: i32 },
}

/// Marks an Item entity as a one-time battle technique and carries its
/// effect. Granted after battle (see
/// spawner::Templates::grant_random_battle_loot) and consumed on use in
/// battle::apply_player_technique.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Technique(pub TechniqueEffect);

/// Temporary out-of-combat status granted by the Invisible Cloak. While
/// active, walking into an enemy is blocked like a wall instead of
/// starting a battle (see systems/player_input.rs) - items can still be
/// picked up as normal. Ticks down by one every player turn in
/// systems/end_turn.rs, and is removed once it reaches zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Invisible {
    pub moves_remaining: i32,
}

/// Temporary out-of-combat status granted by Stealth (Rogue). Unlike
/// Invisible, walking into an enemy while Stealthed does NOT block the
/// battle - it starts normally, but as an ambush: see
/// systems/player_input.rs (checks this to arm Battle::sneak_attack, then
/// consumes/removes this component) and battle.rs's forced first-turn +
/// damage-multiplier handling. Ticks down by one every player turn in
/// systems/end_turn.rs, same as Invisible, and is removed once it reaches
/// zero (or immediately, if consumed by an ambush first).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stealthed {
    pub moves_remaining: i32,
}

/// Persistent Defense buff granted by Ice Armor (see ProvidesEffect::IceArmor).
/// Applied immediately on use, outside combat - "Mages buff before battle."
/// Checked and ticked down entirely within battle::resolve_enemy_attack:
/// unlike the old Shield technique it replaces, this survives across
/// multiple turns AND multiple battles until its `attacks_remaining` runs
/// out, since it's a lasting status on the player rather than something
/// scoped to a single Battle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IceArmored {
    pub defense_bonus: i32,
    pub attacks_remaining: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingRandomly;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChasingPlayer;

/// Marker for entities that only exist as part of the purely decorative
/// title/class-select background (see spawn_title_background) - never a
/// real gameplay actor. Tells movement_system to skip attaching a
/// MovingAnimation to this entity when it steps - see that system's own
/// comment for why: entity_render routes anything with an in-flight
/// MovingAnimation onto GLIDE_CONSOLE, which is registered (and therefore
/// composites) ABOVE the class-select icons (console 3) and every
/// headline (BIG_TEXT_CONSOLE), so a background enemy mid-step would
/// otherwise render on top of them. Losing the smooth per-step glide for
/// a decorative-only entity is not visible in practice - it already only
/// takes a step once every BACKGROUND_MOVE_INTERVAL_MS (see title.rs),
/// nowhere near often enough for the missing glide to read as a stutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecorativeOnly;

/// How much gold the player currently holds. Attached at the start of
/// EVERY run now - Battle Arena grants ARENA_STARTING_GOLD (see
/// State::start_arena), Dungeon Crawl starts at 0 (see State::start_game)
/// since its first shop only comes after a whole floor's worth of kills
/// and a guaranteed chest, unlike Arena's very first shop which has no
/// prior kills to draw on. Every gold-granting/spending codepath
/// (systems/use_items.rs, systems/traps.rs, battle::finish_battle_victory,
/// player_input.rs's buy_nearby_item) still relies on checking this
/// component's PRESENCE, not a mode/resource check, to decide "does this
/// kill/purchase even involve gold" - simpler, and it can't drift out of
/// sync with which mode actually granted it; that check just now always
/// passes, in both modes. Lives directly on the player entity, so it
/// automatically survives every world-rebuild that preserves the player
/// (Arena's wave/shop transitions - see State::arena_rebuild_keep_player -
/// and Dungeon Crawl's own advance_level) with no separate carry-over
/// logic needed, the same way Health/Carried items already survive those
/// rebuilds for free.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gold(pub i32);

/// Marks a guaranteed dungeon-floor loot chest entity - see
/// map_builder/prefab.rs's chest room (guarded by
/// spawner::spawn_prefab_chest_guards) and systems/movement.rs, which
/// grants its contents and removes it the moment the player walks onto
/// its tile - the same "auto" convention floor Items already use, not a
/// separate open keypress. Not a template-driven Item itself (see
/// spawner::spawn_chest) since a chest isn't usable/carryable on its own,
/// just an interactive prop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chest;

/// What a chest just granted - gold plus a display name per item granted
/// (e.g. `["Dungeon Map", "Healing Potion", "Healing Potion"]`), shown by
/// the ChestOpened screen (screens/chest.rs) then reset to `None` on
/// dismiss, mirroring `Option<BattleVictory>`'s own dismiss-to-None
/// convention exactly (see screens/battle.rs's battle_victory_tick).
#[derive(Clone, Debug, PartialEq)]
pub struct ChestLoot {
    pub gold: i32,
    pub items: Vec<String>,
}

/// The gold cost to buy ONE unit of a shop counter item - a companion
/// component on the same ShopStock counter-marker entity (see
/// spawner::spawn_shop_stock_at), not on the eventual real item
/// spawn_named_item_via_commands grants once bought. See
/// arena::{HEALING_POTION_PRICE, ABILITY_PRICE, WEAPON_TIER_PRICES}.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Price(pub i32);

/// The ShopStock counter item currently on `pos`'s own tile or directly
/// (orthogonally) adjacent to it, if any - (entity, remaining count,
/// display name, price). Shared by player_input.rs's buy_nearby_item
/// (the actual purchase) and systems/hud.rs's shop tooltip (what gets
/// shown while standing there), so what the tooltip displays always
/// matches what pressing Enter would actually buy - previously each had
/// its own independent copy of this exact query before the tooltip
/// needed the same lookup.
///
/// Shop items are ShopStock counter markers, not real Items sitting on
/// the floor (see spawner::spawn_shop_stock_at) - the counter row is a
/// Wall tile (MapBuilder::new_arena_shop), so the player can never
/// actually stand ON one, only in the walkable row directly below it.
/// That structurally guarantees this only ever matches the one item
/// directly in front of `pos`, not a neighbor one column over.
pub fn shop_item_near<T: EntityStore>(
    ecs: &T,
    pos: Point,
) -> Option<(Entity, i32, String, i32)> {
    const ADJACENT: [Point; 5] = [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 0, y: 1 },
        Point { x: -1, y: 0 },
        Point { x: 1, y: 0 },
    ];

    <(Entity, &ShopStock, &Point, &Name, &Price)>::query()
        .iter(ecs)
        .filter(|(_, _, &p, _, _)| ADJACENT.iter().any(|&d| p == pos + d))
        .map(|(e, stock, _, name, price)| (*e, stock.0, name.0.clone(), price.0))
        .next()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WantsToMove {
    pub entity: Entity,
    pub destination: Point,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Clone, PartialEq)]
pub struct Name(pub String);

/// A character's class, e.g. "Barbarian". Gates which class-restricted
/// battle items (see Template.class in spawner/template.rs) an entity can
/// use or be granted as loot. Represented as a plain string, matching how
/// `provides` tags already work in template.ron, so new classes are a
/// content change, not a code change.
#[derive(Clone, PartialEq)]
pub struct Class(pub String);

/// Marks an Item entity as a one-time battle technique (see Technique)
/// rather than a regular carried item (potion, weapon, map). Lets the HUD
/// split the ordinary "Items carried" list (left) from a separate "Battle
/// Attacks" panel (right) - see systems/hud.rs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BattleItem;

/// The mouse position captured in the Ability Bar console's own
/// coordinate space (see main.rs::ABILITY_BAR_CONSOLE) - captured
/// directly in that console's own (coarse, few-cells) grid rather than
/// converted from a finer one, since converting DOWN loses no precision
/// but converting UP can't recover it. Used by systems/hud.rs to detect
/// hovering a bar slot for its tooltip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AbilityBarMousePos(pub Point);

/// True for exactly one frame per physical left-click - the instant the
/// button transitions from up to down, computed in main.rs::tick() from
/// bracket-lib's own level-state query (`INPUT.lock().is_mouse_button_
/// pressed(0)`) compared against the previous frame's state, NOT from
/// `BTerm::left_click`. `left_click` is edge-triggered per mouse-button
/// EVENT rather than per physical click - bracket-terminal's own
/// `on_mouse_button` sets it unconditionally for button 0 regardless of
/// whether the event was a press or a release, so a single click (one
/// press + one later release) sets `left_click` true on two separate
/// frames. A naive `if ctx.left_click` on the Item Bar's click-to-use
/// handler would fire twice per click. Comparing consecutive frames of
/// the real held/not-held state sidesteps that quirk entirely rather than
/// trying to debounce it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MouseLeftJustPressed(pub bool);

/// Flavor/mechanical text shown when hovering an item's HUD listing (see
/// systems/hud.rs). Optional - only items that want a tooltip need one.
#[derive(Clone, PartialEq)]
pub struct Description(pub String);

/// Converts a mouse position captured in console 0's cell coordinates
/// (32px tiles) into the HUD console's cell coordinates. Both consoles
/// span the same physical window, so this is just a ratio of cell counts
/// - shared by hud.rs (hover-detecting a Battle Attacks row) and
/// tooltips.rs (hover-detecting a dungeon tile).
pub fn mouse_to_hud(mouse_pos: Point) -> Point {
    Point::new(
        (mouse_pos.x as f32 * HUD_COLS as f32 / DISPLAY_WIDTH as f32) as i32,
        (mouse_pos.y as f32 * HUD_ROWS as f32 / DISPLAY_HEIGHT as f32) as i32,
    )
}

#[derive(Clone, PartialEq)]
pub struct Carried(pub Entity);

/// Every Carried+Item entity belonging to `wielder` that's actually
/// usable via a number-key press - i.e. NOT a Weapon (equipped/applied
/// automatically, see carried_weapon_damage in battle.rs) and NOT a
/// BattleItem (used from the battle menu instead, not the dungeon-view
/// item keys). This is the single source of truth both usable_menu_items
/// and usable_ability_items below split further - using the same
/// underlying list in both keeps them from ever double-counting or
/// missing an item type.
pub fn usable_carried_items<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    let mut items: Vec<Entity> = <(Entity, &Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, carried)| carried.0 == wielder)
        .map(|(e, _, _)| *e)
        .filter(|e| {
            let entry = ecs.entry_ref(*e).unwrap();
            entry.get_component::<Weapon>().is_err() && entry.get_component::<BattleItem>().is_err()
        })
        .collect();

    // Healing Potion first, Dungeon Map second (when carried), everything
    // else after - just a stable, sensible default ordering for the Item
    // Menu list now (see usable_menu_items) rather than a fixed-identity
    // hotkey requirement the way it used to be, back when 1/2 were
    // hardcoded to these specific items.
    items.sort_by_key(|e| item_hotkey_priority(ecs, *e));
    items
}

/// The Class name on an item entity, if it has one - the signal that
/// splits usable_carried_items into the Item Menu's universal
/// consumables (usable_menu_items, no Class at all) versus the Ability
/// Bar's class-restricted abilities (usable_ability_items, Class ==
/// wielder's own class). Every out-of-combat item template already
/// either omits `class:` entirely (Healing Potion, Dungeon Map) or tags
/// it with a specific class (Trap, Throw Spear, Invisible Cloak, ...) -
/// this reuses that existing data rather than needing any new field.
fn item_class<T: EntityStore>(ecs: &T, item: Entity) -> Option<String> {
    ecs.entry_ref(item)
        .ok()
        .and_then(|entry| entry.get_component::<Class>().ok().map(|c| c.0.clone()))
}

/// The Item Menu's contents (press M - see screens/item_menu.rs): every
/// usable_carried_items entry with NO Class restriction at all -
/// Healing Potion, Dungeon Map, and any future item every class can
/// carry. Class-restricted items are deliberately excluded here - see
/// usable_ability_items, their home on the Ability Bar instead.
pub fn usable_menu_items<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    usable_carried_items(ecs, wielder)
        .into_iter()
        .filter(|e| item_class(ecs, *e).is_none())
        .collect()
}

/// The Ability Bar's contents (systems/hud.rs) - every
/// usable_carried_items entry whose Class matches `wielder_class`.
/// Filtered to the wielder's own class defensively, though in practice a
/// carried item's Class should already always match the wielder's -
/// items are only ever granted class-matched in the first place.
pub fn usable_ability_items<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    wielder_class: &str,
) -> Vec<Entity> {
    usable_carried_items(ecs, wielder)
        .into_iter()
        .filter(|e| item_class(ecs, *e).as_deref() == Some(wielder_class))
        .collect()
}

/// Groups a list of item entities by display Name, stacking identical
/// copies (e.g. two Healing Potions become one "Healing Potion" entry
/// with a count of 2) instead of one line/hotkey slot per physical copy.
/// Keeps one representative entity per group (the first copy
/// encountered) - using that group's slot consumes just that one entity,
/// so a stack of 2 correctly becomes a stack of 1 after using one, not
/// both. Shared by the Item Menu (grouping usable_menu_items) and the
/// Ability Bar (grouping usable_ability_items) - previously each had its
/// own near-identical copy of this loop before the item/ability split.
pub fn group_items<T: EntityStore>(ecs: &T, items: Vec<Entity>) -> Vec<(String, i32, Entity)> {
    let mut groups: Vec<(String, i32, Entity)> = Vec::new();
    for item in items {
        let name = match ecs
            .entry_ref(item)
            .ok()
            .and_then(|entry| entry.get_component::<Name>().ok().map(|n| n.0.clone()))
        {
            Some(n) => n,
            None => continue,
        };
        match groups.iter_mut().find(|(existing, _, _)| *existing == name) {
            Some(group) => group.1 += 1,
            None => groups.push((name, 1, item)),
        }
    }
    groups
}

/// The Item Menu's grouped, displayable rows (see screens/item_menu.rs) -
/// usable_menu_items, grouped by name. Since it's a real scrollable/
/// cursor-navigable list now rather than fixed hotkey slots, there's no
/// need to preserve gaps for an uncarried Potion/Map the way the old
/// fixed-identity usable_item_slots had to - group_items' natural
/// "only what's actually carried" output is exactly what a real menu
/// wants.
pub fn usable_item_groups<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<(String, i32, Entity)> {
    group_items(ecs, usable_menu_items(ecs, wielder))
}

/// One Ability Bar slot - see ability_bar_slots. `owned` is None for an
/// ability the class roster includes but the player doesn't currently
/// have any copies of (shown greyed out, per the always-show-the-full-
/// roster convention the battle menu already established for
/// techniques).
#[derive(Clone, Debug, PartialEq)]
pub struct AbilityBarSlot {
    pub name: String,
    /// (count, a representative entity to consume when used) - None if
    /// not currently carried at all.
    pub owned: Option<(i32, Entity)>,
}

/// Matches a fixed `roster` of names against whatever's actually owned
/// (`owned_groups`, from group_items) - the shared "always show the full
/// roster, grey out what's unowned" shape behind ability_bar_slots,
/// battle_bar_slots, and item_bar_slots, pulled out once they'd otherwise
/// be three near-identical copies of this same loop (the same reasoning
/// group_items itself was already extracted for).
fn build_roster_slots(
    roster: &[String],
    owned_groups: &[(String, i32, Entity)],
) -> Vec<AbilityBarSlot> {
    roster
        .iter()
        .map(|name| AbilityBarSlot {
            name: name.clone(),
            owned: owned_groups
                .iter()
                .find(|(n, _, _)| n == name)
                .map(|(_, count, entity)| (*count, *entity)),
        })
        .collect()
}

/// The Ability Bar's full row of slots, in FIXED roster order (from
/// `roster` - see spawner::effect_names_for_class, which reads that
/// order straight from template.ron) - every slot the class could ever
/// have shows up here every time, greyed out (owned: None) when not
/// currently carried, rather than the list compacting around whatever's
/// actually owned right now. This is what makes "press 3" always mean
/// "this class's 3rd roster ability," in battle or out, even during a
/// run where that ability hasn't dropped yet.
pub fn ability_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    wielder_class: &str,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, usable_ability_items(ecs, wielder, wielder_class));
    build_roster_slots(roster, &owned_groups)
}

/// Every BattleItem entity `wielder` currently carries - the Battle Bar's
/// contents (systems/hud.rs's second icon bar, next to the out-of-combat
/// Ability Bar), as opposed to usable_carried_items' EXCLUSION of
/// BattleItem (those are used from the battle menu, not out here).
pub fn battle_items_carried<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    <(Entity, &Item, &BattleItem, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, _, carried)| carried.0 == wielder)
        .map(|(e, _, _, _)| *e)
        .collect()
}

/// The Battle Bar's full row of slots (systems/hud.rs) - same shape and
/// "always show the full roster, grey out what's unowned" convention as
/// ability_bar_slots, but for in-battle Techniques (battle_items_carried)
/// rather than out-of-combat Effects. `roster` should be
/// spawner::class_technique_names(class) - the same roster the battle
/// menu itself already builds its technique list from.
pub fn battle_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, battle_items_carried(ecs, wielder));
    build_roster_slots(roster, &owned_groups)
}

/// The Item Bar's full row of slots (systems/hud.rs's third icon bar,
/// left of the Ability Bar) - same shape and "always show the full
/// roster, grey out what's unowned" convention as ability_bar_slots/
/// battle_bar_slots, but for universal (no `class:` tag) consumables
/// rather than a specific class's abilities. `roster` should be
/// spawner::universal_item_names() - Healing Potion, Dungeon Map, and any
/// future item every class can carry, in template.ron's own file order.
pub fn item_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, usable_menu_items(ecs, wielder));
    build_roster_slots(roster, &owned_groups)
}

/// The Item Menu's "Equipped Items" section (screens/item_menu.rs) - just
/// the currently-carried Weapon for now (armor/trinkets don't exist as
/// components yet - see docs/ideas.md). Unlike ability_bar_slots/
/// battle_bar_slots/item_bar_slots, there's no fixed "roster of every
/// weapon this class could ever equip" to show greyed-out placeholders
/// for - only one weapon is ever carried at a time (auto-pickup discards
/// whatever was carried before), so this is just whatever's actually
/// equipped right now, or empty if nothing is.
pub fn equipped_weapon_slots<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<AbilityBarSlot> {
    <(Entity, &Carried, &Weapon, &Name)>::query()
        .iter(ecs)
        .filter(|(_, carried, _, _)| carried.0 == wielder)
        .map(|(entity, _, _, name)| AbilityBarSlot {
            name: name.0.clone(),
            // The real weapon entity, even though Equipped Items is
            // browse/description-only today (see screens/item_menu.rs)
            // and never actions this via Enter - the real entity costs
            // nothing extra to thread through and is a lot less
            // confusing than a placeholder that would need explaining.
            owned: Some((1, *entity)),
        })
        .collect()
}

/// How many columns of breathing room sit between two adjacent bar groups
/// on ABILITY_BAR_CONSOLE (Item | gap | Ability | gap | Battle) - shared
/// by battle_bar_start_col and item_bar_start_col so the two gaps stay
/// visually identical.
pub const BAR_GROUP_GAP_COLS: i32 = 2;

/// Which ABILITY_BAR_CONSOLE row every bar's icons sit on - one full
/// icon-height above the console's very bottom row, so the bars aren't
/// flush against the physical screen edge. Shared by systems/hud.rs
/// (rendering) and systems/player_input.rs (hit-testing an Item Bar
/// click) - both need the exact same row, not two independently-computed
/// copies that could drift out of sync.
pub fn ability_bar_row() -> i32 {
    ABILITY_BAR_ROWS - 2
}

/// The leftmost column `n` icons should start at to appear centered as a
/// group on ABILITY_BAR_CONSOLE - e.g. a 2-ability class's icons sit
/// centered in the middle of the screen, not pinned to the left edge the
/// way a longer roster's would naturally reach toward anyway. Integer
/// division rounds a genuinely-odd remainder toward the left rather than
/// perfectly splitting a half-column, which isn't expressible on a
/// whole-cell grid regardless. Shared with player_input.rs - see
/// ability_bar_row's own doc comment on why this lives here now instead
/// of only in systems/hud.rs.
pub fn ability_bar_start_col(n: i32) -> i32 {
    (ABILITY_BAR_COLS - n) / 2
}

/// The Battle Bar's own starting column - immediately to the right of the
/// out-of-combat Ability Bar's icons, plus a small gap, so the two boxes
/// read as clearly separate groups rather than touching.
pub fn battle_bar_start_col(ability_bar_start_col: i32, ability_bar_n: i32) -> i32 {
    ability_bar_start_col + ability_bar_n + BAR_GROUP_GAP_COLS
}

/// The Item Bar's own starting column - immediately to the LEFT of the
/// out-of-combat Ability Bar's icons (mirroring battle_bar_start_col's
/// gap on the right), so the row reads as Item | gap | Ability | gap |
/// Battle. Subtracts rather than adds since this group grows leftward
/// from the Ability Bar's own left edge.
pub fn item_bar_start_col(ability_bar_start_col: i32, item_bar_n: i32) -> i32 {
    ability_bar_start_col - BAR_GROUP_GAP_COLS - item_bar_n
}

/// Sort key used by usable_carried_items - see its comment for the slot
/// assignment. Falls back to the "other items" bucket if the entity has
/// no Name for some reason, rather than panicking.
fn item_hotkey_priority<T: EntityStore>(ecs: &T, item: Entity) -> i32 {
    let name = ecs
        .entry_ref(item)
        .ok()
        .and_then(|entry| entry.get_component::<Name>().ok().map(|n| n.0.clone()));
    match name.as_deref() {
        Some("Healing Potion") => 0,
        Some("Dungeon Map") => 1,
        _ => 2,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivateItem {
    pub used_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Damage(pub i32);

/// How quickly this entity acts in battle - higher goes first. Used to
/// decide battle initiative order (see Battle::new / battle_tick).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed(pub i32);

/// How much you will take off the damage of an enemy - Higher means lower damage
/// from an enemy

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Defense(pub i32);

/// Base percent chance to fully evade an incoming attack (0 damage,
/// logged as "Dodge attack.") - checked in battle::resolve_enemy_attack
/// alongside any temporary Evade technique bonus (see Battle::dodge_bonus),
/// which stacks additively on top of this. Currently only Rogue has a
/// nonzero value; other classes default to 0 via entity_evasion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evasion(pub i32);

/// How long a single tile-to-tile glide takes, in milliseconds. Shared by
/// tick_animations (systems/animation.rs, which advances/expires it) and
/// entity_render (which reads elapsed_ms to interpolate the drawn
/// position) so both stay in lockstep. Raised from 150 to 220 alongside
/// the FPS cap going from 30 to 60 (see main()'s BTermBuilder chain) -
/// together these give a glide roughly 3x the frames it had before
/// (~4-5 frames -> ~13), which is what actually fixed the visible
/// jumpiness; either change alone would have helped some, but not as
/// much as both together.
pub const MOVE_ANIM_DURATION_MS: f32 = 220.0;

/// Attached alongside the instant Point update in movement.rs so a
/// creature's *logical* position (and therefore FOV/turn-state/anything
/// else that reads Point) updates immediately, while entity_render draws
/// it sliding from `start` to `end` over MOVE_ANIM_DURATION_MS instead of
/// popping straight to the destination tile. Removed by tick_animations
/// once the glide finishes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingAnimation {
    pub start: Point,
    pub end: Point,
    pub elapsed_ms: f32,
}

/// How many "standing still" frame columns `resources/character_idle.png`
/// has per class row - see CHARACTER_IDLE_COLS, which is defined in terms
/// of this. Currently 6, matching Hunter's real PixelLab-exported Walk
/// cycle (the first class to get real walk-in-place art from that
/// pipeline) - Rogue/Amazon's own (older, different-pipeline) real frames
/// only had 5, so their 6th column is just a duplicate of their own first
/// frame rather than a genuinely distinct pose, and Barbarian/Mage's
/// placeholder rows repeat their single existing dungeon portrait across
/// all 6 regardless. Any Vec length up to this works with zero code
/// changes elsewhere in IdleAnimation itself.
pub const MAX_IDLE_FRAMES: usize = 6;

/// How many idle frames a class/enemy gets by default, and how long each
/// one shows before advancing to the next - see idle_frames_for. Every
/// entity in the game currently gets DEFAULT_IDLE_FRAME_COUNT frames that
/// all point at the exact same glyph as its base Render (see
/// idle_frames_for's own doc comment) - kept below MAX_IDLE_FRAMES so
/// there's room to grow a specific class/enemy up to 5 real, visually
/// distinct frames later without touching this constant.
pub const DEFAULT_IDLE_FRAME_COUNT: usize = 3;
pub const IDLE_FRAME_DURATION_MS: f32 = 350.0;

/// Which sprite sheet/console an IdleAnimation's `frames` glyphs are cells
/// in - see systems/entity_render.rs, which needs this to route each
/// entity's draw call to the matching console (CHARACTER_IDLE_CONSOLE's
/// trio for `CharacterIdle`, ENEMY_IDLE_CONSOLE's trio for `EnemyIdle`,
/// the plain dungeonfont ones for `Dungeon`).
/// Deliberately a field on the existing IdleAnimation component rather
/// than a new separate marker component - a new component would need its
/// own `#[read_component]` declaration added everywhere IdleAnimation is
/// already queried, exactly the class of legion access-panic this project
/// has been bitten by before (see CLAUDE.md); a new field on an
/// already-declared component needs no new declarations anywhere.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleSpriteSheet {
    Dungeon,
    CharacterIdle,
    EnemyIdle,
}

/// The "walking in place" idle loop: a small set of glyphs a stationary
/// entity cycles through, advancing one frame every IDLE_FRAME_DURATION_MS
/// (see systems/animation.rs's tick_idle_animation) and wrapping back to
/// frame 0 after the last. `sheet` says which console's font `frames`
/// indexes into (see IdleSpriteSheet) - both fields' actual values come
/// from whichever of idle_frames_for/idle_frames_for_class built this.
/// Deliberately only advances while the entity is NOT in an in-flight
/// MovingAnimation (see gliding_position) - real movement already has its
/// own glide animation, and cycling frames underneath that too would just
/// be visual noise on top of it. tick_idle_animation still exists and
/// runs on a moving entity, it just doesn't advance elapsed_ms for it
/// that frame, so an interrupted glide always resumes idling from
/// whichever frame it left off on rather than losing its place.
#[derive(Clone, Debug, PartialEq)]
pub struct IdleAnimation {
    pub frames: Vec<FontCharType>,
    pub frame_index: usize,
    pub elapsed_ms: f32,
    pub sheet: IdleSpriteSheet,
}

impl IdleAnimation {
    /// The glyph this animation is currently showing.
    pub fn current_glyph(&self) -> FontCharType {
        self.frames
            .get(self.frame_index)
            .copied()
            .unwrap_or_default()
    }
}

/// Builds a fresh, dungeonfont-based placeholder IdleAnimation for a
/// creature whose base dungeon-view glyph is `base_glyph` - called for
/// every Enemy (spawner/template.rs's spawn_entity, which has no class to
/// look up a real sheet row for) and as the fallback for any player class
/// idle_frames_for_class doesn't recognize. Every frame is the SAME glyph
/// as the entity's base Render - a deliberate placeholder (the animation
/// is frame-complete and genuinely cycling under the hood, it just has no
/// visible effect) until real per-enemy walk-cycle art exists.
pub fn idle_frames_for(base_glyph: FontCharType) -> IdleAnimation {
    IdleAnimation {
        frames: vec![base_glyph; DEFAULT_IDLE_FRAME_COUNT],
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::Dungeon,
    }
}

/// How many idle-loop frame columns `resources/character_idle.png` has
/// per class row - see that file's own doc comment in main.rs
/// (CHARACTER_IDLE_CONSOLE) for the full sheet layout, and
/// MAX_IDLE_FRAMES's own doc comment for why this is 6 rather than every
/// class actually having 6 genuinely distinct frames.
pub const CHARACTER_IDLE_COLS: u16 = MAX_IDLE_FRAMES as u16;

/// Which row a class occupies on `resources/character_portrait.png` -
/// `None` for anything without a row there. Safe to reuse a plain
/// 0-based row assignment here (unlike character_idle_row/
/// character_battle_row below) because character_portrait_glyph ONLY
/// EVER populates column 0 of a class's row - a plain console's `cls()`
/// default glyph (32) lands at column 32 % CHARACTER_IDLE_COLS == 2 on
/// this 6-column sheet, and column 2 is guaranteed blank on every row
/// regardless of which row that is, so there's no row this sheet
/// specifically needs to keep empty. (character_idle.png doesn't have
/// this luxury - it fills every column of a class's row with a real
/// walk-cycle frame - which is exactly why it needs its own
/// character_idle_row instead of sharing this one.)
pub fn class_sheet_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        "Hunter" => Some(3),
        "Mage" => Some(4),
        // Debug is the hidden dev/test class (see title.rs::class_select's
        // 'D' hotkey) - not in CLASS_ROSTER, but still a real Player
        // entity that needs real idle/portrait art like any other class.
        "Debug" => Some(5),
        _ => None,
    }
}

/// The exact `resources/character_portrait.png` glyph for `class`'s
/// single static still portrait - `None` if `class` has no row there
/// (see class_sheet_row). Column 0 only (this sheet only ever holds one
/// pose per class - PixelLab's own `rotations/south.png`).
pub fn character_portrait_glyph(class: &str) -> Option<FontCharType> {
    let row = class_sheet_row(class)?;
    Some(row * CHARACTER_IDLE_COLS)
}

/// Which row a class occupies on `resources/character_idle.png`
/// SPECIFICALLY - deliberately NOT class_sheet_row, for the identical
/// reason character_battle_row below needs its own mapping: a plain
/// console's `cls()` fills every never-drawn-this-frame cell with glyph
/// 32 by default, and this sheet's own column count (CHARACTER_IDLE_COLS,
/// 6) puts that at row 5, column 2 (32 / 6 == 5, 32 % 6 == 2). Unlike
/// character_portrait_glyph above, this sheet fills EVERY column of a
/// class's row with a real walk-cycle frame, so column 2 is never
/// guaranteed blank - row 5 has to stay permanently unassigned here, the
/// same way character_battle_row permanently skips row 4. Confirmed for
/// real, not just reasoned: assigning Debug to row 5 here (the hidden
/// dev/test class, added after Barbarian/Rogue/Amazon/Hunter/Mage had
/// already safely filled rows 0-4) reproduced the exact Mage-tiling bug
/// on this sheet instead of character_battle.png - the entire title/
/// adventure-select screen filled with tiled Robot portraits, since
/// CHARACTER_IDLE_CONSOLE spans the full display and was never NOT
/// showing that leaked content. The single source of truth shared by
/// idle_frames_for_class (builds a full IdleAnimation up front, at spawn
/// time) and the Class Select screen's highlighted-class preview
/// (screens/title.rs::class_select, which looks up one frame at a time
/// off its own menu timer instead of a real IdleAnimation component,
/// since a roster entry there isn't a real ECS entity) - both stay in
/// sync automatically if a class's row on THIS sheet ever moves.
fn character_idle_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        "Hunter" => Some(3),
        "Mage" => Some(4),
        // Row 5 deliberately skipped - see this fn's own doc comment.
        "Debug" => Some(6),
        _ => None,
    }
}

/// The exact `resources/character_idle.png` glyph for `class`'s idle
/// frame `frame_index` (wrapped modulo CHARACTER_IDLE_COLS, so any
/// ever-increasing counter can be passed directly) - `None` if `class`
/// has no row on that sheet (see character_idle_row).
pub fn character_idle_glyph(class: &str, frame_index: usize) -> Option<FontCharType> {
    let row = character_idle_row(class)?;
    let col = (frame_index as u16) % CHARACTER_IDLE_COLS;
    Some(row * CHARACTER_IDLE_COLS + col)
}

/// How many battle-idle frame columns `resources/character_battle.png`
/// has per class row - 8, matching Hunter's real PixelLab-exported
/// Fight_Stance_Idle/east cycle (see main.rs's CHARACTER_BATTLE_CONSOLE
/// for the full sheet layout).
pub const CHARACTER_BATTLE_COLS: u16 = 8;

/// Which row a class occupies on `resources/character_battle.png`
/// SPECIFICALLY - deliberately its own mapping, not shared with
/// character_idle_row or class_sheet_row, and this is load-bearing, not
/// a stylistic choice: a console's `cls()` fills every never-drawn-this-
/// frame cell with glyph 32 by default (see CLAUDE.md's standing
/// gotchas), and which ROW that lands on depends on THIS sheet's own
/// column count (32 / 8 = 4 exactly) - independent of how many columns
/// any OTHER sheet has. A naive plain 0..4 assignment would put row 4 at
/// Mage, and Mage's frames would silently replace every undrawn cell
/// across the WHOLE console (which spans the full display) on every
/// screen, all the time - confirmed for real: the entire title screen
/// filled with tiled Mage portraits before this was caught. Row 4 is
/// left deliberately blank here and Mage moved to row 5 instead. (The
/// exact same class of bug hit character_idle.png too, once Debug's row
/// landed on ITS sheet's own forbidden row 5 - see character_idle_row's
/// own doc comment for that second confirmed occurrence. Every
/// per-class sheet needs this check done fresh for its own column count,
/// never assumed safe by analogy with another sheet.)
fn character_battle_row(class: &str) -> Option<u16> {
    match class {
        "Barbarian" => Some(0),
        "Rogue" => Some(1),
        "Amazon" => Some(2),
        "Hunter" => Some(3),
        // Row 4 deliberately skipped - see this fn's own doc comment.
        "Mage" => Some(5),
        "Debug" => Some(6),
        _ => None,
    }
}

/// The exact `resources/character_battle.png` glyph for `class`'s
/// battle-idle frame `frame_index` (wrapped modulo CHARACTER_BATTLE_COLS)
/// - `None` if `class` has no row on that sheet (see
/// character_battle_row). Used by the battle screen's own portrait loop,
/// NOT IdleAnimation - the player's battle portrait isn't a dungeon-view
/// entity, so this is driven by a plain frame counter on `Battle` itself
/// instead (see Battle::player_idle_frame).
pub fn character_battle_glyph(class: &str, frame_index: usize) -> Option<FontCharType> {
    let row = character_battle_row(class)?;
    let col = (frame_index as u16) % CHARACTER_BATTLE_COLS;
    Some(row * CHARACTER_BATTLE_COLS + col)
}

/// Builds a fresh IdleAnimation for a player class, pulling real frames
/// from `resources/character_idle.png` when `class` has a row there (see
/// character_idle_row - every current class does, including the hidden
/// Debug one). Falls back to the plain dungeonfont placeholder
/// (idle_frames_for) for anything without a row there - a future class
/// added without art yet.
pub fn idle_frames_for_class(class: &str, base_glyph: FontCharType) -> IdleAnimation {
    let row = match character_idle_row(class) {
        Some(r) => r,
        None => return idle_frames_for(base_glyph),
    };
    let frames = (0..CHARACTER_IDLE_COLS)
        .map(|col| row * CHARACTER_IDLE_COLS + col)
        .collect();
    IdleAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::CharacterIdle,
    }
}

/// How many idle-loop frame columns `resources/enemy_idle.png` has per
/// enemy row - see CHARACTER_IDLE_COLS's own doc comment; this sheet
/// follows the identical "one row per <thing>, MAX_IDLE_FRAMES columns"
/// convention, just for enemies on their own dedicated sheet instead of
/// playable classes (see docs/ideas.md's "Add PixelLab art for enemies"
/// backlog item, and the design conversation that preceded it: enemies
/// get their own sheets rather than more rows on the class sheets, since
/// those only have one free row left and enemies are a different lookup
/// domain entirely - keyed by name, not class).
pub const ENEMY_IDLE_COLS: u16 = MAX_IDLE_FRAMES as u16;

/// Which row an enemy occupies on `resources/enemy_idle.png`
/// SPECIFICALLY - its own dedicated mapping, not shared with
/// character_idle_row, for the identical reason every per-sheet row
/// function in this file is its own: a plain console's `cls()` fills
/// every never-drawn-this-frame cell with glyph 32 by default, and which
/// (row, col) that lands on depends on THIS sheet's own column count.
/// enemy_idle.png happens to share character_idle.png's 6-column layout,
/// so the forbidden row is the same number (32 / 6 == 5) - a coincidence
/// of matching column counts, not a reason to ever look this up via
/// character_idle_row instead.
fn enemy_idle_row(name: &str) -> Option<u16> {
    match name {
        "Goblin" => Some(0),
        "Orc" => Some(1),
        "Ogre" => Some(2),
        "Ettin" => Some(3),
        "Goblin Chieftain" => Some(4),
        // Row 5 deliberately skipped - see this fn's own doc comment.
        // "Orc Warlord" was held back here through 2026-09-08's first
        // batch (a genuine PixelLab generation defect - a thin off-model
        // sliver instead of a full character, on both Walk and
        // Fight_Stance_Idle) - the redo batch (same day) came back
        // clean, confirmed by screenshot. Its Walk/south came back with
        // 8 frames, more than this sheet's own 6-column ceiling
        // (MAX_IDLE_FRAMES) allows - 6 of the 8 were evenly sampled
        // (indices 0,1,3,4,6,7) rather than just truncated, so the walk
        // cycle doesn't visibly skip its back half.
        "Orc Warlord" => Some(6),
        "Ogre Warlord" => Some(7),
        "Ettin Overlord" => Some(8),
        _ => None,
    }
}

/// How many battle-idle frame columns `resources/enemy_battle.png` has
/// per enemy row - see CHARACTER_BATTLE_COLS's own doc comment; same
/// convention, enemy-specific sheet.
pub const ENEMY_BATTLE_COLS: u16 = 8;

/// Which row an enemy occupies on `resources/enemy_battle.png`
/// SPECIFICALLY - see character_battle_row's own doc comment for why this
/// needs its own mapping, never shared across sheets. This sheet also
/// happens to share character_battle.png's 8-column layout, so its own
/// forbidden row is also 4 (32 / 8 == 4) - independently re-derived here,
/// not assumed safe from that coincidence.
fn enemy_battle_row(name: &str) -> Option<u16> {
    match name {
        "Goblin" => Some(0),
        "Orc" => Some(1),
        "Ogre" => Some(2),
        "Ettin" => Some(3),
        // Row 4 deliberately skipped - see this fn's own doc comment.
        "Goblin Chieftain" => Some(5),
        // "Orc Warlord" - see enemy_idle_row's own doc comment for the
        // redo/defect history. Its Fight_Stance_Idle/south-west came
        // back with exactly 8 frames, matching this sheet's own column
        // count - no sampling needed here, unlike the idle sheet.
        "Orc Warlord" => Some(6),
        "Ogre Warlord" => Some(7),
        "Ettin Overlord" => Some(8),
        _ => None,
    }
}

/// The exact `resources/enemy_battle.png` glyph for enemy `name`'s
/// battle-idle frame `frame_index` (wrapped modulo ENEMY_BATTLE_COLS) -
/// `None` if `name` has no row on that sheet (see enemy_battle_row). Used
/// by the battle screen's own per-enemy portrait loop (screens/battle.rs)
/// the same way character_battle_glyph drives the player's.
pub fn enemy_battle_glyph(name: &str, frame_index: usize) -> Option<FontCharType> {
    let row = enemy_battle_row(name)?;
    let col = (frame_index as u16) % ENEMY_BATTLE_COLS;
    Some(row * ENEMY_BATTLE_COLS + col)
}

/// Builds a fresh IdleAnimation for an enemy, pulling real walk-cycle
/// frames from `resources/enemy_idle.png` when `name` has a row there
/// (see enemy_idle_row) - falls back to the plain dungeonfont placeholder
/// (idle_frames_for) for any enemy without real art yet, the same
/// fallback idle_frames_for_class uses for an unrecognized class.
pub fn idle_frames_for_enemy(name: &str, base_glyph: FontCharType) -> IdleAnimation {
    let row = match enemy_idle_row(name) {
        Some(r) => r,
        None => return idle_frames_for(base_glyph),
    };
    let frames = (0..ENEMY_IDLE_COLS)
        .map(|col| row * ENEMY_IDLE_COLS + col)
        .collect();
    IdleAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        sheet: IdleSpriteSheet::EnemyIdle,
    }
}

/// A non-looping animation: plays through `frames` once, then holds on
/// the last one - the shape Death, Victory, and a single-hit technique's
/// own battle animation share, as opposed to `IdleAnimation`'s permanent
/// loop. Deliberately its own type rather than a flag on `IdleAnimation`
/// - those two only ever need the opposite behavior from each other,
/// and every existing `IdleAnimation` call site would need to start
/// handling a "hold at the end" case it never actually hits.
///
/// `repeat` (added 2026-09-08, explicit user feedback after seeing
/// Flurry's animation in a real fight) makes this loop back to frame 0
/// instead of holding once it reaches the end - for a multi-hit/AOE
/// technique, whose HitQueue keeps landing damage over a real span of
/// time far longer than one play-through, a single strike pose held
/// motionless for most of that span read as broken/frozen rather than an
/// ongoing flurry of hits. `frame_duration_ms` is baked in per-animation
/// rather than a shared global constant like `IdleAnimation`'s
/// `IDLE_FRAME_DURATION_MS`, since a technique needs a much faster,
/// punchier pace than a breathing idle stance or a held Death/Victory
/// pose - see `TECHNIQUE_FRAME_DURATION_MS`.
#[derive(Clone, Debug, PartialEq)]
pub struct OneShotAnimation {
    pub frames: Vec<FontCharType>,
    pub frame_index: usize,
    pub elapsed_ms: f32,
    pub frame_duration_ms: f32,
    pub repeat: bool,
}

impl OneShotAnimation {
    pub fn current_glyph(&self) -> FontCharType {
        self.frames
            .get(self.frame_index)
            .copied()
            .unwrap_or_default()
    }

    /// True once this animation has reached its last frame and has
    /// nothing left to advance to - always false for a `repeat` one,
    /// which by definition never reaches a permanent end.
    pub fn finished(&self) -> bool {
        !self.repeat && self.frame_index + 1 >= self.frames.len()
    }

    /// Advances by `dt_ms` of real time, at this animation's own
    /// `frame_duration_ms` pace - a no-op once `finished()`, so the
    /// caller never has to check that separately before ticking. A
    /// `repeat` animation wraps back to frame 0 instead of stopping.
    pub fn tick(&mut self, dt_ms: f32) {
        if self.finished() {
            return;
        }
        self.elapsed_ms += dt_ms;
        if self.elapsed_ms >= self.frame_duration_ms {
            self.elapsed_ms -= self.frame_duration_ms;
            self.frame_index += 1;
            if self.repeat && self.frame_index >= self.frames.len() {
                self.frame_index = 0;
            }
        }
    }
}

/// Columns on `resources/character_death.png` / `character_victory.png` /
/// `character_technique.png` - all three happen to share this column
/// count because Rogue's own Death/Victory/Flurry exports all came back
/// with exactly 9 frames; a future class/technique with more frames
/// would need this bumped (and the sheets rebuilt wider) the same way
/// any other sheet's column count has grown before.
pub const EXTRA_ANIM_COLS: u16 = 9;

/// Real ms each frame of a technique animation holds before advancing -
/// see `OneShotAnimation`'s own doc comment for why this needs its own,
/// much faster pace than `IDLE_FRAME_DURATION_MS` (350ms): at that pace,
/// a 9-frame technique animation like Flurry's would only get through
/// ~3 frames before `RESULT_AUTO_ADVANCE_MS` (1100ms) auto-dismisses a
/// single-hit ActionResult - an attack needs to read as fast and punchy,
/// not like a slow held pose.
pub const TECHNIQUE_FRAME_DURATION_MS: f32 = 80.0;

/// Which row a class occupies on `resources/character_death.png` -
/// `None` for a class without one yet, same "grow as art arrives"
/// shape as every other per-class sheet. Row 3 is this sheet's own
/// forbidden row (32 / 9 == 3, see the glyph-32 gotcha in CLAUDE.md) -
/// skipped permanently, independent of character_idle_row/
/// character_battle_row's own forbidden rows on their different
/// column counts.
fn character_death_row(class: &str) -> Option<u16> {
    match class {
        "Rogue" => Some(0),
        "Debug" => Some(1),
        "Hunter" => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        "Barbarian" => Some(4),
        "Amazon" => Some(5),
        "Mage" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s death sequence -
/// `None` if `class` has no row yet (caller keeps whatever fallback it
/// already had, e.g. the rotated-glyph approach).
pub fn death_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_death_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a class occupies on `resources/character_victory.png` -
/// same shape/forbidden-row (3) as `character_death_row`, own dedicated
/// mapping since this is a different sheet.
fn character_victory_row(class: &str) -> Option<u16> {
    match class {
        "Rogue" => Some(0),
        "Debug" => Some(1),
        "Hunter" => Some(2),
        // Row 3 deliberately skipped - see character_death_row's own
        // doc comment for why (identical reasoning, this sheet's own
        // column count).
        "Barbarian" => Some(4),
        "Amazon" => Some(5),
        "Mage" => Some(6),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s victory pose -
/// `None` if `class` has no row yet.
pub fn victory_animation_for_class(class: &str) -> Option<OneShotAnimation> {
    let row = character_victory_row(class)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: IDLE_FRAME_DURATION_MS,
        repeat: false,
    })
}

/// Which row a (class, technique name) pair occupies on
/// `resources/character_technique.png` - keyed by a COMPOUND identity
/// rather than by class alone, since a class can end up with several of
/// these over time ("assume we will add a lot more battle animations for
/// each class" - 2026-09-08). Adding the next one is one more match arm
/// with the next free row, same shape as every other per-thing row
/// function in this file - row 3 is this sheet's own forbidden row
/// (32 / 9 == 3), skipped permanently.
fn technique_animation_row(class: &str, technique: &str) -> Option<u16> {
    match (class, technique) {
        ("Rogue", "Flurry") => Some(0),
        ("Hunter", "Arrow Volley") => Some(1),
        ("Barbarian", "Whirlwind") => Some(2),
        // Row 3 deliberately skipped - see this fn's own doc comment.
        // Amazon's own PixelLab batch named this animation "Spear_Volley"
        // (after the class's weapon) - the real item name in
        // template.ron is "Javelin Volley", which is the string
        // resolve_player_action actually looks this row up by (see
        // entity_name), so that's the name matched here, not the zip's
        // own folder name.
        ("Amazon", "Javelin Volley") => Some(4),
        ("Mage", "Blizzard") => Some(5),
        _ => None,
    }
}

/// Builds a fresh `OneShotAnimation` playing `class`'s own animation for
/// `technique` - `None` if that specific (class, technique) pair has no
/// row yet, in which case the caller keeps showing the ordinary
/// Fight_Stance_Idle loop instead (see `Battle::player_technique_
/// animation`'s own doc comment). `repeat` should be true for a multi-
/// hit/AOE technique (see `OneShotAnimation::repeat`'s own doc comment)
/// - the caller decides this from the item's own `TechniqueEffect`
/// (`battle::technique_effect`) before calling, since that's the only
/// place that already knows whether this specific use is single-hit or
/// not.
pub fn technique_animation_for(
    class: &str,
    technique: &str,
    repeat: bool,
) -> Option<OneShotAnimation> {
    let row = technique_animation_row(class, technique)?;
    let frames = (0..EXTRA_ANIM_COLS)
        .map(|col| row * EXTRA_ANIM_COLS + col)
        .collect();
    Some(OneShotAnimation {
        frames,
        frame_index: 0,
        elapsed_ms: 0.0,
        frame_duration_ms: TECHNIQUE_FRAME_DURATION_MS,
        repeat,
    })
}

/// Wall-clock milliseconds since the last frame (see BTerm::frame_time_ms),
/// inserted as a resource every tick so animation systems advance at a
/// consistent real-world speed regardless of the current frame rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameTime(pub f32);

/// Standard ease-out cubic: fast start, gentle settle into the
/// destination tile rather than a linear, slightly mechanical glide.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// If `entity` currently has an in-flight MovingAnimation, returns its
/// eased fractional (x, y) for *this* frame - still in logical/world
/// coordinates, before the camera offset entity_render applies. Returns
/// None once the glide has finished (elapsed_ms has caught up to
/// MOVE_ANIM_DURATION_MS) or if the entity was never moving, so callers
/// know to fall back to drawing the entity's plain integer Point instead.
///
/// Deliberately returns a raw (f32, f32) tuple rather than a PointF -
/// this codebase has only ever *constructed* a PointF (see
/// draw_end_screen_fallen_portrait), never read x/y back off one, so
/// there's no proof here that PointF exposes public fields the way Point
/// does. Building the tuple ourselves and letting the caller make the
/// final PointF (after subtracting its own camera offset) avoids leaning
/// on that unconfirmed API surface entirely.
///
/// Does NOT consume/remove the animation - that stays tick_animations'
/// job (systems/animation.rs), so "when does a glide end" is only ever
/// decided in one place.
pub fn gliding_position(ecs: &SubWorld, entity: Entity) -> Option<(f32, f32)> {
    let entry = ecs.entry_ref(entity).ok()?;
    let anim = entry.get_component::<MovingAnimation>().ok()?;
    if anim.elapsed_ms >= MOVE_ANIM_DURATION_MS {
        return None;
    }
    let t = ease_out_cubic((anim.elapsed_ms / MOVE_ANIM_DURATION_MS).min(1.0));
    Some((
        lerp(anim.start.x as f32, anim.end.x as f32, t),
        lerp(anim.start.y as f32, anim.end.y as f32, t),
    ))
}

/// Returns the fractional world-space point map_render/entity_render
/// should currently treat as "camera center" for actual DRAWING - as
/// opposed to Camera's own left_x/top_y/right_x/bottom_y, which
/// Camera::on_player_move snaps to the player's destination tile the
/// instant a move is committed (see systems/movement.rs) and which
/// drive what world region counts as "in view" for tile iteration and
/// FOV, not how any of it lands on screen.
///
/// While the player's own MovingAnimation is in flight, this reuses its
/// eased in-between position (see gliding_position above) - the exact
/// same data entity_render already reads for any OTHER gliding entity -
/// minus half the display, so the screen visibly pans from the old
/// center to the new one over MOVE_ANIM_DURATION_MS instead of
/// snapping. See MAP_SCROLL_CONSOLE/ENTITY_SCROLL_CONSOLE in main.rs
/// for the two fancy consoles this drives. Returns None once the
/// player isn't animating (including "never has" and "glide already
/// expired"), telling callers to fall back to Camera's own integer
/// left_x/top_y - the cheap, by-far-more-common path, taken every frame
/// the player isn't actively mid-step.
///
/// Deliberately recomputed fresh from the ECS on every call rather than
/// cached on Camera and refreshed by some dedicated per-frame system: a
/// cached field is only ever as fresh as whatever schedule last wrote
/// it, and not every schedule that calls map_render runs the same
/// systems ahead of it (build_pause_scheduler, for one, runs map_render
/// completely alone). Recomputing here means there's no stale-value
/// case to reason about - whatever this returns is true for the exact
/// instant it's called, in any schedule, always.
///
/// Returns a raw (f32, f32) tuple rather than a PointF, for the same
/// reason gliding_position does above: nothing in this codebase has
/// ever read x/y fields back off a PointF, so there's no confirmed way
/// to subtract one from a Point/(i32,i32) pair. Callers build the final
/// PointF themselves after doing that subtraction in plain f32 math.
///
/// Interpolates between `Camera::clamped_top_left` of the glide's start
/// and end tile (the same clamp `Camera::new`/`on_player_move` apply),
/// rather than the player's own eased position minus a constant half-
/// window offset - the two only agree when neither endpoint is close
/// enough to a map edge for the clamp to actually do anything. Near an
/// edge, using the player's raw position would visibly disagree with
/// where the discrete camera actually lands the instant this glide
/// commits (see `Camera::clamped_top_left`'s own doc comment); lerping
/// the two ALREADY-clamped corners instead means this always agrees with
/// the real camera, whether the clamp is active for the whole step, only
/// part of it (the step that first reaches an edge), or not at all.
pub fn camera_render_offset(ecs: &SubWorld) -> Option<(f32, f32)> {
    let mut player = <(Entity, &Point)>::query().filter(component::<Player>());
    let player_entity = player.iter(ecs).nth(0).map(|(e, _)| *e)?;
    let entry = ecs.entry_ref(player_entity).ok()?;
    let anim = entry.get_component::<MovingAnimation>().ok()?;
    if anim.elapsed_ms >= MOVE_ANIM_DURATION_MS {
        return None;
    }
    let t = ease_out_cubic((anim.elapsed_ms / MOVE_ANIM_DURATION_MS).min(1.0));
    let (start_left, start_top) = Camera::clamped_top_left(anim.start);
    let (end_left, end_top) = Camera::clamped_top_left(anim.end);
    Some((
        lerp(start_left as f32, end_left as f32, t),
        lerp(start_top as f32, end_top as f32, t),
    ))
}

/// Which sprite sheet/console a tile_render_at glyph is a cell in - the
/// tile-rendering equivalent of IdleSpriteSheet, needed for the same
/// reason: map_render.rs has to route each tile's draw call to whichever
/// console actually holds that glyph's font (MAP_TILE_CONSOLE's trio for
/// `MapTiles`, the plain dungeonfont ones for `Dungeon`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileSpriteSheet {
    Dungeon,
    MapTiles,
}

/// Flat brightness multiplier applied to real-texture WALL tiles only
/// (see tile_render_at) - confirmed needed in real play, 2026-09-08:
/// wall and floor variants at the same brightness read as too visually
/// similar for some themes (Sewer specifically). Tune this if a future
/// theme still reads too flat even with it applied.
const WALL_TEXTURE_SHADE: f32 = 0.72;

/// The exact `resources/map_tiles.png` glyph for a Floor/Wall tile,
/// given its theme's starting row (`MapTheme::tile_row`) and its own
/// rolled variant (`Map::tile_variant`) - `None` for any TileType
/// without a variant pool yet (Exit/Counter/Water), so the caller falls
/// through to the old dungeonfont rendering for those. See
/// docs/Map_Tile_Theme_Guide.md for the row layout this encodes: row
/// `base_row + 0` is basic floor, `+1` wall, `+2` themed floor (floor's
/// variant pool spans both `+0` and `+2`, `variant` 0..FLOOR_VARIANT_COUNT
/// picks between them), `+3` special wall (not wired up yet).
fn map_tile_glyph(base_row: u16, tile: TileType, variant: u8) -> Option<FontCharType> {
    let (row_offset, col) = match tile {
        TileType::Floor if variant < MAP_TILE_COLS as u8 => (0, variant as u16),
        TileType::Floor => (2, (variant - MAP_TILE_COLS as u8) as u16),
        TileType::Wall => (1, variant as u16),
        TileType::Exit | TileType::Counter | TileType::Water => return None,
    };
    Some((base_row + row_offset) * MAP_TILE_COLS + col)
}

/// Computes the same ColorPair/glyph/sheet a tile would be drawn with in
/// map_render.rs, for a single point - shared so entity_render can paint
/// the real floor/wall tile underneath a mid-glide entity (see
/// systems/entity_render.rs) instead of duplicating this logic, and so
/// the two never drift apart. Returns None if the tile is out of bounds
/// or has never been seen (nothing should be drawn there).
pub fn tile_render_at(
    map: &Map,
    theme: &dyn MapTheme,
    visible_tiles: &HashSet<Point>,
    pt: Point,
) -> Option<(ColorPair, FontCharType, TileSpriteSheet)> {
    if !map.in_bounds(pt) {
        return None;
    }
    let idx = map_idx(pt.x, pt.y);
    if !(visible_tiles.contains(&pt) || map.revealed_tiles[idx]) {
        return None;
    }
    let visible = visible_tiles.contains(&pt);
    let tile = map.tiles[idx];

    // Real per-tile texture path - see map_tile_glyph's own doc comment
    // for which TileTypes actually have art yet. Every real tile texture
    // is fully opaque, so "color" is mostly just the same visible/
    // remembered brightness multiply Floor/Exit's old plain path already
    // used (WHITE = unchanged, DARK_GRAY = dimmed). Walls additionally
    // get a flat darkening multiply on top of that (WALL_TEXTURE_SHADE) -
    // confirmed needed in real play (2026-09-08, Sewer specifically):
    // without any per-type tint, wall and floor variants painted at the
    // same brightness read as too visually similar when the art itself
    // doesn't have enough inherent contrast, the same "hard to tell
    // floor from wall" complaint the patch/accent generation algorithm
    // was built to fix on the LAYOUT side - this is the color-side half
    // of the same problem. Applied as a post-multiply here rather than
    // baked into the art, so it benefits every theme uniformly (not just
    // the one that surfaced it) without needing new art. Safe against
    // the `_no_bg` near-black cutoff (MAP_TILE_CONSOLE's own gotcha,
    // above) because that check runs on the source texture's own raw
    // color, before this multiply ever applies.
    if let Some(base_row) = theme.tile_row() {
        if let Some(glyph) = map_tile_glyph(base_row, tile, map.tile_variant[idx]) {
            let (wr, wg, wb) = if visible { WHITE } else { DARK_GRAY };
            let base_tint = RGB::from_u8(wr, wg, wb);
            let tint = if tile == TileType::Wall {
                RGB::from_f32(
                    base_tint.r * WALL_TEXTURE_SHADE,
                    base_tint.g * WALL_TEXTURE_SHADE,
                    base_tint.b * WALL_TEXTURE_SHADE,
                )
            } else {
                base_tint
            };
            return Some((ColorPair::new(tint, BLACK), glyph, TileSpriteSheet::MapTiles));
        }
    }

    let glyph = theme.tile_to_render(tile);
    let wall_base = theme.wall_color();

    let color_pair = if tile == TileType::Wall {
        let bg = if visible {
            wall_base
        } else {
            RGB::from_f32(wall_base.r * 0.35, wall_base.g * 0.35, wall_base.b * 0.35)
        };
        let fg = RGB::from_f32(
            (bg.r * 1.4).min(1.0),
            (bg.g * 1.4).min(1.0),
            (bg.b * 1.4).min(1.0),
        );
        ColorPair::new(fg, bg)
    } else if tile == TileType::Counter {
        // The Battle Arena shop's counter - a warm red bar, deliberately
        // distinct from both the theme's wall and floor colors, so shop
        // items sitting on it read as "on a counter" rather than "text
        // embedded in a generic brick wall" (the counter's first version
        // just reused TileType::Wall for this, which looked like the
        // latter).
        let bright = RGB::from_f32(0.55, 0.12, 0.12);
        let fg = if visible {
            bright
        } else {
            RGB::from_f32(bright.r * 0.35, bright.g * 0.35, bright.b * 0.35)
        };
        ColorPair::new(fg, BLACK)
    } else {
        let tint = if visible { WHITE } else { DARK_GRAY };
        ColorPair::new(tint, BLACK)
    };

    Some((color_pair, glyph, TileSpriteSheet::Dungeon))
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldOfView {
    pub visible_tiles: HashSet<Point>,
    pub radius: i32,
    pub is_dirty: bool,
}

impl FieldOfView {
    pub fn new(radius: i32) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius,
            is_dirty: true,
        }
    }

    pub fn clone_dirty(&self) -> Self {
        Self {
            visible_tiles: HashSet::new(),
            radius: self.radius,
            is_dirty: true,
        }
    }
}
