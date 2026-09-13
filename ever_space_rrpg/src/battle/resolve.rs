use crate::prelude::*;
use crate::screens::battle::enemy_portrait_position;
use crate::State;

/// Outcome of State::dismiss_action_result - whether battle_tick should
/// keep going this frame (back to Filling for a fresh race) or stop
/// immediately because something already fully resolved the resources
/// (fled/GameOver/BattleVictory all replace `Option<Battle>` themselves).
pub(crate) enum ResultOutcome {
    Continue,
    EndBattleTick,
}

impl State {
    /// Records one enemy's death: stats, loot/gold (accumulated onto
    /// `battle` - see its gold_earned/loot_found/defeated_names fields),
    /// removes it from the ECS and from `battle.enemies`. If that was the
    /// last enemy in the fight, finishes the whole battle (BattleVictory)
    /// and returns true - the caller should stop touching `battle`
    /// immediately, same contract the old finish_battle_victory had.
    /// Returns false if the fight continues (other enemies remain).
    /// `enter_held` is whether Enter is the key currently down this frame
    /// (see battle_tick) - threaded all the way down here purely so
    /// finish_battle can arm pending_enter_release on the exact frame a
    /// kill actually transitions into BattleVictory, regardless of which
    /// of several call paths (a plain Attack, a Technique, a DoT tick, a
    /// Counter) led to this kill.
    pub(crate) fn record_enemy_kill(&mut self, battle: &mut Battle, target: Entity, enter_held: bool) -> bool {
        let target_name = battle
            .enemy(target)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "enemy".to_string());

        if let Some(class) = entity_class(&self.ecs, battle.player) {
            if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                stats.record_enemy_killed(&class);
            }
        }

        // Read the enemy's Boss tag and the player's current Gold total
        // BEFORE the enemy is removed below - Gold's own presence (not a
        // separate Option<ArenaRun> check) is what decides whether this
        // is a Battle Arena kill at all, per that component's own doc
        // comment. Loot is rolled here too (before the CommandBuffer
        // below exists) since grant_random_battle_loot also needs
        // `&mut self.ecs` directly - same ordering finish_battle_victory
        // always used, for the same reason.
        let is_boss = self
            .ecs
            .entry_ref(target)
            .map(|e| e.get_component::<Boss>().is_ok())
            .unwrap_or(false);
        let current_gold = self
            .ecs
            .entry_ref(battle.player)
            .ok()
            .and_then(|e| e.get_component::<Gold>().ok().copied());

        // In the Battle Arena, a kill's reward is gold ONLY - the old
        // random ability-drop loot is deliberately not granted alongside
        // it. A Dungeon Crawl kill still rolls for loot exactly as
        // before - checked via Option<ArenaRun>, NOT Gold presence, since
        // Dungeon Crawl players now carry a Gold component too (see its
        // own doc comment) and would otherwise always skip this roll.
        let is_arena_run = self.resources.get::<Option<ArenaRun>>().unwrap().is_some();
        let mut rng = RandomNumberGenerator::new();
        let loot = if is_arena_run {
            None
        } else {
            grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player, target)
        };
        if let Some(item) = loot {
            battle.loot_found.push(item);
        }

        // A boss's own death animation (see components::
        // death_animation_for_enemy - `None` for a basic enemy, which
        // still just vanishes below exactly as before) - captured here,
        // BEFORE removal, as a pure decorative Battle::dying_effects
        // entry rather than delaying anything below it. See that field's
        // own doc comment for why this doesn't block rewards/removal/the
        // fight-over check the way keeping the enemy "alive" in
        // `battle.enemies` until the animation finished would have.
        if let Some(animation) = death_animation_for_enemy(&target_name) {
            let color = entity_render_component(&self.ecs, target)
                .map(|r| r.color)
                .unwrap_or(ColorPair::new(WHITE, BLACK));
            let index = battle.enemies.iter().position(|e| e.entity == target);
            if let Some(index) = index {
                let formation_rows = self
                    .resources
                    .get::<Box<dyn MapTheme>>()
                    .unwrap()
                    .enemy_formation_rows();
                let (col, row) =
                    enemy_portrait_position(battle.enemies.len(), index, formation_rows);
                battle.dying_effects.push(DyingEnemyEffect {
                    col,
                    row,
                    color,
                    animation,
                });
            }
        }

        let mut cb = CommandBuffer::new(&mut self.ecs);
        if let Some(Gold(amount)) = current_gold {
            let reward = gold_reward_for_kill(is_boss);
            cb.add_component(battle.player, Gold(amount + reward));
            battle.gold_earned += reward;
        }
        cb.remove(target);
        cb.flush(&mut self.ecs);

        battle.push_log(format!("Defeated the {}!", target_name));
        battle.defeated_names.push(format!("the {}", target_name));
        battle.enemies.retain(|e| e.entity != target);

        if battle.enemies.is_empty() {
            self.finish_battle(battle, enter_held);
            true
        } else {
            false
        }
    }

    /// Ends the whole battle in victory, using whatever's accumulated on
    /// `battle` across every kill this fight (see record_enemy_kill) -
    /// called the instant `battle.enemies` becomes empty. `enter_held` -
    /// see this same parameter's doc comment on record_enemy_kill/
    /// battle_tick - arms pending_enter_release when true, so the exact
    /// same held Enter that landed this killing blow can't ALSO
    /// immediately dismiss the Victory screen this transitions into.
    pub(crate) fn finish_battle(&mut self, battle: &Battle, enter_held: bool) {
        // Checked fresh here (not just "was gold_earned > 0") so a
        // Battle Arena fight that somehow ended with 0 net gold (it
        // shouldn't - every kill grants at least a few) still correctly
        // shows "0 gold" rather than being mistaken for a Dungeon Crawl
        // fight with no loot. Uses Option<ArenaRun>, NOT Gold presence -
        // Dungeon Crawl players carry Gold too now (see its own doc
        // comment), so that check would otherwise always read "arena"
        // and hide this fight's real ability-loot find behind a
        // "gold_earned" the Victory screen never announces for Dungeon
        // Crawl anyway (see match arm below).
        let is_arena = self.resources.get::<Option<ArenaRun>>().unwrap().is_some();

        let portrait_animation = entity_class(&self.ecs, battle.player)
            .and_then(|class| victory_animation_for_class(&class));
        self.resources.insert(Some(BattleVictory {
            player: battle.player,
            enemy_names: battle.defeated_names.clone(),
            loot: battle.loot_found.clone(),
            gold_earned: if is_arena {
                Some(battle.gold_earned)
            } else {
                None
            },
            portrait_animation,
        }));
        self.resources.insert(None::<Battle>);
        self.resources.insert(TurnState::BattleVictory);
        // See this fn's own doc comment - guards against the same held
        // Enter that fired the killing action also immediately dismissing
        // the screen it just transitioned into.
        if enter_held {
            self.pending_enter_release = true;
        }
    }

    /// Fires `attacker`'s attack right now: ticks its own active Dot
    /// first (a lingering wound, not an action in its own right), checks
    /// whether that alone finished it off, then actually resolves the
    /// attack and enters ActionResult(Enemy(attacker)). Shared by two
    /// triggers: the normal "this enemy's gauge just reached
    /// ATB_GAUGE_MAX while nothing else was in progress" case (Filling),
    /// and - under True ATB (AtbMode::Active) only - an enemy's gauge
    /// filling WHILE the player is still stuck in their own menu (see
    /// BattleTurn::PlayerMenu's handling below) - "decide fast or take
    /// the hit" is the entire point of that mode. Returns true if
    /// `attacker` died to its own Dot tick before it could even swing (or
    /// that was the last enemy standing) - record_enemy_kill has already
    /// been called in that case, possibly ending the whole battle; the
    /// caller should return from battle_tick immediately without
    /// touching `battle` again.
    pub(crate) fn trigger_enemy_action(
        &mut self,
        battle: &mut Battle,
        attacker: Entity,
        enter_held: bool,
    ) -> bool {
        let dot_message = tick_dot(&mut self.ecs, battle, attacker);
        if let Some(dot_message) = dot_message {
            battle.push_log(dot_message);
        }

        let (hp_now, _) = entity_health(&self.ecs, attacker);
        if hp_now < 1 {
            return self.record_enemy_kill(battle, attacker, enter_held);
        }

        if let Some(enemy) = battle.enemy_mut(attacker) {
            enemy.attack_animation = attack_animation_for_enemy(&enemy.name);
        }
        resolve_enemy_attack(&mut self.ecs, battle, attacker);
        battle.enter_result(Combatant::Enemy(attacker));
        false
    }

    /// Resolves a player-chosen BattleAction: applies its effect, logs
    /// the result, clears the one-shot sneak_attack bonus, and enters
    /// ActionResult(Player). Shared by two callers: PlayerMenu's own
    /// immediate resolution (the common case - nothing else was
    /// happening) and Filling's queued-action resolution (True ATB only
    /// - see Battle::queued_player_action) for a choice made earlier,
    /// while some enemy's own result was still on screen. Both need the
    /// exact same effect logic; only WHEN it runs differs.
    ///
    /// Single-target actions (Attack, and every single-target Technique)
    /// hit whichever enemy Battle::primary_target currently picks - the
    /// player doesn't choose directly once there's more than one enemy.
    /// Recomputed fresh here rather than passed in, so a target chosen
    /// via queuing is still whoever's actually fastest-and-alive at the
    /// moment this finally runs, not whoever was fastest back when the
    /// player originally pressed the key.
    pub(crate) fn resolve_player_action(&mut self, battle: &mut Battle, chosen: BattleAction) {
        let target = battle.primary_target(&self.ecs);
        // Captured before the match below - BattleAction::Technique's own
        // branch removes the item entity from the ECS as part of
        // consuming it (via apply_player_technique), so its Name has to
        // be read before that happens, not after. Used only for
        // cross-battle cursor memory (MenuMemory) once resolution
        // finishes below - a separate, earlier read of the same
        // information Stats::record_ability_used's own inline capture
        // needs for the same reason.
        let action_name_for_memory = action_name(&self.ecs, chosen);
        // Every technique's mechanical effect is resolved in one place
        // (battle::apply_player_technique) rather than a match arm per
        // item here - adding a new class's technique needs no change here.
        match chosen {
            BattleAction::Attack => {
                if let Some(class) = entity_class(&self.ecs, battle.player) {
                    let anim = attack_animation_for_class(&class);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Attack);
                    battle.player_action_animation = anim;
                }
                if let Some(target) = target {
                    let mut dmg = player_attack_damage(&self.ecs, battle.player);
                    if battle.sneak_attack {
                        dmg *= 3;
                    }
                    let dmg = damage::strike_enemy(&mut self.ecs, battle, target, dmg);
                    battle.push_log(damage::strike_message(dmg));
                }
            }
            BattleAction::Defend => {
                if let Some(class) = entity_class(&self.ecs, battle.player) {
                    let anim = defend_animation_for_class(&class);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Defend);
                    battle.player_action_animation = anim;
                }
                battle.player_defending = true;
                battle.push_log("Defend.".to_string());
            }
            BattleAction::Flee => {
                battle.fled = true;
                battle.push_log("Flee.".to_string());
            }
            BattleAction::Technique(item) => {
                // Recorded BEFORE apply_player_technique runs - it
                // removes `item` from the ECS as part of consuming it,
                // so its Name/Class have to be read while it's still
                // there. Keyed off the item's own class, same reasoning
                // as Stats::record_ability_used's doc comment.
                if let Some(class) = entity_class(&self.ecs, item) {
                    let name = entity_name(&self.ecs, item);
                    if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                        stats.record_ability_used(&class, &name);
                    }
                    // A real animation for this specific (class,
                    // technique) pair, if one exists yet (see
                    // components::technique_animation_for) - None leaves
                    // draw_battle_arena showing the ordinary
                    // Fight_Stance_Idle loop instead, same as before this
                    // existed. Cleared in dismiss_action_result once turn
                    // returns to Filling. A multi-hit/AOE technique's own
                    // HitQueue (battle::damage) keeps landing damage over
                    // a real span of time far longer than one play-
                    // through of the animation, so it loops instead of
                    // holding on its last frame for most of that span -
                    // see OneShotAnimation::repeat's own doc comment.
                    let repeats = matches!(
                        technique_effect(&self.ecs, item),
                        Some(TechniqueEffect::MultiHit(_))
                            | Some(TechniqueEffect::AoeMultiHit { .. })
                    );
                    let anim = technique_animation_for(&class, &name, repeats);
                    battle.player_action_kind =
                        anim.is_some().then(|| PlayerActionKind::Technique);
                    battle.player_action_animation = anim;
                }
                // A self-buff technique (Heal/Evade/WarCry/Counter) just
                // ignores `target` entirely inside apply_player_technique
                // - still needs SOME entity to satisfy the signature, so
                // this falls back to the player itself in the (should be
                // unreachable - see primary_target's own doc comment)
                // case there's no enemy to target at all.
                let result = apply_player_technique(
                    &mut self.ecs,
                    battle,
                    item,
                    target.unwrap_or(battle.player),
                );
                battle.push_log(result);
            }
        }
        // Sneak attack is a one-shot ambush bonus for the guaranteed
        // first action only - clear it here regardless of which action
        // was actually chosen, so it can never linger and apply again
        // later in the same fight.
        battle.sneak_attack = false;

        // Cross-battle cursor memory (MenuMemory) - remember whatever was
        // just chosen, per class, so a FUTURE battle's cursor can start
        // there (see MenuCursor's seeding in battle_tick above). Recorded
        // here rather than at each individual call site (PlayerMenu's
        // direct resolution AND Filling's queued-action resolution both
        // funnel through this one function) so neither path can forget
        // it. Uses action_name_for_memory, captured above BEFORE the
        // match ran (see that binding's own comment for why).
        if *self.resources.get::<MenuMemory>().unwrap() == MenuMemory::On {
            if let Some(class) = entity_class(&self.ecs, battle.player) {
                let mut memory = self.resources.get_mut::<LastBattleAction>().unwrap();
                memory.record(&class, &action_name_for_memory);
                let saved = memory.clone();
                drop(memory);
                saved.save();
            }
        }

        battle.enter_result(Combatant::Player);
    }

    /// Outcome of dismiss_action_result - see that function. `enter_held`
    /// - see battle_tick's own doc note on this parameter - is threaded
    /// through so BOTH exit paths that can lead somewhere Enter might
    /// also dismiss (GameOver here directly; BattleVictory via
    /// record_enemy_kill/finish_battle) can arm pending_enter_release.
    pub(crate) fn dismiss_action_result(&mut self, battle: &mut Battle, enter_held: bool) -> ResultOutcome {
        if battle.fled {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::AwaitingInput);
            return ResultOutcome::EndBattleTick;
        }

        let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
        if player_hp_now < 1 {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::GameOver);
            // Same guard as finish_battle - the player's own held Enter
            // (from an earlier action, or from an enemy's ActionResult
            // dismiss) shouldn't also instantly dismiss Game Over.
            if enter_held {
                self.pending_enter_release = true;
            }
            return ResultOutcome::EndBattleTick;
        }

        // Sweep every remaining enemy for death rather than checking just
        // one specific entity - a Counter Attack can kill the enemy that
        // just attacked as a side effect of ITS OWN attack, the player's
        // own action might have killed whichever enemy it targeted, and
        // (once AOE exists) more than one could die from the same action
        // at once. record_enemy_kill on one enemy can end the whole
        // battle (all enemies gone) - stop immediately if so, same
        // contract trigger_enemy_action's own caller already follows.
        let dead: Vec<Entity> = battle
            .enemies
            .iter()
            .filter(|e| entity_health(&self.ecs, e.entity).0 < 1)
            .map(|e| e.entity)
            .collect();
        for target in dead {
            if self.record_enemy_kill(battle, target, enter_held) {
                return ResultOutcome::EndBattleTick;
            }
        }

        battle.turn = BattleTurn::Filling;
        // Whatever action animation (Attack/Defend/Technique) was playing
        // for the action just dismissed is done - clear it so
        // draw_battle_arena falls back to the ordinary Fight_Stance_Idle
        // loop for the next race, rather than holding on its last frame
        // indefinitely. Same for every enemy's own attack animation.
        //
        // player_flash/enemy.flash are cleared here too, not just left to
        // decay on their own timer - a real bug found 2026-09-13: this
        // fires the instant ANY key is pressed (see battle_tick's own
        // `ctx.key.is_some() || battle.result_timer_ms <= 0.0` check), not
        // only once flash_ms's duration (deliberately synced to the
        // animation's own length - see strike_enemy/strike_player) has
        // actually run out. Dismissing early cleared the animation but
        // left an active Attacking flash behind with real time still on
        // it; the very next frame's draw_battle_arena call then saw NO
        // animation glyph (falling through to the ordinary idle-portrait
        // tier) but STILL an active Attacking flash, and that tier - unlike
        // the animation tier - applies the old pre-real-animation wiggle
        // for exactly that flash. Result: a brief, leftover wiggle on the
        // idle portrait right after dismissing an action, reproducible
        // only when a key happens to be pressed before the flash would
        // have decayed on its own - which is why it never fired
        // consistently. Clearing both together here keeps them in sync
        // regardless of how (or how early) the result was dismissed.
        battle.player_action_animation = None;
        battle.player_action_kind = None;
        battle.player_flash = None;
        for enemy in battle.enemies.iter_mut() {
            enemy.attack_animation = None;
            enemy.flash = None;
        }
        ResultOutcome::Continue
    }
}
