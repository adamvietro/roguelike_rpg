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

/// The mouse position captured directly in the HUD console's own
/// coordinate space (console 4 was active at capture time), rather than
/// converted from console 0's much coarser 32px-cell grid. Needed because
/// converting a value that's already quantized to ~40x25 possible
/// positions up into a 107x67 grid doesn't recover precision - most rows
/// in between become unreachable. Used for HUD row hover-detection; see
/// mouse_to_hud for the (coarser, display-only) conversion used to
/// position dungeon-tile tooltips instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudMousePos(pub Point);

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
/// item keys). This is the single source of truth for both what the HUD
/// lists on the left AND which entity a given number key activates
/// (see systems/hud.rs and systems/player_input.rs::use_item) - using the
/// same list in both places keeps the displayed numbering and the actual
/// key-to-item mapping from ever drifting apart.
pub fn usable_carried_items(ecs: &SubWorld, wielder: Entity) -> Vec<Entity> {
    let mut items: Vec<Entity> = <(Entity, &Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, carried)| carried.0 == wielder)
        .map(|(e, _, _)| *e)
        .filter(|e| {
            let entry = ecs.entry_ref(*e).unwrap();
            entry.get_component::<Weapon>().is_err() && entry.get_component::<BattleItem>().is_err()
        })
        .collect();

    // Fixed hotkey slots: Healing Potion always lands on key 1 (when
    // carried), Dungeon Map always lands on key 2, and every other
    // out-of-combat item (Invisible Cloak, Ice Armor, future class
    // items, etc.) fills key 3+ - in whatever order they were picked up,
    // since sort_by_key is stable. Without this, "1" would be whatever
    // happened to iterate first in the ECS, which has no relationship to
    // item type and would shift around depending on pickup order.
    items.sort_by_key(|e| item_hotkey_priority(ecs, *e));
    items
}

/// Groups usable_carried_items by item name, stacking identical copies
/// (e.g. two Healing Potions become one "Healing Potion" entry with a
/// count of 2) instead of one line/hotkey slot per physical copy. Number
/// keys and the HUD's item list both key off position in THIS list now -
/// see systems/player_input.rs::use_item and systems/hud.rs. Keeps one
/// representative entity per group (the first copy encountered); pressing
/// that slot's number key consumes just that one entity, so a stack of 2
/// potions correctly becomes a stack of 1 after using one, not both.
/// Group order still follows usable_carried_items' hotkey priority
/// (potion first, map second, everything else after) - same pattern
/// systems/hud.rs already uses for grouping the battle-items panel.
pub fn usable_item_groups(ecs: &SubWorld, wielder: Entity) -> Vec<(String, i32, Entity)> {
    let mut groups: Vec<(String, i32, Entity)> = Vec::new();
    for item in usable_carried_items(ecs, wielder) {
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

/// Fixed-identity hotkey slots: index 0 is ALWAYS the Healing Potion slot
/// and index 1 is ALWAYS the Dungeon Map slot - `None` when not carried,
/// rather than usable_item_groups' behavior of letting the next item
/// quietly slide up to fill the gap (so key 2 stops being "whatever's
/// second" and starts being "the map, or nothing"). Every other item
/// group follows at index 2+, in usable_item_groups' order. main.rs and
/// hud.rs both index off THIS list now for number keys 1-9.
pub fn usable_item_slots(ecs: &SubWorld, wielder: Entity) -> Vec<Option<(String, i32, Entity)>> {
    let mut potion = None;
    let mut map = None;
    let mut rest = Vec::new();
    for group in usable_item_groups(ecs, wielder) {
        match group.0.as_str() {
            "Healing Potion" => potion = Some(group),
            "Dungeon Map" => map = Some(group),
            _ => rest.push(Some(group)),
        }
    }
    let mut slots = vec![potion, map];
    slots.extend(rest);
    slots
}

/// Sort key used by usable_carried_items - see its comment for the slot
/// assignment. Falls back to the "other items" bucket if the entity has
/// no Name for some reason, rather than panicking.
fn item_hotkey_priority(ecs: &SubWorld, item: Entity) -> i32 {
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
/// position) so both stay in lockstep.
pub const MOVE_ANIM_DURATION_MS: f32 = 150.0;

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

/// Wall-clock milliseconds since the last frame (see BTerm::frame_time_ms),
/// inserted as a resource every tick so animation systems advance at a
/// consistent real-world speed regardless of the current frame rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameTime(pub f32);

// /// Standard ease-out cubic: fast start, gentle settle into the
// /// destination tile rather than a linear, slightly mechanical glide.
// pub fn ease_out_cubic(t: f32) -> f32 {
//     let t = t - 1.0;
//     t * t * t + 1.0
// }

// pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
//     a + (b - a) * t
// }

// /// Where `entity` should actually be drawn this frame: eased between
// /// MovingAnimation.start/.end if one is present and still running,
// /// otherwise its plain logical Point. Keeps the interpolation math in one
// /// place so entity_render doesn't need to know the details.
// pub fn animated_position(ecs: &SubWorld, entity: Entity, logical_pos: Point) -> PointF {
//     if let Ok(entry) = ecs.entry_ref(entity) {
//         if let Ok(anim) = entry.get_component::<MovingAnimation>() {
//             let t = ease_out_cubic((anim.elapsed_ms / MOVE_ANIM_DURATION_MS).min(1.0));
//             return PointF::new(
//                 lerp(anim.start.x as f32, anim.end.x as f32, t),
//                 lerp(anim.start.y as f32, anim.end.y as f32, t),
//             );
//         }
//     }
//     PointF::new(logical_pos.x as f32, logical_pos.y as f32)
// }

/// Computes the same ColorPair/glyph a tile would be drawn with in
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
) -> Option<(ColorPair, FontCharType)> {
    if !map.in_bounds(pt) {
        return None;
    }
    let idx = map_idx(pt.x, pt.y);
    if !(visible_tiles.contains(&pt) || map.revealed_tiles[idx]) {
        return None;
    }
    let visible = visible_tiles.contains(&pt);
    let glyph = theme.tile_to_render(map.tiles[idx]);
    let wall_base = theme.wall_color();

    let color_pair = if map.tiles[idx] == TileType::Wall {
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
    } else {
        let tint = if visible { WHITE } else { DARK_GRAY };
        ColorPair::new(tint, BLACK)
    };

    Some((color_pair, glyph))
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
