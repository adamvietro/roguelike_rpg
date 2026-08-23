use crate::prelude::*;

// --- Battle action capability components -----------------------------------
//
// Each of these is a marker component an entity can carry to say "I can do
// this in battle." The battle menu is built at runtime from whichever of
// these the acting entity actually has, rather than a hardcoded list - so a
// future class can mix and match (e.g. a Mage might get CanAttack + CanFlee
// but not CanDefend, or later a CanCastSpell component of its own) without
// touching the menu code at all.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanAttack;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanDefend;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanFlee;

/// One battle menu option. The always-available capability actions
/// (Attack/Defend/Flee) come from CanXxx components above; Technique wraps
/// a carried item entity whose mechanical effect (TechniqueEffect, see
/// components.rs) is class/content data rather than a fixed enum variant -
/// see `available_actions` and `apply_player_technique`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleAction {
    Attack,
    Defend,
    Flee,
    /// Points at one representative entity from a group of same-named
    /// carried technique items (see `grouped_carried_techniques`) -
    /// resolving it consumes one item from that group.
    Technique(Entity),
}

/// One rendered battle-menu row: the action it triggers, its display
/// label, and a remaining-use count (None for the always-available
/// capability actions, Some(n) for technique items - only included while
/// n > 0). Built fresh each menu render, so using an item immediately
/// updates the count / removes the option once you run out.
#[derive(Clone, Debug, PartialEq)]
pub struct BattleMenuEntry {
    pub action: BattleAction,
    pub label: String,
    pub count: Option<i32>,
}

fn has_can_attack(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanAttack)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

fn has_can_defend(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanDefend)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

fn has_can_flee(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanFlee)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

/// The ordered list of battle-menu rows this entity currently has
/// available. Built fresh each menu render, so using an item immediately
/// updates the count / removes the option once you run out. Class
/// filtering happens once here (via `grouped_carried_techniques`) - callers
/// don't need to know or pass the wielder's class at all.
pub fn available_actions(ecs: &World, entity: Entity) -> Vec<BattleMenuEntry> {
    let mut actions = Vec::new();
    if has_can_attack(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: BattleAction::Attack,
            label: "Attack".to_string(),
            count: None,
        });
    }
    if has_can_defend(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: BattleAction::Defend,
            label: "Defend".to_string(),
            count: None,
        });
    }

    let class = entity_class(ecs, entity).unwrap_or_default();
    for (name, entities) in grouped_carried_techniques(ecs, entity, &class) {
        actions.push(BattleMenuEntry {
            action: BattleAction::Technique(entities[0]),
            label: name,
            count: Some(entities.len() as i32),
        });
    }

    if has_can_flee(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: BattleAction::Flee,
            label: "Flee".to_string(),
            count: None,
        });
    }
    actions
}

/// Maps the number-row keys to a 0-based menu index, matching the existing
/// item-use UX (Key1..Key9) elsewhere in the game.
pub fn number_key_index(key: VirtualKeyCode) -> Option<usize> {
    match key {
        VirtualKeyCode::Key1 => Some(0),
        VirtualKeyCode::Key2 => Some(1),
        VirtualKeyCode::Key3 => Some(2),
        VirtualKeyCode::Key4 => Some(3),
        VirtualKeyCode::Key5 => Some(4),
        VirtualKeyCode::Key6 => Some(5),
        VirtualKeyCode::Key7 => Some(6),
        VirtualKeyCode::Key8 => Some(7),
        VirtualKeyCode::Key9 => Some(8),
        _ => None,
    }
}

// --- Carried battle-item lookups --------------------------------------------
//
// Each returns every copy of that item `wielder` is currently carrying AND
// can actually use (class-unrestricted items, or items matching
// `wielder_class`), so callers can both count them (for the menu) and
// consume one (removing the first entity in the list) when used.

/// The class name on an entity's Class component, if it has one.
pub fn entity_class(ecs: &World, entity: Entity) -> Option<String> {
    <(Entity, &Class)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, c)| c.0.clone())
}

/// True if an item entity has no class restriction, or its Class matches
/// `wielder_class`.
fn item_usable_by_class(ecs: &World, item: Entity, wielder_class: &str) -> bool {
    entity_class(ecs, item)
        .map(|item_class| item_class == wielder_class)
        .unwrap_or(true)
}

/// Every Carried+Technique item `wielder` can currently use (unrestricted,
/// or matching `wielder_class`), grouped by display Name with all matching
/// entities kept together (so the menu can show "Deathblow x2" and consume
/// one at a time). Replaces the old one-function-per-technique-type
/// approach - the mechanical difference between techniques is data
/// (TechniqueEffect) now, not a distinct Rust component type, so one
/// generic lookup covers every class's techniques.
pub fn grouped_carried_techniques(
    ecs: &World,
    wielder: Entity,
    wielder_class: &str,
) -> Vec<(String, Vec<Entity>)> {
    let mut groups: Vec<(String, Vec<Entity>)> = Vec::new();
    <(Entity, &Carried, &Technique, &Name)>::query()
        .iter(ecs)
        .filter(|(_, carried, _, _)| carried.0 == wielder)
        .filter(|(e, _, _, _)| item_usable_by_class(ecs, **e, wielder_class))
        .for_each(
            |(e, _, _, name)| match groups.iter_mut().find(|(n, _)| *n == name.0) {
                Some((_, entities)) => entities.push(*e),
                None => groups.push((name.0.clone(), vec![*e])),
            },
        );
    groups
}

/// The TechniqueEffect a carried item's Technique component holds, if it
/// has one.
pub fn technique_effect(ecs: &World, item: Entity) -> Option<TechniqueEffect> {
    <(Entity, &Technique)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == item)
        .map(|(_, t)| t.0)
}

/// An entity's Name text, or a generic fallback if it has none.
pub fn entity_name(ecs: &World, entity: Entity) -> String {
    <(Entity, &Name)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, n)| n.0.clone())
        .unwrap_or_else(|| "technique".to_string())
}

// --- Battle state ------------------------------------------------------

/// Which combatant is acting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combatant {
    Player,
    Enemy,
}

/// Which part of the battle round we're in. A round goes:
/// (whoever's faster acts first - automatically, with no menu, if it's the
/// enemy) -> FirstResult -> (the other combatant acts - PlayerMenu if it's
/// the player, automatic if it's the enemy) -> SecondResult -> next round.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleTurn {
    /// Waiting for the player to pick a menu action - shown either at the
    /// start of a round (player is faster) or after the enemy's opening
    /// move resolves (enemy is faster).
    PlayerMenu,
    /// Result of whichever combatant acted first this round.
    FirstResult,
    /// Result of whichever combatant acted second this round.
    SecondResult,
}

/// Resource describing an in-progress battle. Lives in `Resources` as
/// `Option<Battle>` - `None` when no battle is happening, `Some(..)` while
/// `TurnState::InBattle` is active.
#[derive(Clone, Debug, PartialEq)]
pub struct Battle {
    pub player: Entity,
    pub enemy: Entity,
    pub enemy_name: String,
    pub turn: BattleTurn,
    /// Who acts first this round, decided by Speed at the start of each
    /// round (see battle_tick). Placeholder value until then.
    pub first_actor: Combatant,
    /// True right after a new round begins and initiative hasn't been
    /// decided yet - battle_tick resolves this before rendering anything.
    pub awaiting_order_decision: bool,
    pub player_defending: bool,
    /// Set by any Counter-shaped technique; consumed (and cleared) by the
    /// next resolve_enemy_attack call, whenever that happens to land. Holds
    /// the technique's own chance/multiplier rather than a hardcoded
    /// constant, so different classes' counter-style techniques can differ.
    pub countering: Option<CounterState>,
    /// A damage-over-time effect currently active on the enemy (e.g. from
    /// a Rend-shaped technique), ticked once per round in `tick_dot`.
    /// None when inactive.
    pub enemy_dot: Option<DotState>,
    pub fled: bool,
    pub message: String,
    /// A brief post-action color flash for each portrait - which kind
    /// (Attacking/Hit, picking the tint color) and how many milliseconds
    /// are left, ticked down each frame in battle_tick using
    /// ctx.frame_time_ms and cleared to None once it reaches zero. Set
    /// whenever that combatant acts or takes damage; draw_battle_arena
    /// reads these to tint the portrait's foreground color while active
    /// (see flash_tint in main.rs).
    pub enemy_flash: Option<(FlashKind, f32)>,
    pub player_flash: Option<(FlashKind, f32)>,
}

/// Which color a portrait's brief post-action flash should use - see
/// Battle::enemy_flash/player_flash.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlashKind {
    /// This combatant just landed a hit - a bright, energetic flash.
    Attacking,
    /// This combatant just took damage - a red "ouch" flash.
    Hit,
}

impl Battle {
    pub fn new(player: Entity, enemy: Entity, enemy_name: String) -> Self {
        Self {
            player,
            enemy,
            enemy_name,
            turn: BattleTurn::PlayerMenu,
            first_actor: Combatant::Player,
            awaiting_order_decision: true,
            player_defending: false,
            countering: None,
            enemy_dot: None,
            fled: false,
            message: String::new(),
            enemy_flash: None,
            player_flash: None,
        }
    }
}

/// A pending counter-technique's chance/multiplier - see Battle::countering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CounterState {
    pub chance_percent: i32,
    pub multiplier: i32,
}

/// An active damage-over-time effect on the enemy - see Battle::enemy_dot.
#[derive(Clone, Debug, PartialEq)]
pub struct DotState {
    pub damage: i32,
    pub turns_remaining: i32,
    /// The technique's own name, lowercased, for the per-tick message
    /// (e.g. "The garrote bites...").
    pub label: String,
}

/// How long a portrait's post-action color flash lasts, in milliseconds.
/// See Battle::enemy_flash/player_flash and flash_tint in main.rs.
pub const PORTRAIT_FLASH_DURATION_MS: f32 = 150.0;

/// What to show on the post-battle victory screen (TurnState::BattleVictory)
/// - set right when an enemy dies in battle_tick, read once by
/// battle_victory_tick, then cleared when the player dismisses it.
/// `player` stays valid since the player entity is never removed, so its
/// Render is looked up live - the enemy is gone by this point and isn't
/// shown.
#[derive(Clone, Debug, PartialEq)]
pub struct BattleVictory {
    pub player: Entity,
    pub enemy_name: String,
    pub loot: Option<String>,
}

// --- Shared lookups/helpers used by the battle screen -----------------------

/// An entity's own base Damage component value, or 0 if it doesn't have one.
pub fn entity_damage(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Damage)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, d)| d.0)
        .unwrap_or(0)
}

/// An entity's own Speed component value, or a neutral default (5) if it
/// doesn't have one. Higher acts first in battle - see battle_tick.
pub fn entity_speed(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Speed)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, s)| s.0)
        .unwrap_or(5)
}

/// Sum of Damage on anything Carried by `wielder` (i.e. equipped weapons).
/// Mirrors the weapon-damage lookup the old combat system used.
pub fn carried_weapon_damage(ecs: &World, wielder: Entity) -> i32 {
    <(&Carried, &Damage)>::query()
        .iter(ecs)
        .filter(|(carried, _)| carried.0 == wielder)
        .map(|(_, dmg)| dmg.0)
        .sum()
}

/// Current/max HP for an entity, or (0, 0) if it has no Health component.
pub fn entity_health(ecs: &World, entity: Entity) -> (i32, i32) {
    <(Entity, &Health)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, h)| (h.current, h.max))
        .unwrap_or((0, 0))
}

/// Subtract `amount` from an entity's current Health. Can go below zero;
/// callers check for death via entity_health and clamp for display.
/// Defense will remove damage based off the current defense 1 for 1
pub fn apply_damage(ecs: &mut World, entity: Entity, amount: i32) {
    <(Entity, &mut Health, Option<&Defense>)>::query()
        .iter_mut(ecs)
        .filter(|(e, _, _)| **e == entity)
        .for_each(|(_, hp, defense)| {
            let defense = defense.map_or(0, |d| d.0);
            let damage = (amount - defense).max(0);
            hp.current -= damage;
        });
}

/// The player's normal attack damage: base Damage plus any equipped weapon.
pub fn player_attack_damage(ecs: &World, player: Entity) -> i32 {
    entity_damage(ecs, player) + carried_weapon_damage(ecs, player)
}

/// The enemy automatically attacks the player. Enemies currently only ever
/// know Attack (see CanAttack / available_actions), so this is a simple
/// hardcoded action - a natural place for smarter enemy AI to hook in
/// later. Applies the player's Defend reduction if active, then clears it
/// (Defend only blocks the next hit taken, from whichever side lands it).
/// If Counter Attack is armed, rolls it here too, since this is the single
/// place every enemy attack against the player passes through regardless
/// of initiative order. Returns the message to show for this action.
pub fn resolve_enemy_attack(ecs: &mut World, battle: &mut Battle) -> String {
    let mut dmg = entity_damage(ecs, battle.enemy);
    if battle.player_defending && dmg > 0 {
        dmg = (dmg / 2).max(1);
    }
    apply_damage(ecs, battle.player, dmg);
    battle.enemy_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
    battle.player_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
    let mut message = if battle.player_defending {
        format!(
            "The {} attacks - you block some of it! ({} damage)",
            battle.enemy_name, dmg
        )
    } else {
        format!("The {} attacks you for {} damage!", battle.enemy_name, dmg)
    };
    battle.player_defending = false;

    if let Some(counter) = battle.countering.take() {
        let mut rng = RandomNumberGenerator::new();
        if rng.range(0, 100) < counter.chance_percent {
            let counter_dmg = player_attack_damage(ecs, battle.player) * counter.multiplier;
            apply_damage(ecs, battle.enemy, counter_dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            message = format!("{} You counter for {} damage!", message, counter_dmg);
        } else {
            message = format!("{} Your counter-attack missed!", message);
        }
    }

    message
}

/// If a damage-over-time effect is active on the enemy, ticks it down by
/// one and applies its damage. Called once at the start of each round.
/// Returns a message describing the tick if it happened, or None if no
/// effect is active. Generic over whichever technique applied it (Rend,
/// Burn, or any future one) - see Battle::enemy_dot.
pub fn tick_dot(ecs: &mut World, battle: &mut Battle) -> Option<String> {
    let (damage, label, turns_remaining) = match &battle.enemy_dot {
        Some(dot) => (dot.damage, dot.label.clone(), dot.turns_remaining),
        None => return None,
    };
    if turns_remaining <= 0 {
        battle.enemy_dot = None;
        return None;
    }
    apply_damage(ecs, battle.enemy, damage);
    battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
    let remaining = turns_remaining - 1;
    if remaining <= 0 {
        battle.enemy_dot = None;
    } else if let Some(dot) = &mut battle.enemy_dot {
        dot.turns_remaining = remaining;
    }
    Some(format!(
        "The {} bites - {} takes {} damage!",
        label, battle.enemy_name, damage
    ))
}

/// Restores `amount` HP to an entity, clamped to its max. Used by the
/// Heal technique effect.
pub fn heal_entity(ecs: &mut World, entity: Entity, amount: i32) {
    <(Entity, &mut Health)>::query()
        .iter_mut(ecs)
        .filter(|(e, _)| **e == entity)
        .for_each(|(_, hp)| hp.current = (hp.current + amount).min(hp.max));
}

/// Applies a chosen technique's effect on behalf of the player, consuming
/// one copy of `item` first. This is the single place a technique's
/// mechanical effect is interpreted - main.rs no longer needs one match
/// arm per technique. Adding a new class's technique that reuses an
/// existing TechniqueEffect shape needs zero code changes here (just a
/// template.ron entry); a genuinely new mechanic needs one new match arm,
/// not a new component/BattleAction variant/main.rs block like before.
pub fn apply_player_technique(ecs: &mut World, battle: &mut Battle, item: Entity) -> String {
    let effect = match technique_effect(ecs, item) {
        Some(e) => e,
        None => return String::new(),
    };
    let name = entity_name(ecs, item);

    let mut cb = CommandBuffer::new(ecs);
    cb.remove(item);
    cb.flush(ecs);

    match effect {
        TechniqueEffect::DamageMultiplier(multiplier) => {
            let dmg = player_attack_damage(ecs, battle.player) * multiplier;
            apply_damage(ecs, battle.enemy, dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            format!(
                "{}! You strike the {} for {} damage!",
                name, battle.enemy_name, dmg
            )
        }
        TechniqueEffect::FlatDamage(amount) => {
            let dmg = amount + carried_weapon_damage(ecs, battle.player);
            apply_damage(ecs, battle.enemy, dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            format!(
                "{}! You blast the {} for {} damage!",
                name, battle.enemy_name, dmg
            )
        }
        TechniqueEffect::MultiHit(hits) => {
            let dmg = player_attack_damage(ecs, battle.player);
            for _ in 0..hits {
                apply_damage(ecs, battle.enemy, dmg);
            }
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            format!(
                "{}! You strike the {} {} times for {} damage each!",
                name, battle.enemy_name, hits, dmg
            )
        }
        TechniqueEffect::Counter {
            chance_percent,
            multiplier,
        } => {
            battle.countering = Some(CounterState {
                chance_percent,
                multiplier,
            });
            format!("You ready a {}...", name.to_lowercase())
        }
        TechniqueEffect::DamageOverTime { damage, turns } => {
            battle.enemy_dot = Some(DotState {
                damage,
                turns_remaining: turns,
                label: name.to_lowercase(),
            });
            format!(
                "You use {} on the {} - it will wound them over time!",
                name, battle.enemy_name
            )
        }
        TechniqueEffect::Heal { amount } => {
            heal_entity(ecs, battle.player, amount);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            format!("{}! You recover {} HP!", name, amount)
        }
    }
}

/// An entity's Render component (color + glyph), if it has one. Used to draw
/// the scaled-up battle portraits using the same glyph the entity uses on
/// the dungeon map.
pub fn entity_render_component(ecs: &World, entity: Entity) -> Option<Render> {
    <(Entity, &Render)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, r)| *r)
}

/// A simple bracket-style text health bar, e.g. "[######----]".
pub fn hp_bar_string(current: i32, max: i32, width: usize) -> String {
    if max <= 0 {
        return format!("[{}]", "-".repeat(width));
    }
    let ratio = (current.max(0) as f32 / max as f32).min(1.0);
    let filled = ((ratio * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}
