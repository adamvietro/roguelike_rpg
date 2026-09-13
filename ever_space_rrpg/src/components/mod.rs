pub use crate::prelude::*;
use std::collections::HashSet;

mod animation;
mod bars;
mod glide;
mod tiles;
pub use animation::*;
pub use bars::*;
pub use glide::*;
pub use tiles::*;

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

/// An out-of-combat item's effect, applied when used from the dungeon-view
/// item list (see systems/use_items.rs) - set from `Template.effect` in
/// template.ron.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
// --- Out-of-combat item effects ---------------------------------------------
//
// One component + data enum, mirroring Technique/TechniqueEffect below -
// replaces the old approach of a separate marker component per effect
// (ProvidesHealing, ProvidesDungeonMap, ProvidesInvisibility) with one
// generic Effect component whose meaning is data, not a distinct Rust
// type. An item has at most one such effect.

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
    /// Debug-class item: instantly starts a real Battle against 4 fresh
    /// Goblins, spawned right next to the user - added specifically so
    /// the multi-enemy battle-screen formation (enemy_portrait_position,
    /// screens/battle.rs) can be tested on demand instead of needing to
    /// find/herd a real 4-enemy encounter in normal play. See
    /// systems/use_items.rs.
    DebugBattle4,
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

/// One battle technique's mechanical effect. Attached to an Item entity via
/// `Technique`, set from `Template.technique` in template.ron - see
/// spawner/template.rs. Interpreted in one place: battle::apply_player_technique.
#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize)]
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

/// The player entity and its current Point, or `None` if no entity
/// carries `Player` (shouldn't happen in practice, but every caller
/// already treated it as a possibility rather than unwrapping blind).
/// A real bug once came from a hand-rolled query missing exactly this
/// `.filter(component::<Player>())` - it silently matched a different,
/// Point-tagged entity instead (a shop's Shopkeeper/ShopStock marker),
/// depending on legion's internal archetype iteration order, and only
/// surfaced once a different entity-creation order exposed it (see
/// `player_input.rs`'s own `buy_nearby_item` history). Centralizing the
/// query here makes that whole class of mistake structurally impossible
/// in new code, not just fixed at the one call site it was found.
pub fn find_player<T: EntityStore>(ecs: &T) -> Option<(Entity, Point)> {
    <(Entity, &Point)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .map(|(entity, pos)| (*entity, *pos))
}

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

