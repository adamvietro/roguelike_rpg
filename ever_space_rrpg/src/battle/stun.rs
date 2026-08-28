use crate::prelude::*;

// --- Stun --------------------------------------------------------------------
//
// Skips the enemy's attack entirely for N of its own turns. Used by
// TechniqueEffect::Stun (a roll) and TechniqueEffect::Feint (guaranteed,
// but only for one turn) - both just arm the same ActiveStatus::Stun,
// they differ only in how it gets armed.

/// TechniqueEffect::Stun - rolls `chance_percent` right now; on success,
/// stuns the enemy for `turns`. Returns the log message either way - the
/// technique is consumed regardless of the roll's outcome.
pub fn roll(battle: &mut Battle, chance_percent: i32, turns: i32) -> String {
    let mut rng = RandomNumberGenerator::new();
    if rng.range(0, 100) < chance_percent {
        battle.enemy_statuses.set(ActiveStatus::Stun {
            turns_remaining: turns,
        });
        "Stun the enemy.".to_string()
    } else {
        "Stun fails.".to_string()
    }
}

/// TechniqueEffect::Feint - guaranteed (no roll), unlike Stun above, but
/// only for the enemy's very next single attack. Overwrites any Stun
/// already in progress rather than extending it - a rare enough overlap
/// that a fresh, shorter feint replacing a longer stun isn't worth extra
/// bookkeeping to prevent (same "just overwrite" behavior StatusSet::set
/// already gives every status).
pub fn feint(battle: &mut Battle) -> String {
    battle
        .enemy_statuses
        .set(ActiveStatus::Stun { turns_remaining: 1 });
    "Feint - the enemy holds back its attack.".to_string()
}

/// If the enemy is currently stunned, ticks it down (clearing it if this
/// was its last turn) and returns true, meaning: the enemy's attack is
/// skipped entirely this turn - no Defend/Ice Armor/Counter gets
/// consumed, the same treatment a full dodge already gets. Called first
/// thing in resolve_enemy_attack, ahead of even the Evasion check, since
/// this is "the enemy never swings" rather than "the enemy swings and
/// misses."
pub fn check_and_tick(battle: &mut Battle) -> bool {
    let (was_active, expired) = match battle.enemy_statuses.get_mut(StatusKind::Stun) {
        Some(ActiveStatus::Stun { turns_remaining }) => {
            *turns_remaining -= 1;
            (true, *turns_remaining <= 0)
        }
        _ => (false, false),
    };
    if expired {
        battle.enemy_statuses.clear(StatusKind::Stun);
    }
    was_active
}
