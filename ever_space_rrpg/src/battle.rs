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
/// label, and a remaining-use count. `action` is None for a class
/// technique you don't currently own a copy of - it's still shown (greyed
/// out, see main.rs) so the menu always reflects the class's full
/// technique roster rather than only whatever you happen to be carrying,
/// but there's no Entity to reference for it and it can't be selected.
/// Built fresh each menu render, so using an item immediately updates the
/// count.
#[derive(Clone, Debug, PartialEq)]
pub struct BattleMenuEntry {
    pub action: Option<BattleAction>,
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
/// available. Always shows the acting class's full technique roster (see
/// class_technique_names) - not just what's currently carried - so the
/// menu stays a stable reference of "what this class can eventually do";
/// entries for techniques not currently owned get `action: None` (count
/// Some(0)) so main.rs can grey them out and skip them on selection. Class
/// filtering happens once here - callers don't need to know or pass the
/// wielder's class at all.
pub fn available_actions(ecs: &World, entity: Entity) -> Vec<BattleMenuEntry> {
    let mut actions = Vec::new();
    if has_can_attack(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Attack),
            label: "Attack".to_string(),
            count: None,
        });
    }
    if has_can_defend(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Defend),
            label: "Defend".to_string(),
            count: None,
        });
    }

    let class = entity_class(ecs, entity).unwrap_or_default();
    let owned = grouped_carried_techniques(ecs, entity, &class);
    for name in class_technique_names(&class) {
        match owned.iter().find(|(n, _)| *n == name) {
            Some((_, entities)) => actions.push(BattleMenuEntry {
                action: Some(BattleAction::Technique(entities[0])),
                label: name,
                count: Some(entities.len() as i32),
            }),
            None => actions.push(BattleMenuEntry {
                action: None,
                label: name,
                count: Some(0),
            }),
        }
    }

    if has_can_flee(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Flee),
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
    /// Scrolling battle log, most recent entry last - replaces the old
    /// single `message: String` so earlier lines (e.g. a DoT tick right
    /// before an attack) stay readable instead of being overwritten the
    /// instant the next action resolves. Bounded to MAX_LOG_LINES by
    /// push_log; main.rs renders the tail of this Vec every battle_tick
    /// frame instead of a single centered line.
    pub log: Vec<String>,
    /// A brief post-action color flash for each portrait - which kind
    /// (Attacking/Hit, picking the tint color) and how many milliseconds
    /// are left, ticked down each frame in battle_tick using
    /// ctx.frame_time_ms and cleared to None once it reaches zero. Set
    /// whenever that combatant acts or takes damage; draw_battle_arena
    /// reads these to tint the portrait's foreground color while active
    /// (see flash_tint in render_helpers.rs).
    pub enemy_flash: Option<(FlashKind, f32)>,
    pub player_flash: Option<(FlashKind, f32)>,
    /// A briefly-shown floating damage number over each portrait - set by
    /// show_enemy_damage/show_player_damage right after apply_damage
    /// returns the real (post-Defense) amount, and ticked down each frame
    /// in battle_tick the same way as enemy_flash/player_flash.
    pub enemy_damage_popup: Option<DamagePopup>,
    pub player_damage_popup: Option<DamagePopup>,
    /// Counts down while `turn` is FirstResult/SecondResult - once it hits
    /// zero, battle_tick advances automatically instead of waiting for a
    /// keypress (a keypress still skips ahead immediately, it just isn't
    /// required anymore). Set via enter_result whenever the turn changes
    /// to one of those two states.
    pub result_timer_ms: f32,
    /// True for a battle that started as a Stealth ambush (see
    /// systems/player_input.rs) - forces this battle's first round to go
    /// to the player regardless of Speed, and triples the damage of
    /// whichever action the player picks first. Cleared (one-shot) the
    /// instant that first PlayerMenu action resolves, whatever it was -
    /// see screens/battle.rs's BattleTurn::PlayerMenu handling.
    pub sneak_attack: bool,
    /// An active Evade-technique bonus (e.g. Rogue's Dodge) - adds
    /// `chance_percent` on top of the player's base Evasion for the next
    /// `turns` enemy attacks faced, ticked down once per attack in
    /// resolve_enemy_attack regardless of whether that attack was
    /// evaded. None when inactive.
    pub dodge_bonus: Option<DodgeState>,
    /// An active Battle Cry (Amazon) - reduces the enemy's outgoing
    /// damage by a random amount each hit for the next N attacks. Unlike
    /// dodge_bonus, this only ticks down when an attack actually connects
    /// (mirrors Ice Armor's placement in resolve_enemy_attack) - a fully
    /// evaded attack didn't deal damage to reduce, so it shouldn't spend
    /// a charge either. None when inactive.
    pub war_cry: Option<WarCryState>,
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
            log: Vec::new(),
            enemy_flash: None,
            player_flash: None,
            enemy_damage_popup: None,
            player_damage_popup: None,
            result_timer_ms: 0.0,
            sneak_attack: false,
            dodge_bonus: None,
            war_cry: None,
        }
    }

    /// Marks this battle as a Stealth ambush - see Battle::sneak_attack.
    /// Chainable so player_input.rs can set it right after Battle::new
    /// without an extra statement.
    pub fn as_sneak_attack(mut self) -> Self {
        self.sneak_attack = true;
        self
    }

    /// Switches to FirstResult or SecondResult and (re)arms the
    /// auto-advance timer. Use this instead of assigning `self.turn`
    /// directly for those two states, so the timer can never be left
    /// stale from a previous result screen.
    pub fn enter_result(&mut self, turn: BattleTurn) {
        self.turn = turn;
        self.result_timer_ms = RESULT_AUTO_ADVANCE_MS;
    }

    /// Appends a line to the battle log, dropping the oldest line once past
    /// MAX_LOG_LINES. Skips genuinely empty strings so a DoT tick that had
    /// nothing to report (see tick_dot's None case) doesn't leave a blank
    /// entry in the log.
    pub fn push_log(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        self.log.push(line);
        if self.log.len() > MAX_LOG_LINES {
            self.log.remove(0);
        }
    }

    /// Arms a floating damage number over the enemy's portrait - call with
    /// the real (post-Defense) amount apply_damage returned.
    pub fn show_enemy_damage(&mut self, amount: i32) {
        self.enemy_damage_popup = Some(DamagePopup {
            amount,
            remaining_ms: DAMAGE_POPUP_DURATION_MS,
        });
    }

    /// Arms a floating damage number over the player's portrait - call with
    /// the real (post-Defense) amount apply_damage returned.
    pub fn show_player_damage(&mut self, amount: i32) {
        self.player_damage_popup = Some(DamagePopup {
            amount,
            remaining_ms: DAMAGE_POPUP_DURATION_MS,
        });
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

/// An active Evade-technique bonus on the player - see Battle::dodge_bonus.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DodgeState {
    pub chance_percent: i32,
    pub turns_remaining: i32,
}

/// An active Battle Cry (Amazon) on the player - see Battle::war_cry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarCryState {
    pub min_reduction: i32,
    pub max_reduction: i32,
    pub attacks_remaining: i32,
}

/// How long a portrait's post-action color flash lasts, in milliseconds.
/// See Battle::enemy_flash/player_flash and flash_tint in render_helpers.rs.
pub const PORTRAIT_FLASH_DURATION_MS: f32 = 150.0;

/// A floating damage number shown briefly over a portrait - see
/// Battle::enemy_damage_popup/player_damage_popup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamagePopup {
    pub amount: i32,
    pub remaining_ms: f32,
}

/// How long a floating damage number stays on screen, in milliseconds.
pub const DAMAGE_POPUP_DURATION_MS: f32 = 700.0;

/// Battle::log is trimmed to this many most-recent lines - see push_log.
pub const MAX_LOG_LINES: usize = 4;

/// How long FirstResult/SecondResult sit on screen before battle_tick
/// advances automatically - see Battle::result_timer_ms/enter_result. A
/// keypress still skips ahead immediately; this is just the natural pace
/// when the player doesn't bother pressing anything.
pub const RESULT_AUTO_ADVANCE_MS: f32 = 1100.0;

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

/// An entity's own Evasion component value, or 0 (no innate dodge chance)
/// if it doesn't have one - most classes/enemies today.
pub fn entity_evasion(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Evasion)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, ev)| ev.0)
        .unwrap_or(0)
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

/// Subtract `amount` from an entity's current Health, reduced by the
/// target's Defense (if any). Can go below zero; callers check for death
/// via entity_health and clamp for display. Returns the actual damage
/// dealt (post-Defense) - callers should use this return value, not the
/// `amount` they passed in, when building a message: the two can diverge
/// for any entity with nonzero Defense (e.g. Mage's -1).
pub fn apply_damage(ecs: &mut World, entity: Entity, amount: i32) -> i32 {
    let mut actual_damage = 0;
    <(Entity, &mut Health, Option<&Defense>)>::query()
        .iter_mut(ecs)
        .filter(|(e, _, _)| **e == entity)
        .for_each(|(_, hp, defense)| {
            let defense = defense.map_or(0, |d| d.0);
            let damage = (amount - defense).max(0);
            hp.current -= damage;
            actual_damage = damage;
        });
    actual_damage
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
/// of initiative order. Pushes its own log line(s) directly (main hit, and
/// a separate counter line if one fires) rather than returning a string -
/// the terse log format doesn't try to narrate cause (Defend vs Ice Armor
/// vs neither all just reduce the same final number), so there's nothing
/// left for a caller to do with a returned message.
pub fn resolve_enemy_attack(ecs: &mut World, battle: &mut Battle) {
    // Evasion check first (base Evasion stat + any active Dodge-technique
    // bonus, additive - see components::Evasion / Battle::dodge_bonus). A
    // full dodge skips the entire rest of this function: no Defend or Ice
    // Armor gets consumed and no Counter triggers, since nothing actually
    // landed to defend against or counter.
    let dodge_chance = entity_evasion(ecs, battle.player)
        + battle.dodge_bonus.as_ref().map_or(0, |d| d.chance_percent);
    let mut rng = RandomNumberGenerator::new();
    let evaded = dodge_chance > 0 && rng.range(0, 100) < dodge_chance;

    // The Dodge-technique bonus's duration ticks down once per enemy
    // attack faced, regardless of whether this particular attack was the
    // one that got evaded.
    if let Some(dodge) = &mut battle.dodge_bonus {
        dodge.turns_remaining -= 1;
        if dodge.turns_remaining <= 0 {
            battle.dodge_bonus = None;
        }
    }

    if evaded {
        battle.enemy_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
        battle.push_log("Dodge attack.".to_string());
        battle.player_defending = false;
        return;
    }

    let mut dmg = entity_damage(ecs, battle.enemy);
    if battle.player_defending && dmg > 0 {
        dmg = (dmg / 2).max(1);
    }

    // Ice Armor is applied out-of-combat (via an Invisible-Cloak-style
    // item - "Mages buff before battle") and persists as a status on the
    // player rather than per-Battle state, so it's looked up here instead
    // of read from `battle` directly - see components::IceArmored.
    let ice_armor = entity_ice_armor(ecs, battle.player);
    if let Some(armor) = &ice_armor {
        dmg = (dmg - armor.defense_bonus).max(0);
    }

    // Battle Cry (Amazon) - a random 1-2 (or whatever the technique's
    // configured range is) reduction per hit, on top of any Ice Armor
    // already subtracted above. Ticks down (and clears once exhausted)
    // only here, past the evaded-early-return above - a dodge shouldn't
    // spend a Battle Cry charge, since no damage landed to reduce.
    if let Some(cry) = &mut battle.war_cry {
        let mut rng = RandomNumberGenerator::new();
        let reduction = rng.range(cry.min_reduction, cry.max_reduction + 1);
        dmg = (dmg - reduction).max(0);
        cry.attacks_remaining -= 1;
        if cry.attacks_remaining <= 0 {
            battle.war_cry = None;
        }
    }

    let dmg = apply_damage(ecs, battle.player, dmg);
    battle.show_player_damage(dmg);
    battle.enemy_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
    battle.player_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
    battle.push_log(if dmg == 0 {
        "Dodge attack.".to_string()
    } else {
        format!("Take {} damage.", dmg)
    });
    battle.player_defending = false;

    // Ice Armor wears down by one attack actually absorbed, same "attacks"
    // semantics the old in-battle Shield technique used - just persistent
    // across turns and battles now instead of scoped to a single fight.
    if let Some(armor) = ice_armor {
        let mut cb = CommandBuffer::new(ecs);
        if armor.attacks_remaining <= 1 {
            cb.remove_component::<IceArmored>(battle.player);
        } else {
            cb.add_component(
                battle.player,
                IceArmored {
                    defense_bonus: armor.defense_bonus,
                    attacks_remaining: armor.attacks_remaining - 1,
                },
            );
        }
        cb.flush(ecs);
    }

    if let Some(counter) = battle.countering.take() {
        let mut rng = RandomNumberGenerator::new();
        if rng.range(0, 100) < counter.chance_percent {
            let counter_dmg = player_attack_damage(ecs, battle.player) * counter.multiplier;
            let counter_dmg = apply_damage(ecs, battle.enemy, counter_dmg);
            battle.show_enemy_damage(counter_dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            battle.push_log(if counter_dmg == 0 {
                "Dodge attack.".to_string()
            } else {
                format!("Deal {} damage.", counter_dmg)
            });
        } else {
            battle.push_log("Miss counter.".to_string());
        }
    }
}

/// An entity's active IceArmored bonus, if any - see resolve_enemy_attack.
/// Public so main.rs can also show it as an active-status line in battle.
pub fn entity_ice_armor(ecs: &World, entity: Entity) -> Option<IceArmored> {
    <(Entity, &IceArmored)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, armor)| *armor)
}

/// If a damage-over-time effect is active on the enemy, ticks it down by
/// one and applies its damage. Called once at the start of each round.
/// Returns a message describing the tick if it happened, or None if no
/// effect is active. Generic over whichever technique applied it (Rend,
/// Burn, or any future one) - see Battle::enemy_dot.
pub fn tick_dot(ecs: &mut World, battle: &mut Battle) -> Option<String> {
    let (damage, turns_remaining) = match &battle.enemy_dot {
        Some(dot) => (dot.damage, dot.turns_remaining),
        None => return None,
    };
    if turns_remaining <= 0 {
        battle.enemy_dot = None;
        return None;
    }
    let damage = apply_damage(ecs, battle.enemy, damage);
    battle.show_enemy_damage(damage);
    battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
    let remaining = turns_remaining - 1;
    if remaining <= 0 {
        battle.enemy_dot = None;
    } else if let Some(dot) = &mut battle.enemy_dot {
        dot.turns_remaining = remaining;
    }
    Some(if damage == 0 {
        "Dodge attack.".to_string()
    } else {
        format!("Deal {} damage.", damage)
    })
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
            let dmg = apply_damage(ecs, battle.enemy, dmg);
            battle.show_enemy_damage(dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            if dmg == 0 {
                "Dodge attack.".to_string()
            } else {
                format!("Deal {} damage.", dmg)
            }
        }
        TechniqueEffect::FlatDamage(amount) => {
            let dmg = amount + carried_weapon_damage(ecs, battle.player);
            let dmg = apply_damage(ecs, battle.enemy, dmg);
            battle.show_enemy_damage(dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            if dmg == 0 {
                "Dodge attack.".to_string()
            } else {
                format!("Deal {} damage.", dmg)
            }
        }
        TechniqueEffect::MultiHit(hits) => {
            let raw = player_attack_damage(ecs, battle.player);
            let mut dmg = 0;
            for _ in 0..hits {
                dmg = apply_damage(ecs, battle.enemy, raw);
            }
            // Only the last hit's damage gets a popup/log line - showing
            // every hit would need a queue of popups rather than one slot,
            // which is more than this technique needs right now.
            battle.show_enemy_damage(dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            if dmg == 0 {
                "Dodge attack.".to_string()
            } else {
                format!("Deal {} damage.", dmg)
            }
        }
        TechniqueEffect::Counter {
            chance_percent,
            multiplier,
        } => {
            battle.countering = Some(CounterState {
                chance_percent,
                multiplier,
            });
            "Ready counter.".to_string()
        }
        TechniqueEffect::DamageOverTime { damage, turns } => {
            battle.enemy_dot = Some(DotState {
                damage,
                turns_remaining: turns,
                label: name.to_lowercase(),
            });
            "Apply bleed.".to_string()
        }
        TechniqueEffect::Heal { amount } => {
            heal_entity(ecs, battle.player, amount);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            format!("Heal {} HP.", amount)
        }
        TechniqueEffect::Evade {
            chance_percent,
            turns,
        } => {
            battle.dodge_bonus = Some(DodgeState {
                chance_percent,
                turns_remaining: turns,
            });
            "Boost evasion.".to_string()
        }
        TechniqueEffect::WarCry {
            min_reduction,
            max_reduction,
            attacks,
        } => {
            battle.war_cry = Some(WarCryState {
                min_reduction,
                max_reduction,
                attacks_remaining: attacks,
            });
            "Rally your courage.".to_string()
        }
        TechniqueEffect::PoisonStrike {
            initial,
            dot_damage,
            dot_turns,
        } => {
            let dmg = initial + carried_weapon_damage(ecs, battle.player);
            let dmg = apply_damage(ecs, battle.enemy, dmg);
            battle.show_enemy_damage(dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_dot = Some(DotState {
                damage: dot_damage,
                turns_remaining: dot_turns,
                label: name.to_lowercase(),
            });
            if dmg == 0 {
                "Dodge attack. Poison lingers.".to_string()
            } else {
                format!("Deal {} damage. Poison lingers.", dmg)
            }
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
