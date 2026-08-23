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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Item;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weapon;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmuletOfYala;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesHealing {
    pub amount: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesDungeonMap;

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
    /// Boost your own Defense by `defense_bonus`, reducing incoming damage
    /// further, for the next `attacks` enemy hits actually landed on you
    /// (not turns - only counts down when the enemy connects). Scoped to
    /// the battle it's cast in; doesn't carry over into the next fight -
    /// see Battle::shield.
    Shield { defense_bonus: i32, attacks: i32 },
    /// Restore `amount` HP to yourself right now. Not used by any current
    /// Barbarian technique - included so a Mage healing spell has
    /// somewhere to plug in without another battle.rs change.
    Heal { amount: i32 },
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

/// Marks an Item entity as an Invisible Cloak - granting `moves` turns of
/// Invisible on use (see systems/use_items.rs). Ordinary carried item, not
/// a BattleItem - used from the dungeon-view item keys, not the battle
/// menu, since it's explicitly an out-of-combat ability.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProvidesInvisibility {
    pub moves: i32,
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
    <(Entity, &Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, carried)| carried.0 == wielder)
        .map(|(e, _, _)| *e)
        .filter(|e| {
            let entry = ecs.entry_ref(*e).unwrap();
            entry.get_component::<Weapon>().is_err() && entry.get_component::<BattleItem>().is_err()
        })
        .collect()
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
