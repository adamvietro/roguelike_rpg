use crate::prelude::*;
use crate::State;

/// Outcome of State::dismiss_action_result - whether battle_tick should
/// keep going this frame (back to Filling for a fresh race) or stop
/// immediately because something already fully resolved the resources
/// (fled/GameOver/BattleVictory all replace `Option<Battle>` themselves).
enum ResultOutcome {
    Continue,
    EndBattleTick,
}

/// One enemy's resolved portrait info for draw_battle_arena - its Render
/// (looked up live from the ECS) and its own flash state (see
/// EnemyCombatant::flash). Bundled here rather than passed as two
/// parallel slices, which would need index-matching to stay correct.
struct EnemyPortrait {
    render: Render,
    flash: Option<(FlashKind, f32)>,
}

/// Fractional (col, row) position, in the coarse BATTLE_PORTRAIT_COLS x
/// BATTLE_PORTRAIT_ROWS grid, for enemy #`index` of `count` total - a
/// deliberate formation per count rather than a plain vertical stack
/// (which is what the old enemy_portrait_row did, and it visually
/// collided with the Actions box the moment 3+ enemies were actually on
/// screen - see the screenshot that prompted this rework). Whole-number
/// draw_portrait can't express most of these (a "1.2 spaces up" shift
/// isn't one of the 5 whole rows) - see render_helpers.rs's
/// draw_portrait_fancy/draw_wiggling_portrait, both fractional now.
///
/// The formation, one count at a time:
/// - 1: the classic single-enemy spot, unchanged - (3, 1).
/// - 2: side by side at (2, 1) and (4, 1) - straddling column 3, so the
///   midpoint between the two lands exactly on the classic spot.
/// - 3: that same side-by-side pair, shifted up to row 0.6 (staying
///   clear of row 0 itself - see enemy_text_position's own note on why
///   that row is treated as unsafe), plus a third enemy below and
///   centered at (3, 1.8) - a 1.2-row gap between the two tiers.
/// - 4: the count-3 formation plus a fourth enemy immediately to the
///   right of the bottom-center one, at (4, 1.8).
fn enemy_portrait_position(count: usize, index: usize) -> (f32, f32) {
    match (count, index) {
        (0, _) | (1, _) => (3.0, 1.0),
        (2, 0) => (2.0, 1.0),
        (2, _) => (4.0, 1.0),
        (_, 0) => (2.0, 0.6),
        (_, 1) => (4.0, 0.6),
        (_, 2) => (3.0, 1.8),
        _ => (4.0, 1.8),
    }
}

/// HUD_CONSOLE (col, row) for enemy #`index` (of `count`)'s own name/HP-
/// bar/ATB-bar/status text block, anchored directly BELOW that enemy's
/// own portrait (see enemy_portrait_position) instead of in one shared
/// column off to the side. A single shared column stopped making sense
/// once portraits spread across columns instead of stacking in one - it
/// used to land squarely on top of the Actions box, which sits in that
/// same horizontal territory (see the screenshot that prompted this
/// whole rework). Single-enemy keeps the exact original spot (col 96,
/// row 41) - completely unaffected by any of this.
///
/// NOTE: despite living in a variable named after HUD_CONSOLE
/// conventions elsewhere in this file, this text actually renders on
/// console 2 (see battle_tick's own `ctx.set_active_console(2)` right
/// before this block runs), NOT HUD_CONSOLE - the ratios below (32
/// columns and 20 rows per one coarse portrait-grid unit) are console
/// 2's own. Confirmed against the original single-enemy constants
/// themselves: portrait position (3, 1) gives col 3*32=96 and row
/// (1+1)*20+1=41, exactly matching the values already proven correct -
/// console 2 is a 160x100 grid over the same 1280x800 window, so each
/// portrait-grid unit (256x160px) is exactly 32 console-2 columns and 20
/// console-2 rows, no rounding even needed. An earlier version of this
/// function used HUD_CONSOLE's own ~21.4/~13.4 ratios by mistake, which
/// put every multi-enemy text block in the wrong place entirely. Row is
/// measured from the portrait's own BOTTOM edge (position + 1.0, since
/// every portrait is exactly one grid unit tall regardless of its
/// fractional top-left anchor), plus a 1-row margin so text starts just
/// past the sprite rather than flush against it.
fn enemy_text_position(count: usize, index: usize) -> (i32, i32) {
    if count <= 1 {
        return (96, 41);
    }
    let (col, row) = enemy_portrait_position(count, index);
    let console2_col = (col * 32.0).round() as i32;
    let console2_row = ((row + 1.0) * 20.0).round() as i32 + 1;
    (console2_col, console2_row)
}

/// BIG_TEXT_CONSOLE (col, row) to center a floating damage number over
/// enemy #`index` (of `count`) - see the floating-damage-number block in
/// battle_tick. Derived the same way the original single-enemy constants
/// (28, 7 - for portrait position (3, 1)) were: BIG_TEXT_CONSOLE and the
/// portrait console cover the same physical window, at ratios of 8
/// BIG_TEXT columns and 5 BIG_TEXT rows per one portrait-grid unit,
/// centered half a unit into whichever cell the portrait occupies.
fn enemy_damage_popup_position(count: usize, index: usize) -> (i32, i32) {
    let (col, row) = enemy_portrait_position(count, index);
    let big_col = (col * 8.0 + 4.0).round() as i32;
    let big_row = (row * 5.0 + 2.0).round() as i32;
    (big_col, big_row)
}

/// A natural-language join of names for the victory message - "the Goblin!",
/// "the Goblin and the Orc!", "the Goblin, the Orc, and the Rat!". Every
/// name already includes its own "the " (see how `enemy_names` is built
/// in record_enemy_kill) - this just handles the comma/and joinery.
fn join_enemy_names(names: &[String]) -> String {
    match names.len() {
        0 => String::new(),
        1 => names[0].clone(),
        2 => format!("{} and {}", names[0], names[1]),
        _ => {
            let (last, rest) = names.split_last().unwrap();
            format!("{}, and {}", rest.join(", "), last)
        }
    }
}

impl State {
    /// Draws the battle arena background (console 0: the current theme's
    /// floor/walls/scenery) and every combatant portrait (console 3) -
    /// the player plus up to MAX_BATTLE_ENEMIES enemies. Callers look up
    /// each Render live during an active battle, or pass captured values
    /// (battle_victory_tick, where every enemy has already been removed
    /// from the ECS - it always passes an empty `enemies` slice).
    fn draw_battle_arena(
        &mut self,
        enemies: &[EnemyPortrait],
        player_render: Option<Render>,
        player_flash: Option<(FlashKind, f32)>,
    ) {
        // --- Arena background: the current dungeon theme's floor/wall
        // tiles, tinted with that theme's palette and framed with a border,
        // plus a soft vignette that brightens toward the center (a
        // "clearing") and darkens toward the edges. Console 0 is otherwise
        // blank outside battle-related states, so this is free real estate.
        //
        // Cell backgrounds (not just the thin foreground glyph) carry the
        // tint, since a small character like '.' or ';' only covers a
        // fraction of a cell's pixels - foreground-only color reads as
        // scattered specks on black rather than an actual colored floor.
        {
            let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap();
            let floor_glyph = theme.tile_to_render(TileType::Floor);
            let wall_glyph = theme.tile_to_render(TileType::Wall);
            let floor_base = theme.floor_color();
            let wall_base = theme.wall_color();
            let scenery = theme.battle_scenery();
            drop(theme);

            let mut arena = DrawBatch::new();
            arena.target(0);
            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    let is_border =
                        x == 0 || y == 0 || x == DISPLAY_WIDTH - 1 || y == DISPLAY_HEIGHT - 1;
                    let (glyph, base) = if is_border {
                        (wall_glyph, wall_base)
                    } else {
                        (floor_glyph, floor_base)
                    };
                    let bg = vignette(base, x, y, DISPLAY_WIDTH, DISPLAY_HEIGHT);
                    let fg = RGB::from_f32(
                        (bg.r * 1.4).min(1.0),
                        (bg.g * 1.4).min(1.0),
                        (bg.b * 1.4).min(1.0),
                    );
                    arena.set(Point::new(x, y), ColorPair::new(fg, bg), glyph);
                }
            }

            match scenery {
                BattleScenery::ScatteredTrees => {
                    // A handful of large tree/foliage silhouettes,
                    // hand-placed clear of the portraits, their labels,
                    // and the message/menu panel. Drawn at full-strength
                    // color (not vignetted) so they read as distinct
                    // features wherever they land on the light/dark
                    // gradient above.
                    let canopy_color = RGB::from_f32(
                        (floor_base.r * 1.5).min(1.0),
                        (floor_base.g * 1.5).min(1.0),
                        (floor_base.b * 1.5).min(1.0),
                    );
                    let trunk_color =
                        RGB::from_f32(wall_base.r * 0.85, wall_base.g * 0.85, wall_base.b * 0.85);
                    for &(tx, ty) in &[(6, 3), (18, 3), (35, 15), (22, 19)] {
                        draw_tree(&mut arena, wall_glyph, canopy_color, trunk_color, tx, ty);
                    }
                }
                BattleScenery::RoomWalls => {
                    // Thick stone walls down the left/right sides, so the
                    // arena reads as an enclosed room rather than open
                    // ground. Full-strength color (not vignetted) - these
                    // are structural, not lighting, so they stay solid
                    // regardless of the floor's center-lit gradient.
                    const SIDE_WALL_THICKNESS: i32 = 4;
                    let fg = RGB::from_f32(
                        (wall_base.r * 1.4).min(1.0),
                        (wall_base.g * 1.4).min(1.0),
                        (wall_base.b * 1.4).min(1.0),
                    );
                    let wall_color_pair = ColorPair::new(fg, wall_base);
                    for y in 0..DISPLAY_HEIGHT {
                        for x in 0..SIDE_WALL_THICKNESS {
                            arena.set(Point::new(x, y), wall_color_pair, wall_glyph);
                            let rx = DISPLAY_WIDTH - 1 - x;
                            arena.set(Point::new(rx, y), wall_color_pair, wall_glyph);
                        }
                    }
                }
            }

            arena.submit(0).expect("Batch error");
        }

        // --- Portraits: each creature's own glyph, drawn once on the
        // coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS console, so it
        // renders far larger than its normal dungeon-map size. Enemies
        // stack down column 3 (see enemy_portrait_row); player sits
        // bottom-left. Whichever side is currently "Attacking" gets a
        // small shake instead of the plain draw - see
        // draw_wiggling_portrait below.
        let mut portraits = DrawBatch::new();
        portraits.target(3);
        let mut wiggle = DrawBatch::new();
        wiggle.target(BATTLE_PORTRAIT_WIGGLE_CONSOLE);
        let enemy_count = enemies.len();
        for (index, enemy) in enemies.iter().enumerate() {
            let (col, row) = enemy_portrait_position(enemy_count, index);
            let tinted = Render {
                color: flash_tint(enemy.render.color, enemy.flash),
                glyph: enemy.render.glyph,
            };
            if !draw_wiggling_portrait(&mut wiggle, col, row, tinted, enemy.flash) {
                if enemy_count <= 1 {
                    // Single enemy - the exact original whole-cell draw,
                    // unchanged, on console 3 like it always has been.
                    draw_portrait(&mut portraits, col as i32, row as i32, tinted);
                } else {
                    // 2+ enemies - fractional position (see
                    // enemy_portrait_position), which the whole-cell-only
                    // draw_portrait can't express, so this goes through
                    // the fancy console instead even while idle.
                    draw_portrait_fancy(&mut wiggle, col, row, tinted);
                }
            }
        }
        if let Some(render) = player_render {
            let tinted = Render {
                color: flash_tint(render.color, player_flash),
                glyph: render.glyph,
            };
            if !draw_wiggling_portrait(&mut wiggle, 1.0, 3.0, tinted, player_flash) {
                draw_portrait(&mut portraits, 1, 3, tinted);
            }
        }
        portraits.submit(0).expect("Batch error");
        wiggle.submit(1).expect("Batch error");
    }

    /// Records one enemy's death: stats, loot/gold (accumulated onto
    /// `battle` - see its gold_earned/loot_found/defeated_names fields),
    /// removes it from the ECS and from `battle.enemies`. If that was the
    /// last enemy in the fight, finishes the whole battle (BattleVictory)
    /// and returns true - the caller should stop touching `battle`
    /// immediately, same contract the old finish_battle_victory had.
    /// Returns false if the fight continues (other enemies remain).
    fn record_enemy_kill(&mut self, battle: &mut Battle, target: Entity) -> bool {
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
        // it. A Dungeon Crawl kill (no Gold component at all) keeps the
        // original loot roll exactly as before.
        let mut rng = RandomNumberGenerator::new();
        let loot = if current_gold.is_some() {
            None
        } else {
            grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player, target)
        };
        if let Some(item) = loot {
            battle.loot_found.push(item);
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
            self.finish_battle(battle);
            true
        } else {
            false
        }
    }

    /// Ends the whole battle in victory, using whatever's accumulated on
    /// `battle` across every kill this fight (see record_enemy_kill) -
    /// called the instant `battle.enemies` becomes empty.
    fn finish_battle(&mut self, battle: &Battle) {
        // Checked fresh here (not just "was gold_earned > 0") so a
        // Battle Arena fight that somehow ended with 0 net gold (it
        // shouldn't - every kill grants at least a few) still correctly
        // shows "0 gold" rather than being mistaken for a Dungeon Crawl
        // fight with no loot.
        let is_arena = self
            .ecs
            .entry_ref(battle.player)
            .ok()
            .and_then(|e| e.get_component::<Gold>().ok().copied())
            .is_some();

        self.resources.insert(Some(BattleVictory {
            player: battle.player,
            enemy_names: battle.defeated_names.clone(),
            loot: battle.loot_found.clone(),
            gold_earned: if is_arena {
                Some(battle.gold_earned)
            } else {
                None
            },
        }));
        self.resources.insert(None::<Battle>);
        self.resources.insert(TurnState::BattleVictory);
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
    fn trigger_enemy_action(&mut self, battle: &mut Battle, attacker: Entity) -> bool {
        let dot_message = tick_dot(&mut self.ecs, battle, attacker);
        if let Some(dot_message) = dot_message {
            battle.push_log(dot_message);
        }

        let (hp_now, _) = entity_health(&self.ecs, attacker);
        if hp_now < 1 {
            return self.record_enemy_kill(battle, attacker);
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
    fn resolve_player_action(&mut self, battle: &mut Battle, chosen: BattleAction) {
        let target = battle.primary_target(&self.ecs);
        // Every technique's mechanical effect is resolved in one place
        // (battle::apply_player_technique) rather than a match arm per
        // item here - adding a new class's technique needs no change here.
        match chosen {
            BattleAction::Attack => {
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
        battle.enter_result(Combatant::Player);
    }

    /// Outcome of dismiss_action_result - see that function.
    fn dismiss_action_result(&mut self, battle: &mut Battle) -> ResultOutcome {
        if battle.fled {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::AwaitingInput);
            return ResultOutcome::EndBattleTick;
        }

        let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
        if player_hp_now < 1 {
            self.resources.insert(None::<Battle>);
            self.resources.insert(TurnState::GameOver);
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
            if self.record_enemy_kill(battle, target) {
                return ResultOutcome::EndBattleTick;
            }
        }

        battle.turn = BattleTurn::Filling;
        ResultOutcome::Continue
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn battle_tick(&mut self, ctx: &mut BTerm) {
        let battle_snapshot = self.resources.get::<Option<Battle>>().unwrap().clone();
        let mut battle = match battle_snapshot {
            Some(b) => b,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        // Tick down any active post-action portrait flash/damage popup -
        // the player's own (still flat fields on Battle) and every
        // enemy's own (now on EnemyCombatant - see that struct's doc
        // comment).
        if let Some((_, remaining)) = &mut battle.player_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.player_flash = None;
            }
        }
        if let Some(popup) = &mut battle.player_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.player_damage_popup = None;
            }
        }
        for enemy in battle.enemies.iter_mut() {
            if let Some((_, remaining)) = &mut enemy.flash {
                *remaining -= ctx.frame_time_ms;
                if *remaining <= 0.0 {
                    enemy.flash = None;
                }
            }
            if let Some(popup) = &mut enemy.damage_popup {
                popup.remaining_ms -= ctx.frame_time_ms;
                if popup.remaining_ms <= 0.0 {
                    enemy.damage_popup = None;
                }
            }
        }

        // A multi-hit technique (MultiHit/AoeMultiHit) still has hits
        // waiting to land one at a time - see battle::damage::HitQueue.
        // Ticked unconditionally, same as the flash/popup timers just
        // above, so it keeps landing hits regardless of which
        // BattleTurn is currently showing. See the ActionResult(Player)
        // arm further below - it holds off its own auto-advance timer
        // while this is still Some, so the summary line has actually
        // been pushed before the result screen can dismiss.
        damage::tick_hit_queue(&mut self.ecs, &mut battle, ctx.frame_time_ms);

        // --- ATB gauges: fill continuously from Speed (see
        // BattleTurn::Filling's doc comment and atb_fill_rate) - the
        // FFVII-style replacement for the old fixed "whoever's faster
        // goes first this round" system, now generalized to the player
        // plus every enemy in the fight (each with its own independent
        // gauge - see EnemyCombatant::gauge). Every fill rate is scaled
        // by the player's chosen BattleSpeed (Options screen) - a pure
        // pacing knob applied uniformly to everyone, changing how long
        // battles take to sit through without changing who's faster than
        // whom.
        //
        // WHICH gauges actually tick this frame depends on both the
        // current BattleTurn and the chosen AtbMode - see AtbMode's own
        // doc comment for the full rules; the decision is identical
        // whether there's one enemy or four, it's just applied to the
        // whole `enemies` list now instead of a single field. Filling
        // always ticks everyone, regardless of mode: nobody has anything
        // "in progress" to protect a wait-pause for. Wait mode freezes
        // everything outside Filling, exactly as before this setting
        // existed. Active (True ATB) mode additionally lets every
        // enemy's gauge fill during the player's own PlayerMenu (see the
        // interrupt check further down), and lets the player's AND every
        // enemy's gauge keep filling during any one enemy's own
        // ActionResult - but still freezes everything during the
        // PLAYER's own ActionResult, the one deliberate exception (see
        // AtbMode::Active's doc comment for why).
        let battle_speed = *self.resources.get::<BattleSpeed>().unwrap();
        let atb_mode = *self.resources.get::<AtbMode>().unwrap();
        let (tick_player, tick_enemies) = match battle.turn {
            BattleTurn::Filling => (true, true),
            BattleTurn::PlayerMenu => (false, atb_mode == AtbMode::Active),
            BattleTurn::ActionResult(Combatant::Enemy(_)) => {
                let active = atb_mode == AtbMode::Active;
                (active, active)
            }
            BattleTurn::ActionResult(Combatant::Player) => (false, false),
        };
        if tick_player || tick_enemies {
            let speed_mult = battle_speed.rate_multiplier();
            if tick_player {
                battle.player_gauge = (battle.player_gauge
                    + atb_fill_rate(&self.ecs, battle.player) * speed_mult * ctx.frame_time_ms)
                    .min(ATB_GAUGE_MAX);
            }
            if tick_enemies {
                for enemy in battle.enemies.iter_mut() {
                    let rate = atb_fill_rate(&self.ecs, enemy.entity);
                    enemy.gauge =
                        (enemy.gauge + rate * speed_mult * ctx.frame_time_ms).min(ATB_GAUGE_MAX);
                }
            }
        }

        if battle.turn == BattleTurn::Filling {
            if let Some(chosen) = battle.queued_player_action.take() {
                // A True ATB queued choice (see Battle::queued_player_action)
                // - resolve it right now rather than waiting for the
                // player_gauge check below: it's already at max (that's
                // what made queuing possible in the first place), and the
                // whole point of queuing was to not make the player wait
                // any longer than necessary once it's finally safe to act.
                self.resolve_player_action(&mut battle, chosen);
            } else if battle.player_gauge >= ATB_GAUGE_MAX {
                // A same-frame tie always favors the player - simplest
                // deterministic rule, and it means the player is never the
                // one left waiting an extra frame purely due to check order.
                battle.turn = BattleTurn::PlayerMenu;
            } else if let Some(attacker) = battle
                .enemies
                .iter()
                .find(|e| e.gauge >= ATB_GAUGE_MAX)
                .map(|e| e.entity)
            {
                // Whichever enemy is first in stable list order among
                // those ready wins a same-frame tie, mirroring the
                // player-favoring rule above - simple and deterministic,
                // not meant to imply anything about "real" simultaneity.
                if self.trigger_enemy_action(&mut battle, attacker) {
                    return;
                }
            }
        }

        let (player_hp, player_max) = entity_health(&self.ecs, battle.player);
        let player_render = entity_render_component(&self.ecs, battle.player);

        let enemy_portraits: Vec<EnemyPortrait> = battle
            .enemies
            .iter()
            .filter_map(|e| {
                entity_render_component(&self.ecs, e.entity).map(|render| EnemyPortrait {
                    render,
                    flash: e.flash,
                })
            })
            .collect();
        self.draw_battle_arena(&enemy_portraits, player_render, battle.player_flash);

        // --- "You can act" indicator: whether the player can issue an
        // action RIGHT NOW - either a normal open PlayerMenu, or (True
        // ATB only) the queuing window during some enemy's own
        // ActionResult (see Battle::queued_player_action's doc comment).
        // Drives the Actions box border color below (green normally,
        // yellow while this is true) rather than tinting the player's
        // own portrait - a portrait tint turned out to read as a stray
        // color change with no clear meaning, and worse, it silently
        // went dark again the instant an enemy interrupted (turn moved
        // off PlayerMenu) even though - under True ATB - the player
        // could very much still act in that moment via queuing. The box
        // color is checked here, once, against the SAME condition that
        // actually gates input capture in both spots below (PlayerMenu's
        // own key handling and ActionResult(Enemy(_))'s queuing capture),
        // so it can never drift out of sync with what's actually
        // interactive.
        let player_can_act = battle.turn == BattleTurn::PlayerMenu
            || (atb_mode == AtbMode::Active
                && battle.queued_player_action.is_none()
                && battle.player_gauge >= ATB_GAUGE_MAX);

        // --- Text: name + HP bar anchored next to each portrait, and a
        // message/menu panel centered in the gap between them.
        ctx.set_active_console(2);

        // Whichever enemy the player's own single-target actions will hit
        // right now (see Battle::primary_target) - highlighted so the
        // player has SOME visibility into who they're about to attack,
        // even though they can't choose it directly with more than one
        // enemy present.
        let primary_target = battle.primary_target(&self.ecs);
        let enemy_count = battle.enemies.len();
        // A narrower bar for multi-enemy - text now sits directly below
        // each enemy's own (narrower, spread-out) portrait slot instead
        // of one shared wide column, so the old width-16 bar (an 18+
        // character string once the current/max numbers are appended)
        // would run into the NEXT enemy's own text. Single-enemy keeps
        // the original width entirely unchanged.
        // A narrower bar for multi-enemy - even though the corrected
        // console-2 math above gives a genuine 32-column gap between
        // adjacent enemy columns (comfortable room for a full-width bar
        // on its own), keeping this a bit narrower leaves visible
        // breathing room on either side rather than filling the gap
        // edge-to-edge. Single-enemy keeps the original width entirely
        // unchanged.
        let bar_width = if enemy_count <= 1 { 16 } else { 10 };
        for (index, enemy) in battle.enemies.iter().enumerate() {
            let (enemy_hp, enemy_max) = entity_health(&self.ecs, enemy.entity);
            let (col, base) = enemy_text_position(enemy_count, index);
            let is_target = Some(enemy.entity) == primary_target;
            let name_color = if is_target { YELLOW } else { WHITE };
            let name_text = if is_target && enemy_count > 1 {
                format!("> {}", enemy.name)
            } else {
                enemy.name.clone()
            };
            ctx.print_color(col, base, name_color, BLACK, &name_text);
            ctx.print_color(
                col,
                base + 1,
                YELLOW,
                BLACK,
                &format!(
                    "{} {}/{}",
                    hp_bar_string(enemy_hp, enemy_max, bar_width),
                    enemy_hp.max(0),
                    enemy_max
                ),
            );
            // ATB gauge, drawn as the same bracket-style bar as the HP bar
            // just above it - reuses hp_bar_string's ratio/width logic
            // directly by treating the gauge as a "current/max" pair of
            // its own. CYAN (rather than the HP bar's implicit yellow-on-
            // black) so the two bars read as different things at a
            // glance. Full-ready shows in GREEN instead, as a clear
            // "it's ready" signal distinct from "it's filling."
            ctx.print_color(
                col,
                base + 2,
                if enemy.gauge >= ATB_GAUGE_MAX {
                    GREEN
                } else {
                    CYAN
                },
                BLACK,
                &hp_bar_string(enemy.gauge as i32, ATB_GAUGE_MAX as i32, bar_width),
            );
            if let Some(ActiveStatus::Dot {
                label,
                turns_remaining,
                ..
            }) = enemy.statuses.get(StatusKind::Dot)
            {
                ctx.print_color(
                    col,
                    base + 3,
                    RED,
                    BLACK,
                    &format!("{} ({}t)", label, turns_remaining),
                );
            }
        }

        ctx.print_color(32, 58, WHITE, BLACK, "You");
        ctx.print_color(
            32,
            59,
            WHITE,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(player_hp, player_max, 16),
                player_hp.max(0),
                player_max
            ),
        );
        ctx.print_color(
            32,
            56,
            if battle.player_gauge >= ATB_GAUGE_MAX {
                GREEN
            } else {
                CYAN
            },
            BLACK,
            &hp_bar_string(battle.player_gauge as i32, ATB_GAUGE_MAX as i32, 16),
        );

        // --- Active-status line for the player: previously Defending,
        // Ice Armor, and an active counter all existed as real state with
        // zero visual presence. One combined line, shown whenever any of
        // it is active, ABOVE the name/HP block instead of below: the
        // player portrait starts at pixel y=480 / row 60, so a status
        // line at row 60 would sit directly under the portrait on
        // console 3 (registered after console 2) and never actually be
        // visible - row 57 keeps clear.
        let mut player_statuses = Vec::new();
        if battle.player_defending {
            player_statuses.push("Defending".to_string());
        }
        if let Some(armor) = entity_ice_armor(&self.ecs, battle.player) {
            player_statuses.push(format!("Ice Armor ({} left)", armor.attacks_remaining));
        }
        if let Some(remaining) = buff::remaining(&battle, BuffKind::DamageReduction) {
            player_statuses.push(format!("Battle Cry ({} left)", remaining));
        }
        if let Some(remaining) = buff::remaining(&battle, BuffKind::Evasion) {
            let chance = buff::flat_value(&battle, BuffKind::Evasion);
            player_statuses.push(format!("Dodge (+{}% evasion, {} left)", chance, remaining));
        }
        if battle.player_statuses.is_active(StatusKind::Counter) {
            player_statuses.push("Countering".to_string());
        }
        if !player_statuses.is_empty() {
            ctx.print_color(32, 57, CYAN, BLACK, &player_statuses.join(" | "));
        }

        // --- Battle log: up to MAX_LOG_LINES most-recent lines, in a
        // bordered box centered above the player (not the whole screen) -
        // the player portrait spans console-2 columns 32-64, centered on
        // column 48, so the box is centered there too. Sits in the gap
        // between the enemy panel and the player's status/name/HP block
        // (starts row 57), with a line of padding on both sides.
        const MSG_BOX_X: i32 = 36;
        const MSG_BOX_Y: i32 = 45;
        const MSG_BOX_WIDTH: i32 = 24;
        const MSG_BOX_HEIGHT: i32 = MAX_LOG_LINES as i32 + 2;

        let mut log_batch = DrawBatch::new();
        log_batch.target(2);
        draw_ascii_box(
            &mut log_batch,
            MSG_BOX_X,
            MSG_BOX_Y,
            MSG_BOX_WIDTH,
            MSG_BOX_HEIGHT,
            ColorPair::new(WHITE, BLACK),
        );
        log_batch.submit(0).expect("Batch error");

        for (i, line) in battle.log.iter().enumerate() {
            ctx.print_color(MSG_BOX_X + 2, MSG_BOX_Y + 1 + i as i32, WHITE, BLACK, line);
        }

        // --- Floating damage numbers: bigger (32px cells, same "big
        // text" font used for title/class-select screens), and centered
        // directly over each portrait now rather than off to the side -
        // big enough now to read clearly on top of the sprite instead of
        // needing to dodge it. Drawn on DAMAGE_POPUP_CONSOLE specifically
        // (not BIG_TEXT_CONSOLE, despite sharing its exact grid/font) -
        // BIG_TEXT_CONSOLE sits BELOW BATTLE_PORTRAIT_WIGGLE_CONSOLE in
        // z-order, and every non-wiggling multi-enemy portrait now draws
        // on that wiggle console too (see draw_portrait_fancy - it needs
        // a fancy console for fractional positions even when nothing's
        // actually shaking), which meant an idle enemy portrait was
        // painting directly over its own damage number every frame.
        // DAMAGE_POPUP_CONSOLE is registered last, so it renders above
        // every portrait regardless of which console that portrait used.
        // Every console gets ctx.cls()'d at the top of every frame (see
        // State::tick), so nothing lingers once a popup's timer expires.
        //
        // Both the portrait console (5x5) and this one (40x25) cover the
        // same physical 1280x800 window. Player portrait spans columns
        // 8-16 (center 12), rows 15-20 (center 17) - unaffected by enemy
        // count. Each enemy's own popup centers over wherever its own
        // portrait actually is now (see enemy_damage_popup_position),
        // rather than a single shared column/row. print_color draws
        // left-to-right from the given column, so the start column is
        // nudged left by half the number's length to actually center it
        // rather than just its left edge.
        ctx.set_active_console(DAMAGE_POPUP_CONSOLE);
        for (index, enemy) in battle.enemies.iter().enumerate() {
            if let Some(popup) = &enemy.damage_popup {
                let text = format!("-{}", popup.amount);
                let (center_col, row) = enemy_damage_popup_position(enemy_count, index);
                let start_col = center_col - (text.chars().count() as i32) / 2;
                ctx.print_color(start_col, row, RED, BLACK, &text);
            }
        }
        if let Some(popup) = &battle.player_damage_popup {
            let text = format!("-{}", popup.amount);
            let start_col = 12 - (text.chars().count() as i32) / 2;
            ctx.print_color(start_col, 17, RED, BLACK, &text);
        }
        ctx.set_active_console(2);

        // --- Actions box, on the HUD console (107x67 grid, ~12px cells -
        // the same "1.5x" size used for the dungeon HUD) rather than the
        // fine-text console (8px, too small) or the big-text title console
        // (32px, too big) - a middle ground per your feedback. BOX_X=44 is
        // deliberate: the player portrait is drawn at column 1 of the
        // 5-column portrait console (256-512px), and 44*~12=528px clears
        // that portrait's right edge (512px) with a little margin, at any
        // box height, since only the box's top edge moves with action
        // count.
        //
        // BOX_Y aligns the box's top edge with the player portrait's top
        // edge instead of bottom-anchoring to the HUD console. The player
        // portrait sits at row 3 of the 5-row portrait console (both
        // consoles cover the same physical 1280x800 window): row 3 starts
        // at 3 * (800/5) = 480px down, which lands at HUD row
        // 480 / (800/67) = ~40 on the HUD console's finer grid.
        //
        // Drawn here, before the match on battle.turn, so it's visible on
        // every battle_tick frame - Filling, PlayerMenu, and ActionResult
        // alike - rather than disappearing while a result message is on
        // screen (it's only interactive during PlayerMenu, but staying
        // visible the rest of the time avoids it popping in and out).
        // `actions` is computed here too since both
        // the box's labels and PlayerMenu's key-selection logic below need
        // the same list.
        let actions = available_actions(&self.ecs, battle.player);

        // Two columns instead of one long stacked list, which used to
        // blend the always-available capability actions in with the
        // class's technique roster - hard to scan at a glance,
        // especially once a class has 3-4 techniques. Left column: the
        // 3 capability actions (Attack/Defend/Flee), identified by
        // BattleAction variant rather than position, since Flee sits at
        // the END of the underlying Vec, after every technique (see
        // battle::available_actions) - it still needs to land in the
        // left column visually. Right column: the class's technique
        // roster, owned or not. Splitting is purely a DRAWING decision -
        // `i` below is still each entry's real index into `actions`, so
        // PlayerMenu's number-key selection further down (which does
        // `actions.get(i)`) is completely unaffected by which column
        // something is drawn in.
        let is_main_action = |entry: &&BattleMenuEntry| {
            matches!(
                entry.action,
                Some(BattleAction::Attack) | Some(BattleAction::Defend) | Some(BattleAction::Flee)
            )
        };
        let main_actions: Vec<(usize, &BattleMenuEntry)> = actions
            .iter()
            .enumerate()
            .filter(|(_, entry)| is_main_action(entry))
            .collect();
        let other_actions: Vec<(usize, &BattleMenuEntry)> = actions
            .iter()
            .enumerate()
            .filter(|(_, entry)| !is_main_action(entry))
            .collect();

        // Dropped the old "(locked)" suffix on an unowned technique - the
        // greyed-out DARK_GRAY color already says the same thing, and
        // the suffix was some of the longest text in the whole menu,
        // which was its own part of the "everything blends together"
        // problem.
        fn menu_row_label(i: usize, entry: &BattleMenuEntry) -> (String, RGB) {
            if entry.action.is_some() {
                let label = match entry.count {
                    Some(n) => format!("{}) {} x{}", i + 1, entry.label, n),
                    None => format!("{}) {}", i + 1, entry.label),
                };
                (label, GREEN.into())
            } else {
                (format!("{}) {}", i + 1, entry.label), DARK_GRAY.into())
            }
        }

        const BOX_X: i32 = 44;
        // Anchored to the player's own portrait top edge (see the doc
        // comment below) in the common case, but pushed down further
        // whenever there are 3+ enemies - the bottom tier of that
        // formation (see enemy_portrait_position) reaches down to around
        // row 39-42 of this same console, which would otherwise land
        // right under this box's own top edge.
        let box_y_base = if battle.enemies.len() >= 3 { 44 } else { 40 };
        const BOX_COL_WIDTH: i32 = 20;
        let box_width = BOX_COL_WIDTH * 2 + 3;
        let box_content_rows = main_actions.len().max(other_actions.len()) as i32;
        let box_height = box_content_rows + 4;
        // Clamp so a tall action list (more techniques than fit below row
        // 40) never runs off the bottom of the console.
        let box_y = box_y_base.min(HUD_ROWS - box_height);

        let mut menu_batch = DrawBatch::new();
        menu_batch.target(HUD_CONSOLE);
        // Border color reflects player_can_act (see its own doc comment
        // above) - yellow whenever the player can issue an action right
        // now, green otherwise. Replaces the earlier attempt at tinting
        // the player's own portrait, which read as an unexplained color
        // change and, worse, dropped out the instant the enemy
        // interrupted even when queuing (True ATB) still meant the
        // player could act.
        let box_border_color = if player_can_act { YELLOW } else { GREEN };
        draw_ascii_box(
            &mut menu_batch,
            BOX_X,
            box_y,
            box_width,
            box_height,
            ColorPair::new(box_border_color, BLACK),
        );
        menu_batch.submit(0).expect("Batch error");

        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color(BOX_X + 1, box_y + 1, YELLOW, BLACK, "Actions");
        for (row, (i, entry)) in main_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            ctx.print_color(BOX_X + 1, box_y + 3 + row as i32, color, BLACK, &label);
        }
        for (row, (i, entry)) in other_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            ctx.print_color(
                BOX_X + 1 + BOX_COL_WIDTH + 1,
                box_y + 3 + row as i32,
                color,
                BLACK,
                &label,
            );
        }
        ctx.set_active_console(2);

        match battle.turn {
            BattleTurn::Filling => {
                // Nothing to show beyond the gauges already drawn above -
                // there's no menu to interact with and no result to
                // dismiss while everyone is still racing to full.
            }
            BattleTurn::PlayerMenu => {
                if let Some(key) = ctx.key {
                    let chosen = number_key_index(key)
                        .and_then(|i| actions.get(i))
                        .and_then(|entry| entry.action);
                    if let Some(chosen) = chosen {
                        self.resolve_player_action(&mut battle, chosen);
                    }
                }

                // True ATB interrupt: if the player DIDN'T just act above
                // (battle.turn is still PlayerMenu - the resolve call
                // above would have moved it to ActionResult(Player)
                // otherwise) and some enemy's gauge has since filled all
                // the way (see this function's tick_enemies logic, which
                // only lets enemies fill during PlayerMenu under
                // AtbMode::Active), that enemy attacks right now instead
                // of waiting for a menu choice that never came in time.
                // This is the entire point of True ATB - "select an
                // attack ASAP or take the hit." The player isn't locked
                // out afterward, though - see ActionResult(Enemy(_))
                // below, which keeps accepting a choice (queued rather
                // than resolved immediately) for exactly this situation.
                let interrupting =
                    if atb_mode == AtbMode::Active && battle.turn == BattleTurn::PlayerMenu {
                        battle
                            .enemies
                            .iter()
                            .find(|e| e.gauge >= ATB_GAUGE_MAX)
                            .map(|e| e.entity)
                    } else {
                        None
                    };
                if let Some(attacker) = interrupting {
                    if self.trigger_enemy_action(&mut battle, attacker) {
                        return;
                    }
                }
            }
            BattleTurn::ActionResult(Combatant::Enemy(_)) => {
                // True ATB queuing: some enemy's own result may still be
                // playing while gauges keep moving underneath it (see
                // this function's tick_player/tick_enemies match - Active
                // mode lets both continue here). If the player's gauge
                // is already full and they haven't queued anything yet,
                // let them choose right now instead of forcing them to
                // wait for a fresh PlayerMenu prompt once this dismisses
                // - see Battle::queued_player_action's own doc comment
                // for why this was the actual fix needed: without it, an
                // enemy interrupt (just above) used to fully lock the
                // player out of choosing anything until its result
                // finished, making it very hard to ever land a hit under
                // True ATB. Wait mode never reaches this branch with
                // player_gauge at max in the first place (tick_player is
                // false for it here), so this is a no-op there.
                if atb_mode == AtbMode::Active
                    && battle.queued_player_action.is_none()
                    && battle.player_gauge >= ATB_GAUGE_MAX
                {
                    if let Some(key) = ctx.key {
                        let chosen = number_key_index(key)
                            .and_then(|i| actions.get(i))
                            .and_then(|entry| entry.action);
                        if let Some(chosen) = chosen {
                            battle.queued_player_action = Some(chosen);
                        }
                    }
                }

                battle.result_timer_ms -= ctx.frame_time_ms;
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if let ResultOutcome::EndBattleTick = self.dismiss_action_result(&mut battle) {
                        return;
                    }
                }
            }
            BattleTurn::ActionResult(Combatant::Player) => {
                // No queuing capture here (unlike the Enemy variant above)
                // - gauges are fully frozen during the player's OWN
                // action result in every mode (see this function's
                // tick_player/tick_enemies match), so player_gauge can't
                // possibly be back at max yet for there to be anything
                // to queue.
                //
                // While a multi-hit technique still has hits left to land
                // (battle.hit_queue - ticked unconditionally above, near
                // the flash/popup timers), hold off entirely: don't count
                // down the auto-advance timer, and ignore a keypress
                // dismiss too. Otherwise the result screen could dismiss
                // itself (or a keypress could dismiss it) before the
                // player ever saw every hit land - tick_hit_queue resets
                // result_timer_ms once the queue actually finishes, so
                // normal dismissal resumes automatically right after.
                if battle.hit_queue.is_none() {
                    battle.result_timer_ms -= ctx.frame_time_ms;
                    if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                        if let ResultOutcome::EndBattleTick =
                            self.dismiss_action_result(&mut battle)
                        {
                            return;
                        }
                    }
                }
            }
        }

        self.resources.insert(Some(battle));
    }

    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    pub fn battle_victory_tick(&mut self, ctx: &mut BTerm) {
        let victory_snapshot = self
            .resources
            .get::<Option<BattleVictory>>()
            .unwrap()
            .clone();
        let victory = match victory_snapshot {
            Some(v) => v,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        let player_render = entity_render_component(&self.ecs, victory.player);
        self.draw_battle_arena(&[], player_render, None);

        ctx.set_active_console(2);
        ctx.print_color_centered(
            45,
            GREEN,
            BLACK,
            &format!("You defeated {}!", join_enemy_names(&victory.enemy_names)),
        );
        match (victory.loot.is_empty(), victory.gold_earned) {
            (false, _) => {
                ctx.print_color_centered(
                    48,
                    YELLOW,
                    BLACK,
                    &format!("You found: {}!", victory.loot.join(", ")),
                );
            }
            (true, Some(gold)) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {} gold!", gold));
            }
            (true, None) => {
                ctx.print_color_centered(48, WHITE, BLACK, "No loot this time.");
            }
        }
        ctx.print_color_centered(51, YELLOW, BLACK, "Press ENTER to continue.");

        // Requires Enter specifically, not "any key" - this is the one
        // dismiss point in the whole battle flow that genuinely needed
        // it. Holding down an attack hotkey to keep queuing attacks ASAP
        // under Fast + True ATB (see Battle::queued_player_action) means
        // that key can still be held the instant the last enemy actually
        // dies. If this screen dismissed on any key, that same held key
        // would instantly exit back to the dungeon map - where, if it
        // happens to double as a Potion/Map slot key, it would keep
        // "using" that item and consuming a real dungeon turn on every
        // single frame it's still held, potentially burning through an
        // entire stack of potions and handing the monsters a pile of
        // free turns before the player even lets go of the key. The
        // in-battle ActionResult screens deliberately keep dismissing on
        // any key, unlike this one - blowing through those as fast as
        // possible while held is exactly the desired behavior there.
        if ctx.key == Some(VirtualKeyCode::Return) {
            self.resources.insert(None::<BattleVictory>);
            let arena_run = self.resources.get::<Option<ArenaRun>>().unwrap().clone();
            match arena_run {
                Some(run) => self.handle_arena_kill(run),
                None => {
                    self.resources.insert(TurnState::AwaitingInput);
                }
            }
        }
    }
}
