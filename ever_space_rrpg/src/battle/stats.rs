use crate::prelude::*;

/// The class name on an entity's Class component, if it has one. Generic
/// over EntityStore so it works both from plain `&World` contexts
/// (screens/battle.rs, screens/title.rs) and from inside a `#[system]`'s
/// `&SubWorld` (systems/player_input.rs's use_ability).
pub fn entity_class<T: EntityStore>(ecs: &T, entity: Entity) -> Option<String> {
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
// --- Carried battle-item lookups --------------------------------------------
//
// Each returns every copy of that item `wielder` is currently carrying AND
// can actually use (class-unrestricted items, or items matching
// `wielder_class`), so callers can both count them (for the menu) and
// consume one (removing the first entity in the list) when used.

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

/// How fast `entity`'s ATB gauge fills, in gauge points per millisecond -
/// see ATB_GAUGE_PER_MS_PER_SPEED. Speed 0 or below would never fill at
/// all, which would softlock a battle, so this floors the effective
/// Speed used for the rate at 1 (entity_speed's own "no Speed component"
/// default is already 5, well above this floor - this only guards
/// against a template that explicitly sets speed: Some(0) or a negative
/// value).
pub fn atb_fill_rate(ecs: &World, entity: Entity) -> f32 {
    let speed = entity_speed(ecs, entity).max(1) as f32;
    speed * ATB_GAUGE_PER_MS_PER_SPEED
}

/// An entity's own base Damage component value, or 0 if it doesn't have one.
// --- Shared lookups/helpers used by the battle screen -----------------------

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

/// The player's normal attack damage: base Damage plus any equipped weapon.
pub fn player_attack_damage(ecs: &World, player: Entity) -> i32 {
    entity_damage(ecs, player) + carried_weapon_damage(ecs, player)
}

/// An entity's active IceArmored bonus, if any - see resolve_enemy_attack.
/// Public so main.rs can also show it as an active-status line in battle.
pub fn entity_ice_armor(ecs: &World, entity: Entity) -> Option<IceArmored> {
    <(Entity, &IceArmored)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, armor)| *armor)
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

