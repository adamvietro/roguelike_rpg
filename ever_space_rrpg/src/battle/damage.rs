use crate::prelude::*;

// --- Pure damage ------------------------------------------------------------
//
// Everything here is "hit the other combatant for N right now" - no
// lingering status, no roll beyond the hit itself. Before the ATB/multi-
// enemy refactors this exact block was duplicated multiple times across
// the plain Attack, DamageMultiplier, FlatDamage, MultiHit, and
// PoisonStrike's immediate half. `strike_enemy`/`strike_player` below are
// the two shared landing points everything (including Counter and an
// enemy's own normal attack) now goes through - split into two functions,
// rather than the old single `strike(Combatant)`, because "which enemy"
// needs an explicit Entity now that a battle can hold more than one; the
// player, being singular, never needed that ambiguity in the first place.

/// Deals `amount` damage from the PLAYER to `target` (a specific enemy in
/// this battle), arms that enemy's damage popup, and flashes both
/// portraits the way every direct hit in this game always has (player
/// flashes Attacking, target flashes Hit). Returns the actual post-
/// Defense damage dealt - callers should use this, not `amount`, when
/// building a message.
pub fn strike_enemy(ecs: &mut World, battle: &mut Battle, target: Entity, amount: i32) -> i32 {
    let dmg = apply_damage(ecs, target, amount);
    battle.show_enemy_damage(target, dmg);
    battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
    battle.set_enemy_flash(target, FlashKind::Hit);
    dmg
}

/// Deals `amount` damage from `attacker` (a specific enemy in this
/// battle) to the PLAYER, arms the player's damage popup, and flashes
/// both portraits.
pub fn strike_player(ecs: &mut World, battle: &mut Battle, attacker: Entity, amount: i32) -> i32 {
    let dmg = apply_damage(ecs, battle.player, amount);
    battle.show_player_damage(dmg);
    battle.set_enemy_flash(attacker, FlashKind::Attacking);
    battle.player_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
    dmg
}

/// Standard "Deal N damage." / "Dodge attack." log line for a strike
/// dealt BY the returning combatant (player attacking, or a landed
/// counter).
pub fn strike_message(dmg: i32) -> String {
    if dmg == 0 {
        "Dodge attack.".to_string()
    } else {
        format!("Deal {} damage.", dmg)
    }
}

/// Same as strike_message, but phrased for damage the PLAYER took -
/// resolve_enemy_attack's log line has always read "Take N damage."
/// rather than "Deal", even though it's the same kind of hit.
pub fn take_message(dmg: i32) -> String {
    if dmg == 0 {
        "Dodge attack.".to_string()
    } else {
        format!("Take {} damage.", dmg)
    }
}

/// TechniqueEffect::DamageMultiplier - attack `target` right now for
/// `multiplier`x normal attack damage.
pub fn damage_multiplier(
    ecs: &mut World,
    battle: &mut Battle,
    target: Entity,
    multiplier: i32,
) -> String {
    let amount = player_attack_damage(ecs, battle.player) * multiplier;
    strike_message(strike_enemy(ecs, battle, target, amount))
}

/// TechniqueEffect::FlatDamage - attack `target` for a fixed `amount`
/// plus any carried weapon damage (unlike DamageMultiplier, which scales
/// off the normal attack rather than adding a flat base).
pub fn flat_damage(ecs: &mut World, battle: &mut Battle, target: Entity, amount: i32) -> String {
    let total = amount + carried_weapon_damage(ecs, battle.player);
    strike_message(strike_enemy(ecs, battle, target, total))
}

/// TechniqueEffect::MultiHit - attack `target` `hits` times in a row,
/// each for full normal attack damage. Only the last hit's damage gets a
/// popup/log line - showing every hit would need a queue of popups
/// rather than one slot, which is more than this technique needs.
pub fn multi_hit(ecs: &mut World, battle: &mut Battle, target: Entity, hits: i32) -> String {
    let raw = player_attack_damage(ecs, battle.player);
    let mut dmg = 0;
    for _ in 0..hits {
        dmg = strike_enemy(ecs, battle, target, raw);
    }
    strike_message(dmg)
}
