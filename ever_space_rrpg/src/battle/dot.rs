use crate::prelude::*;

// --- Damage over time --------------------------------------------------------
//
// A wound that deals damage at the start of each round rather than all at
// once. Used by TechniqueEffect::DamageOverTime directly (Rend/Burn/
// Garrote) and by the second half of PoisonStrike (see battle::apply_player_technique).
// Every function here takes an explicit `target` Entity now that a
// battle can hold more than one enemy - you could have one enemy bleeding
// while another isn't, tracked entirely independently on each
// EnemyCombatant's own StatusSet.

/// Arms (or refreshes) a Dot on `target`.
pub fn apply(battle: &mut Battle, target: Entity, damage: i32, turns: i32, label: String) {
    if let Some(enemy) = battle.enemy_mut(target) {
        enemy.statuses.set(ActiveStatus::Dot {
            damage,
            turns_remaining: turns,
            label,
        });
    }
}

/// If a Dot is active on `target`, ticks it down by one and applies its
/// damage. Called once per enemy, right before that enemy would act (see
/// screens/battle.rs's battle_tick). Returns a message describing the
/// tick, or None if no Dot is active (or `target` is no longer in this
/// battle at all).
///
/// Deliberately doesn't go through damage::strike_enemy - a Dot ticking
/// isn't the player actively swinging, so only the target's own Hit flash
/// fires, not the player's Attacking flash strike_enemy would also set.
pub fn tick(ecs: &mut World, battle: &mut Battle, target: Entity) -> Option<String> {
    let (damage, turns_remaining, label) = match battle.enemy(target)?.statuses.get(StatusKind::Dot)
    {
        Some(ActiveStatus::Dot {
            damage,
            turns_remaining,
            label,
        }) => (*damage, *turns_remaining, label.clone()),
        _ => return None,
    };
    if turns_remaining <= 0 {
        battle.enemy_mut(target)?.statuses.clear(StatusKind::Dot);
        return None;
    }

    let dmg = apply_damage(ecs, target, damage);
    battle.show_enemy_damage(target, dmg);
    battle.set_enemy_flash(target, FlashKind::Hit);

    let remaining = turns_remaining - 1;
    let enemy = battle.enemy_mut(target)?;
    if remaining <= 0 {
        enemy.statuses.clear(StatusKind::Dot);
    } else {
        enemy.statuses.set(ActiveStatus::Dot {
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
