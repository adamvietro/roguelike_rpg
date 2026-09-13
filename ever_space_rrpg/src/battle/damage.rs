use crate::prelude::*;
use std::collections::VecDeque;

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
///
/// Both flashes last as long as `battle.player_action_animation`'s own
/// real Attack/Technique swing (see `OneShotAnimation::total_duration_ms`)
/// when one is currently set, rather than the shorter fixed
/// `PORTRAIT_FLASH_DURATION_MS` - added 2026-09-11 once Attack got a real
/// played-once animation of its own: a fixed-length flash used to fade
/// out well before a ~700ms swing actually finished, showing the target
/// (the enemy) as flinching red for only the first fraction of the hit,
/// then reverting to its normal color mid-swing. Falls back to the fixed
/// duration for anything with no matching animation (there's currently
/// nothing in that state, but this keeps the fallback path this whole
/// system has always had rather than assuming every future action will
/// get real art immediately).
pub fn strike_enemy(ecs: &mut World, battle: &mut Battle, target: Entity, amount: i32) -> i32 {
    let dmg = apply_damage(ecs, target, amount);
    battle.show_enemy_damage(target, dmg);
    let flash_ms = battle
        .player_action_animation
        .as_ref()
        .map(OneShotAnimation::total_duration_ms)
        .unwrap_or(PORTRAIT_FLASH_DURATION_MS);
    battle.player_flash = Some((FlashKind::Attacking, flash_ms));
    battle.set_enemy_flash_for(target, FlashKind::Hit, flash_ms);
    dmg
}

/// Deals `amount` damage from `attacker` (a specific enemy in this
/// battle) to the PLAYER, arms the player's damage popup, and flashes
/// both portraits. Same animation-length sync as `strike_enemy` above,
/// keyed off `attacker`'s own `EnemyCombatant::attack_animation` instead
/// of the player's.
pub fn strike_player(ecs: &mut World, battle: &mut Battle, attacker: Entity, amount: i32) -> i32 {
    let dmg = apply_damage(ecs, battle.player, amount);
    battle.show_player_damage(dmg);
    let flash_ms = battle
        .enemy(attacker)
        .and_then(|e| e.attack_animation.as_ref())
        .map(OneShotAnimation::total_duration_ms)
        .unwrap_or(PORTRAIT_FLASH_DURATION_MS);
    battle.set_enemy_flash_for(attacker, FlashKind::Attacking, flash_ms);
    battle.player_flash = Some((FlashKind::Hit, flash_ms));
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

// --- Multi-hit sequencing ----------------------------------------------
//
// Both MultiHit and AoeMultiHit used to land every hit in one synchronous
// loop, all within the same instant. Damage/HP were always applied
// correctly, but the popup only has one slot per combatant (see
// EnemyCombatant::damage_popup) - each hit's popup silently overwrote the
// previous one before a single frame could ever render it, so only the
// LAST hit's number was ever visible. HitQueue fixes that by spacing the
// hits out over real time (HIT_QUEUE_INTERVAL_MS apart, ticked in
// screens/battle.rs's battle_tick via tick_hit_queue - the same "advance
// by real elapsed ms" shape every other timer in this project already
// uses), so each hit gets its own visible moment on screen.

/// How many real milliseconds apart each queued hit lands - see
/// HitQueue/tick_hit_queue. Deliberately similar in feel to
/// PORTRAIT_FLASH_DURATION_MS (150ms) so a hit's flash/popup roughly
/// finishes before the next one begins, rather than overlapping.
pub const HIT_QUEUE_INTERVAL_MS: f32 = 150.0;

/// A multi-hit technique's still-pending hits, landing one at a time -
/// see Battle::hit_queue and tick_hit_queue. Lives on Battle (not a local
/// variable) because it has to survive across multiple battle_tick
/// frames, same reasoning as every other timed field on Battle
/// (flashes, popups, result_timer_ms).
#[derive(Clone, Debug, PartialEq)]
pub struct HitQueue {
    /// (target, per-hit amount) pairs still waiting to land, in order.
    pub remaining: VecDeque<(Entity, i32)>,
    /// Counts down to the next landed hit; resets to
    /// HIT_QUEUE_INTERVAL_MS each time one lands.
    pub timer_ms: f32,
    /// Running total of actual (post-Defense) damage dealt so far -
    /// used to build the final summary line once the queue drains.
    pub total_dealt: i32,
    /// How many landed hits actually dealt non-zero damage (a full dodge
    /// still counts as "a hit," just for 0).
    pub hits_dealt: i32,
    /// How many hits this queue started with in total, for the message.
    pub total_hits: i32,
    /// How many distinct enemies were targeted - lets the final message
    /// read "Hit all N enemies" (AoeMultiHit) vs. plain "Hit X times"
    /// (single-target MultiHit), matching the old wording exactly.
    pub enemy_count: usize,
}

/// Ticks `battle`'s in-progress HitQueue (if any) by `frame_time_ms` -
/// called unconditionally every battle_tick frame, the same way flash/
/// popup timers already are, so it keeps landing hits regardless of
/// which BattleTurn the screen happens to be showing. Once the last hit
/// lands, pushes the final summary log line and restarts
/// result_timer_ms - the ActionResult auto-advance clock was already
/// counting down from the moment the technique was chosen (see
/// screens/battle.rs's resolve_player_action), so this gives the player
/// a full, undiminished look at the finished summary instead of letting
/// that countdown run out silently while hits were still landing.
pub fn tick_hit_queue(ecs: &mut World, battle: &mut Battle, frame_time_ms: f32) {
    let mut queue = match battle.hit_queue.take() {
        Some(q) => q,
        None => return,
    };

    queue.timer_ms -= frame_time_ms;
    if queue.timer_ms <= 0.0 {
        queue.timer_ms += HIT_QUEUE_INTERVAL_MS;
        if let Some((target, amount)) = queue.remaining.pop_front() {
            let dmg = strike_enemy(ecs, battle, target, amount);
            queue.total_dealt += dmg;
            if dmg > 0 {
                queue.hits_dealt += 1;
            }
        }
    }

    if queue.remaining.is_empty() {
        battle.push_log(hit_queue_summary(&queue));
        battle.result_timer_ms = RESULT_AUTO_ADVANCE_MS;
    } else {
        battle.hit_queue = Some(queue);
    }
}

/// The final "Hit N times for M damage" summary line, once every queued
/// hit has landed - see tick_hit_queue and start_hit_queue (the
/// single-hit edge case resolves this immediately instead of queuing).
fn hit_queue_summary(queue: &HitQueue) -> String {
    if queue.enemy_count <= 1 {
        format!(
            "Hit {} times for {} total damage.",
            queue.total_hits, queue.total_dealt
        )
    } else {
        format!(
            "Hit all {} enemies {} times for {} total damage.",
            queue.enemy_count, queue.total_hits, queue.total_dealt
        )
    }
}

/// Shared by multi_hit/aoe_multi_hit: lands the FIRST queued hit right
/// now (so there's no dead-air delay before anything visibly happens),
/// then hands the rest to Battle::hit_queue for tick_hit_queue to land
/// one at a time. If that first hit was the only one (hits == 1), there's
/// nothing left to queue - the summary line is returned immediately
/// instead, same as any other instant technique.
fn start_hit_queue(
    ecs: &mut World,
    battle: &mut Battle,
    mut remaining: VecDeque<(Entity, i32)>,
    total_hits: i32,
    enemy_count: usize,
) -> String {
    let mut queue = HitQueue {
        remaining: VecDeque::new(),
        timer_ms: HIT_QUEUE_INTERVAL_MS,
        total_dealt: 0,
        hits_dealt: 0,
        total_hits,
        enemy_count,
    };
    if let Some((target, amount)) = remaining.pop_front() {
        let dmg = strike_enemy(ecs, battle, target, amount);
        queue.total_dealt += dmg;
        if dmg > 0 {
            queue.hits_dealt += 1;
        }
    }
    queue.remaining = remaining;

    if queue.remaining.is_empty() {
        hit_queue_summary(&queue)
    } else {
        battle.hit_queue = Some(queue);
        // The real summary isn't known until every hit has landed -
        // tick_hit_queue pushes it then. push_log already ignores empty
        // lines, so returning "" here is a clean no-op for the caller.
        String::new()
    }
}

/// TechniqueEffect::MultiHit - attack `target` `hits` times in a row,
/// each for full normal attack damage, spaced out via HitQueue so each
/// hit's popup is actually visible (see this module's doc comment).
pub fn multi_hit(ecs: &mut World, battle: &mut Battle, target: Entity, hits: i32) -> String {
    let raw = player_attack_damage(ecs, battle.player);
    let remaining: VecDeque<(Entity, i32)> = (0..hits).map(|_| (target, raw)).collect();
    start_hit_queue(ecs, battle, remaining, hits, 1)
}

/// TechniqueEffect::AoeMultiHit - strikes EVERY enemy currently in the
/// battle, `hits` times each, for (normal attack damage + `bonus_damage`)
/// per hit, spaced out via HitQueue (see this module's doc comment). The
/// first technique that isn't single-target, now that a battle can hold
/// more than one enemy (see battle::MAX_BATTLE_ENEMIES) - every other
/// technique in the game still only ever touches whichever one enemy
/// Battle::primary_target picks.
///
/// Deliberately doesn't skip an enemy that a previous hit in this same
/// sequence already dropped to 0 or below - death isn't checked/handled
/// until the result screen dismisses (see screens/battle.rs's
/// dismiss_action_result), same as every other technique in this game;
/// an AOE landing "extra" hits on an already-doomed enemy is harmless and
/// avoids needing an early-exit death check here that nothing else in
/// the codebase does either.
pub fn aoe_multi_hit(ecs: &mut World, battle: &mut Battle, hits: i32, bonus_damage: i32) -> String {
    let per_hit = player_attack_damage(ecs, battle.player) + bonus_damage;
    let targets: Vec<Entity> = battle.enemies.iter().map(|e| e.entity).collect();
    let enemy_count = targets.len();

    let mut remaining: VecDeque<(Entity, i32)> = VecDeque::new();
    for _ in 0..hits {
        for &target in &targets {
            remaining.push_back((target, per_hit));
        }
    }
    start_hit_queue(ecs, battle, remaining, hits, enemy_count)
}
