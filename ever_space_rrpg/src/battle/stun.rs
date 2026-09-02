use crate::prelude::*;

// --- Stun --------------------------------------------------------------------
//
// Skips a specific enemy's attack entirely for N of its own turns. Used
// by TechniqueEffect::Stun (a roll) and TechniqueEffect::Feint
// (guaranteed, but only for one turn) - both just arm the same
// ActiveStatus::Stun on `target`'s own StatusSet, they differ only in how
// it gets armed. Every function here takes an explicit `target` now that
// a battle can hold more than one enemy - stunning one doesn't touch any
// other enemy's own ability to act.

/// TechniqueEffect::Stun - rolls `chance_percent` right now; on success,
/// stuns `target` for `turns`. Returns the log message either way - the
/// technique is consumed regardless of the roll's outcome.
pub fn roll(battle: &mut Battle, target: Entity, chance_percent: i32, turns: i32) -> String {
    let mut rng = RandomNumberGenerator::new();
    if rng.range(0, 100) < chance_percent {
        if let Some(enemy) = battle.enemy_mut(target) {
            enemy.statuses.set(ActiveStatus::Stun {
                turns_remaining: turns,
            });
        }
        "Stun the enemy.".to_string()
    } else {
        "Stun fails.".to_string()
    }
}

/// TechniqueEffect::Feint - guaranteed (no roll), unlike Stun above, but
/// only for `target`'s very next single attack. Overwrites any Stun
/// already in progress on `target` rather than extending it - a rare
/// enough overlap that a fresh, shorter feint replacing a longer stun
/// isn't worth extra bookkeeping to prevent (same "just overwrite"
/// behavior StatusSet::set already gives every status).
pub fn feint(battle: &mut Battle, target: Entity) -> String {
    if let Some(enemy) = battle.enemy_mut(target) {
        enemy
            .statuses
            .set(ActiveStatus::Stun { turns_remaining: 1 });
    }
    "Feint - the enemy holds back its attack.".to_string()
}

/// If `target` is currently stunned, ticks it down (clearing it if this
/// was its last turn) and returns true, meaning: `target`'s attack is
/// skipped entirely this turn - no Defend/Ice Armor/Counter gets
/// consumed, the same treatment a full dodge already gets. Called first
/// thing in resolve_enemy_attack, ahead of even the Evasion check, since
/// this is "the enemy never swings" rather than "the enemy swings and
/// misses." False if `target` isn't (or is no longer) in this battle.
pub fn check_and_tick(battle: &mut Battle, target: Entity) -> bool {
    let enemy = match battle.enemy_mut(target) {
        Some(e) => e,
        None => return false,
    };
    let (was_active, expired) = match enemy.statuses.get_mut(StatusKind::Stun) {
        Some(ActiveStatus::Stun { turns_remaining }) => {
            *turns_remaining -= 1;
            (true, *turns_remaining <= 0)
        }
        _ => (false, false),
    };
    if expired {
        enemy.statuses.clear(StatusKind::Stun);
    }
    was_active
}
