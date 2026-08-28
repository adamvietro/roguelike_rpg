use crate::prelude::*;

// --- Damage over time --------------------------------------------------------
//
// A wound that deals damage at the start of each round rather than all at
// once. Used by TechniqueEffect::DamageOverTime directly (Rend/Burn/
// Garrote) and by the second half of PoisonStrike (see battle::apply_player_technique).

/// Arms (or refreshes) a Dot on the enemy.
pub fn apply(battle: &mut Battle, damage: i32, turns: i32, label: String) {
    battle.enemy_statuses.set(ActiveStatus::Dot {
        damage,
        turns_remaining: turns,
        label,
    });
}

/// If a Dot is active on the enemy, ticks it down by one and applies its
/// damage. Called once at the start of each round (see battle_tick in
/// screens/battle.rs). Returns a message describing the tick, or None if
/// no Dot is active.
///
/// Deliberately doesn't go through damage::strike - a Dot ticking isn't
/// the player actively swinging, so only the enemy's own Hit flash fires,
/// not the player's Attacking flash strike() would also set.
pub fn tick(ecs: &mut World, battle: &mut Battle) -> Option<String> {
    let (damage, turns_remaining, label) = match battle.enemy_statuses.get(StatusKind::Dot) {
        Some(ActiveStatus::Dot {
            damage,
            turns_remaining,
            label,
        }) => (*damage, *turns_remaining, label.clone()),
        _ => return None,
    };
    if turns_remaining <= 0 {
        battle.enemy_statuses.clear(StatusKind::Dot);
        return None;
    }

    let dmg = apply_damage(ecs, battle.enemy, damage);
    battle.show_enemy_damage(dmg);
    battle.enemy_flash = Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));

    let remaining = turns_remaining - 1;
    if remaining <= 0 {
        battle.enemy_statuses.clear(StatusKind::Dot);
    } else {
        battle.enemy_statuses.set(ActiveStatus::Dot {
            damage,
            turns_remaining: remaining,
            label,
        });
    }

    Some(if dmg == 0 {
        "Dodge attack.".to_string()
    } else {
        format!("Deal {} damage.", dmg)
    })
}
