use crate::prelude::*;

// --- Buffs -------------------------------------------------------------
//
// A temporary boost to some player-side stat - distinct from a reactive
// effect like Counter, which sits dormant until a specific event (taking
// a hit) and then does something extra. A Buff just passively modifies a
// stat for as long as it's active, no trigger needed.
//
// Both of the game's current buffs go through this one module: Rogue's
// Dodge (an Evasion buff) and Amazon's Battle Cry (a DamageReduction
// buff) - previously two separate modules (evade.rs/warcry.rs) with
// near-identical apply/tick logic and no shared name for "this is a
// buff." A future Attack/Defense/Speed buff is a new BuffKind variant
// plus wiring it into wherever that stat gets read - not a new module.

/// Which stat a Buff modifies. Add a variant here (and wire it into
/// wherever that stat is read/applied - see resolve_enemy_attack for the
/// two current examples) for a new buff type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuffKind {
    /// Added to the player's base Evasion chance - see
    /// resolve_enemy_attack. Rogue's Dodge.
    Evasion,
    /// Subtracted from the enemy's damage on each hit that connects - see
    /// resolve_enemy_attack. Amazon's Battle Cry.
    DamageReduction,
}

/// How much a Buff is worth - either a fixed amount every time it's
/// read (Evasion), or a random amount rolled fresh each time it applies
/// (DamageReduction, per Amazon's original "deliberately random" design).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Magnitude {
    Flat(i32),
    Random { min: i32, max: i32 },
}

impl Magnitude {
    pub fn roll(&self) -> i32 {
        match self {
            Magnitude::Flat(v) => *v,
            Magnitude::Random { min, max } => RandomNumberGenerator::new().range(*min, *max + 1),
        }
    }
}

/// TechniqueEffect::Evade / TechniqueEffect::WarCry both funnel here now -
/// grants (or refreshes) a Buff of `kind` for `duration` more of whichever
/// event ticks it down. Which event that is depends on `kind`: Evasion
/// ticks on every attack faced (see tick_on_attack_faced), DamageReduction
/// only on a landed hit (see tick_and_reduce) - the buff doesn't need to
/// know which; whoever reads it calls the matching tick function.
pub fn apply(battle: &mut Battle, kind: BuffKind, magnitude: Magnitude, duration: i32) {
    battle.player_statuses.set(ActiveStatus::Buff {
        kind,
        magnitude,
        remaining: duration,
    });
}

/// The current flat value of an active Buff of `kind`, or 0 if none is
/// active or it isn't a Flat-magnitude buff. Reading a "current value"
/// for a Random-magnitude buff wouldn't mean anything since it rerolls on
/// use - see tick_and_reduce for those instead.
pub fn flat_value(battle: &Battle, kind: BuffKind) -> i32 {
    match battle.player_statuses.get(StatusKind::Buff(kind)) {
        Some(ActiveStatus::Buff {
            magnitude: Magnitude::Flat(v),
            ..
        }) => *v,
        _ => 0,
    }
}

/// Ticks a Buff of `kind` down by one, clearing it once exhausted. For
/// buffs that tick once per enemy attack FACED (currently just Evasion),
/// regardless of that attack's outcome.
pub fn tick_on_attack_faced(battle: &mut Battle, kind: BuffKind) {
    let expired = match battle.player_statuses.get_mut(StatusKind::Buff(kind)) {
        Some(ActiveStatus::Buff { remaining, .. }) => {
            *remaining -= 1;
            *remaining <= 0
        }
        _ => return,
    };
    if expired {
        battle.player_statuses.clear(StatusKind::Buff(kind));
    }
}

/// If a Buff of `kind` is active, rolls its magnitude, subtracts it from
/// `dmg` (floored at 0), and ticks its charge count down (clearing it
/// once exhausted). For buffs that only tick when an attack actually
/// LANDS (currently just DamageReduction) - a fully evaded attack didn't
/// deal damage to reduce, so it shouldn't spend a charge either. Returns
/// `dmg` unchanged if no Buff of `kind` is active.
pub fn tick_and_reduce(battle: &mut Battle, kind: BuffKind, dmg: i32) -> i32 {
    let magnitude = match battle.player_statuses.get(StatusKind::Buff(kind)) {
        Some(ActiveStatus::Buff { magnitude, .. }) => *magnitude,
        _ => return dmg,
    };
    let reduced = (dmg - magnitude.roll()).max(0);

    let expired = match battle.player_statuses.get_mut(StatusKind::Buff(kind)) {
        Some(ActiveStatus::Buff { remaining, .. }) => {
            *remaining -= 1;
            *remaining <= 0
        }
        _ => false,
    };
    if expired {
        battle.player_statuses.clear(StatusKind::Buff(kind));
    }

    reduced
}

/// The remaining duration of an active Buff of `kind`, if any - used for
/// status-line display (see screens/battle.rs).
pub fn remaining(battle: &Battle, kind: BuffKind) -> Option<i32> {
    match battle.player_statuses.get(StatusKind::Buff(kind)) {
        Some(ActiveStatus::Buff { remaining, .. }) => Some(*remaining),
        _ => None,
    }
}
