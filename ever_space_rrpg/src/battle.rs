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

/// One battle menu option. Add new variants here (and a matching CanXxx
/// component above) as new actions come online.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleAction {
    Attack,
    Defend,
    Flee,
}

impl BattleAction {
    pub fn label(self) -> &'static str {
        match self {
            BattleAction::Attack => "Attack",
            BattleAction::Defend => "Defend",
            BattleAction::Flee => "Flee",
        }
    }
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

/// The ordered list of battle actions this entity currently has available,
/// built from whichever CanXxx components it carries. This is what both the
/// player's menu and (eventually) enemy AI should consult.
pub fn available_actions(ecs: &World, entity: Entity) -> Vec<BattleAction> {
    let mut actions = Vec::new();
    if has_can_attack(ecs, entity) {
        actions.push(BattleAction::Attack);
    }
    if has_can_defend(ecs, entity) {
        actions.push(BattleAction::Defend);
    }
    if has_can_flee(ecs, entity) {
        actions.push(BattleAction::Flee);
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
    pub fled: bool,
    pub message: String,
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
            fled: false,
            message: String::new(),
        }
    }
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
pub fn apply_damage(ecs: &mut World, entity: Entity, amount: i32) {
    <(Entity, &mut Health)>::query()
        .iter_mut(ecs)
        .filter(|(e, _)| **e == entity)
        .for_each(|(_, hp)| hp.current -= amount);
}

/// The enemy automatically attacks the player. Enemies currently only ever
/// know Attack (see CanAttack / available_actions), so this is a simple
/// hardcoded action - a natural place for smarter enemy AI to hook in
/// later. Applies the player's Defend reduction if active, then clears it
/// (Defend only blocks the next hit taken, from whichever side lands it).
/// Returns the message to show for this action.
pub fn resolve_enemy_attack(ecs: &mut World, battle: &mut Battle) -> String {
    let mut dmg = entity_damage(ecs, battle.enemy);
    if battle.player_defending && dmg > 0 {
        dmg = (dmg / 2).max(1);
    }
    apply_damage(ecs, battle.player, dmg);
    let message = if battle.player_defending {
        format!(
            "The {} attacks - you block some of it! ({} damage)",
            battle.enemy_name, dmg
        )
    } else {
        format!("The {} attacks you for {} damage!", battle.enemy_name, dmg)
    };
    battle.player_defending = false;
    message
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
