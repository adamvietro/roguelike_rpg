use crate::prelude::*;

// --- Counter -------------------------------------------------------------
//
// Skip your attack this turn; the next hit the enemy lands on you has a
// chance to reflect damage back at them. Used by TechniqueEffect::Counter.

/// TechniqueEffect::Counter - arms a pending counter: the next hit the
/// player takes has `chance_percent` chance to reflect `multiplier`x
/// normal attack damage back at the enemy.
pub fn apply(battle: &mut Battle, chance_percent: i32, multiplier: i32) -> String {
    battle.player_statuses.set(ActiveStatus::Counter {
        chance_percent,
        multiplier,
    });
    "Ready counter.".to_string()
}

/// If a Counter is armed, consumes it and rolls it: on success, strikes
/// `attacker` back for `multiplier`x the player's normal attack damage and
/// pushes a log line; on failure, just pushes a miss line. Does nothing
/// if no Counter is armed. `attacker` is whichever enemy just landed the
/// hit that triggered this check - a Counter always reflects back at
/// THAT enemy specifically, not some other enemy in the same battle.
///
/// Called once per connected enemy hit, past both the stunned and evaded
/// early returns in resolve_enemy_attack - a Counter that's armed but
/// never actually gets hit stays armed for a future turn, matching the
/// original countering.take() placement.
pub fn resolve_on_hit(ecs: &mut World, battle: &mut Battle, attacker: Entity) {
    let (chance_percent, multiplier) = match battle.player_statuses.take(StatusKind::Counter) {
        Some(ActiveStatus::Counter {
            chance_percent,
            multiplier,
        }) => (chance_percent, multiplier),
        _ => return,
    };

    let mut rng = RandomNumberGenerator::new();
    if rng.range(0, 100) < chance_percent {
        let amount = player_attack_damage(ecs, battle.player) * multiplier;
        let dmg = damage::strike_enemy(ecs, battle, attacker, amount);
        battle.push_log(damage::strike_message(dmg));
    } else {
        battle.push_log("Miss counter.".to_string());
    }
}
