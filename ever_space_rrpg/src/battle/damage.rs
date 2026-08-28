use crate::prelude::*;

// --- Pure damage ------------------------------------------------------------
//
// Everything here is "hit the other combatant for N right now" - no
// lingering status, no roll beyond the hit itself. Before this refactor
// this exact 6-line apply_damage+popup+flash block was duplicated five
// separate times (plain Attack in screens/battle.rs, DamageMultiplier,
// FlatDamage, MultiHit, and PoisonStrike's immediate half). `strike` below
// is the one shared landing point all five (plus Counter and the enemy's
// own normal attack) now go through.

/// Deals `amount` damage from `attacker`'s side to the other combatant,
/// arms the defender's damage popup, and flashes both portraits the way
/// every direct hit in this game always has (attacker flashes Attacking,
/// defender flashes Hit). Returns the actual post-Defense damage dealt -
/// callers should use this, not `amount`, when building a message.
pub fn strike(ecs: &mut World, battle: &mut Battle, attacker: Combatant, amount: i32) -> i32 {
    match attacker {
        Combatant::Player => {
            let dmg = apply_damage(ecs, battle.enemy, amount);
            battle.show_enemy_damage(dmg);
            battle.player_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            dmg
        }
        Combatant::Enemy => {
            let dmg = apply_damage(ecs, battle.player, amount);
            battle.show_player_damage(dmg);
            battle.enemy_flash = Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
            battle.player_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
            dmg
        }
    }
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

/// TechniqueEffect::DamageMultiplier - attack right now for `multiplier`x
/// normal attack damage.
pub fn damage_multiplier(ecs: &mut World, battle: &mut Battle, multiplier: i32) -> String {
    let amount = player_attack_damage(ecs, battle.player) * multiplier;
    strike_message(strike(ecs, battle, Combatant::Player, amount))
}

/// TechniqueEffect::FlatDamage - attack for a fixed `amount` plus any
/// carried weapon damage (unlike DamageMultiplier, which scales off the
/// normal attack rather than adding a flat base).
pub fn flat_damage(ecs: &mut World, battle: &mut Battle, amount: i32) -> String {
    let total = amount + carried_weapon_damage(ecs, battle.player);
    strike_message(strike(ecs, battle, Combatant::Player, total))
}

/// TechniqueEffect::MultiHit - attack `hits` times in a row, each for
/// full normal attack damage. Only the last hit's damage gets a
/// popup/log line - showing every hit would need a queue of popups
/// rather than one slot, which is more than this technique needs.
pub fn multi_hit(ecs: &mut World, battle: &mut Battle, hits: i32) -> String {
    let raw = player_attack_damage(ecs, battle.player);
    let mut dmg = 0;
    for _ in 0..hits {
        dmg = strike(ecs, battle, Combatant::Player, raw);
    }
    strike_message(dmg)
}
