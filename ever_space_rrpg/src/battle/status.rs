use crate::battle::buff::{BuffKind, Magnitude};

// --- Unified battle status storage ------------------------------------------
//
// Before this refactor, every ongoing/one-shot battle status was its own
// bespoke `Option<SomeState>` field bolted directly onto Battle
// (`enemy_stunned: Option<StunState>`, `enemy_dot: Option<DotState>`,
// `dodge_bonus: Option<DodgeState>`, `war_cry: Option<WarCryState>`,
// `countering: Option<CounterState>`). Adding a genuinely new status meant
// a new struct, a new Battle field, and hand-threading its tick/consume
// logic into resolve_enemy_attack by position.
//
// Now every status is one variant of ActiveStatus, held in one of two
// StatusSet containers on Battle (player_statuses / enemy_statuses,
// depending which side the effect targets). A new status type is one new
// variant here plus one module (see battle::stun, battle::dot, etc.) that
// knows how to apply/tick/consume it - Battle itself never needs a new
// field again.

/// One kind of active battle status, with its own payload. See the
/// module-level comment above for why this replaced five separate
/// bespoke structs.
#[derive(Clone, Debug, PartialEq)]
pub enum ActiveStatus {
    /// Enemy can't act for `turns_remaining` more of its turns - see
    /// battle::stun.
    Stun { turns_remaining: i32 },
    /// Enemy takes `damage` at the start of each of the next
    /// `turns_remaining` rounds - see battle::dot.
    Dot {
        damage: i32,
        turns_remaining: i32,
        /// The technique's own name, lowercased, for the per-tick
        /// message (e.g. "the garrote bites...").
        label: String,
    },
    /// Player's active buffs (Evasion, DamageReduction, and any future
    /// stat buff) are one Buff variant now, parameterized by `kind` and
    /// `magnitude` instead of a separate ActiveStatus variant per stat -
    /// see battle::buff.
    Buff {
        kind: BuffKind,
        magnitude: Magnitude,
        /// Counts down on whichever event that buff's kind ticks on -
        /// either every attack faced or only attacks that land, see
        /// battle::buff::tick_on_attack_faced / tick_and_reduce.
        remaining: i32,
    },
    /// The next hit the player takes has `chance_percent` chance to
    /// reflect `multiplier`x normal attack damage back at the enemy - see
    /// battle::counter.
    Counter {
        chance_percent: i32,
        multiplier: i32,
    },
}

/// Which variant of ActiveStatus an instance is, independent of its
/// payload - lets StatusSet check/replace/remove "the active Stun, if
/// any" without matching out the payload just to test presence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Stun,
    Dot,
    /// Carries which stat the buff affects, so an Evasion buff and a
    /// DamageReduction buff can be active at the same time - each is its
    /// own StatusKind despite sharing the ActiveStatus::Buff variant.
    Buff(BuffKind),
    Counter,
}

impl ActiveStatus {
    pub fn kind(&self) -> StatusKind {
        match self {
            ActiveStatus::Stun { .. } => StatusKind::Stun,
            ActiveStatus::Dot { .. } => StatusKind::Dot,
            ActiveStatus::Buff { kind, .. } => StatusKind::Buff(*kind),
            ActiveStatus::Counter { .. } => StatusKind::Counter,
        }
    }
}

/// A small set of active statuses, at most one instance of each
/// StatusKind at a time - setting a status that's already active just
/// overwrites it with a fresh instance. This is the same "just replace"
/// refresh behavior every timed status in this project already used
/// individually before this refactor (Ice Armor, Invisible, Stealthed
/// all still work this way as ECS components), now made generic. A plain
/// Vec + linear scan is simpler than a HashMap for a list this short (5
/// possible kinds total, almost always 0-2 active at once).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatusSet {
    active: Vec<ActiveStatus>,
}

impl StatusSet {
    /// Activates `status`, replacing any existing instance of the same
    /// kind.
    pub fn set(&mut self, status: ActiveStatus) {
        let kind = status.kind();
        self.active.retain(|s| s.kind() != kind);
        self.active.push(status);
    }

    pub fn get(&self, kind: StatusKind) -> Option<&ActiveStatus> {
        self.active.iter().find(|s| s.kind() == kind)
    }

    pub fn get_mut(&mut self, kind: StatusKind) -> Option<&mut ActiveStatus> {
        self.active.iter_mut().find(|s| s.kind() == kind)
    }

    pub fn is_active(&self, kind: StatusKind) -> bool {
        self.get(kind).is_some()
    }

    /// Removes and returns the active instance of `kind`, if any - used
    /// for one-shot consumption (e.g. Counter resolving on a landed hit).
    pub fn take(&mut self, kind: StatusKind) -> Option<ActiveStatus> {
        let idx = self.active.iter().position(|s| s.kind() == kind)?;
        Some(self.active.remove(idx))
    }

    pub fn clear(&mut self, kind: StatusKind) {
        self.active.retain(|s| s.kind() != kind);
    }
}
