use crate::prelude::*;
use crate::State;

impl State {
    /// Draws the battle arena background (console 0: the current theme's
    /// floor/walls/scenery) and both combatant portraits (console 3).
    /// Callers look up Render live during an active battle, or pass a
    /// value captured before an entity was removed (see
    /// battle_victory_tick, where the enemy no longer exists in the ECS).
    fn draw_battle_arena(
        &mut self,
        enemy_render: Option<Render>,
        player_render: Option<Render>,
        enemy_flash: Option<(FlashKind, f32)>,
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
        // renders far larger than its normal dungeon-map size. Enemy sits
        // top-right, player sits bottom-left.
        let mut portraits = DrawBatch::new();
        portraits.target(3);
        if let Some(render) = enemy_render {
            let tinted = Render {
                color: flash_tint(render.color, enemy_flash),
                glyph: render.glyph,
            };
            draw_portrait(&mut portraits, 3, 1, tinted);
        }
        if let Some(render) = player_render {
            let tinted = Render {
                color: flash_tint(render.color, player_flash),
                glyph: render.glyph,
            };
            draw_portrait(&mut portraits, 1, 3, tinted);
        }
        portraits.submit(0).expect("Batch error");
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

        // Tick down any active post-action portrait flash (see
        // Battle::enemy_flash/player_flash and flash_tint).
        if let Some((_, remaining)) = &mut battle.enemy_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.enemy_flash = None;
            }
        }
        if let Some((_, remaining)) = &mut battle.player_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.player_flash = None;
            }
        }

        // Tick down any active floating damage number (see
        // Battle::enemy_damage_popup/player_damage_popup) the same way.
        if let Some(popup) = &mut battle.enemy_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.enemy_damage_popup = None;
            }
        }
        if let Some(popup) = &mut battle.player_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.player_damage_popup = None;
            }
        }

        // --- Initiative: decided once at the start of each round, from
        // Speed. Any active damage-over-time effect (Rend, Burn, etc.)
        // ticks first, before initiative is even decided - it's a
        // lingering wound, not an action. If the enemy is faster, they
        // attack immediately here - no menu shown - before the player
        // ever gets a choice this round.
        if battle.awaiting_order_decision {
            let dot_message = tick_dot(&mut self.ecs, &mut battle);

            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
            if enemy_hp_now < 1 {
                let mut rng = RandomNumberGenerator::new();
                let loot =
                    grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player, battle.enemy);
                let mut cb = CommandBuffer::new(&mut self.ecs);
                cb.remove(battle.enemy);
                cb.flush(&mut self.ecs);
                self.resources.insert(Some(BattleVictory {
                    player: battle.player,
                    enemy_name: battle.enemy_name.clone(),
                    loot,
                }));
                self.resources.insert(None::<Battle>);
                self.resources.insert(TurnState::BattleVictory);
                return;
            }

            let player_speed = entity_speed(&self.ecs, battle.player);
            let enemy_speed = entity_speed(&self.ecs, battle.enemy);
            battle.first_actor = if battle.sneak_attack || player_speed >= enemy_speed {
                Combatant::Player
            } else {
                Combatant::Enemy
            };
            battle.awaiting_order_decision = false;

            if battle.first_actor == Combatant::Enemy {
                if let Some(dot_message) = dot_message {
                    battle.push_log(dot_message);
                }
                resolve_enemy_attack(&mut self.ecs, &mut battle);
                battle.enter_result(BattleTurn::FirstResult);
            } else if let Some(dot_message) = dot_message {
                battle.push_log(dot_message);
            }
        }

        let (enemy_hp, enemy_max) = entity_health(&self.ecs, battle.enemy);
        let (player_hp, player_max) = entity_health(&self.ecs, battle.player);

        let enemy_render = entity_render_component(&self.ecs, battle.enemy);
        let player_render = entity_render_component(&self.ecs, battle.player);
        self.draw_battle_arena(
            enemy_render,
            player_render,
            battle.enemy_flash,
            battle.player_flash,
        );

        // --- Text: name + HP bar anchored next to each portrait, and a
        // message/menu panel centered in the gap between them.
        ctx.set_active_console(2);

        ctx.print_color(96, 41, YELLOW, BLACK, &battle.enemy_name);
        ctx.print_color(
            96,
            42,
            YELLOW,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(enemy_hp, enemy_max, 16),
                enemy_hp.max(0),
                enemy_max
            ),
        );

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

        // --- Active-status lines: previously Defending, Ice Armor, an
        // active counter, and enemy damage-over-time all existed as real
        // state with zero visual presence. One combined line per
        // combatant, shown whenever any of that combatant's statuses are
        // active. Enemy's goes below its HP bar (clear of the portrait,
        // which ends at pixel y=320 / row 40). Player's goes ABOVE its
        // name/HP block instead of below: the player portrait starts at
        // pixel y=480 / row 60, so a status line at row 60 would sit
        // directly under the portrait on console 3 (registered after
        // console 2) and never actually be visible - row 57 keeps clear.
        if let Some(dot) = &battle.enemy_dot {
            ctx.print_color(
                96,
                43,
                RED,
                BLACK,
                &format!("{} ({} turns left)", dot.label, dot.turns_remaining),
            );
        }

        let mut player_statuses = Vec::new();
        if battle.player_defending {
            player_statuses.push("Defending".to_string());
        }
        if let Some(armor) = entity_ice_armor(&self.ecs, battle.player) {
            player_statuses.push(format!("Ice Armor ({} left)", armor.attacks_remaining));
        }
        if battle.countering.is_some() {
            player_statuses.push("Countering".to_string());
        }
        if !player_statuses.is_empty() {
            ctx.print_color(32, 57, CYAN, BLACK, &player_statuses.join(" | "));
        }

        // --- Battle log: up to MAX_LOG_LINES most-recent lines, in a
        // bordered box centered above the player (not the whole screen) -
        // the player portrait spans console-2 columns 32-64, centered on
        // column 48, so the box is centered there too. Sits in the gap
        // between the enemy's text block (ends row 43) and the player's
        // status/name/HP block (starts row 57), with a line of padding on
        // both sides.
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

        // --- Floating damage numbers: bigger (console 5's 32px cells,
        // same "big text" console used for title/class-select screens),
        // and centered directly over each portrait now rather than off to
        // the side - big enough now to read clearly on top of the sprite
        // instead of needing to dodge it. Console 5 is registered last, so
        // it renders above the portraits, and every console gets
        // ctx.cls()'d at the top of every frame (see State::tick), so
        // nothing lingers once a popup's timer expires.
        //
        // Both the portrait console (5x5) and this one (40x25) cover the
        // same physical 1280x800 window. Enemy portrait spans columns
        // 24-32 (center 28), rows 5-10 (center 7). Player portrait spans
        // columns 8-16 (center 12), rows 15-20 (center 17). print_color
        // draws left-to-right from the given column, so the start column
        // is nudged left by half the number's length to actually center
        // it rather than just its left edge.
        ctx.set_active_console(5);
        if let Some(popup) = &battle.enemy_damage_popup {
            let text = format!("-{}", popup.amount);
            let start_col = 28 - (text.chars().count() as i32) / 2;
            ctx.print_color(start_col, 7, RED, BLACK, &text);
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
        // every battle_tick frame - PlayerMenu, FirstResult, and
        // SecondResult alike - rather than disappearing while a result
        // message is on screen. `actions` is computed here too since both
        // the box's labels and PlayerMenu's key-selection logic below need
        // the same list.
        let actions = available_actions(&self.ecs, battle.player);

        const BOX_X: i32 = 44;
        const BOX_WIDTH: i32 = 26;
        const BOX_Y: i32 = 40;
        let box_height = actions.len() as i32 + 4;
        // Clamp so a tall action list (more techniques than fit below row
        // 40) never runs off the bottom of the console.
        let box_y = BOX_Y.min(HUD_ROWS - box_height);

        let mut menu_batch = DrawBatch::new();
        menu_batch.target(4);
        draw_ascii_box(
            &mut menu_batch,
            BOX_X,
            box_y,
            BOX_WIDTH,
            box_height,
            ColorPair::new(GREEN, BLACK),
        );
        menu_batch.submit(0).expect("Batch error");

        ctx.set_active_console(4);
        ctx.print_color(BOX_X + 1, box_y + 1, YELLOW, BLACK, "Actions");
        for (i, entry) in actions.iter().enumerate() {
            // Every action this class could ever have is always listed
            // (see battle::available_actions) - one not currently owned
            // shows greyed out and isn't selectable, rather than
            // disappearing from the menu entirely, so the list stays a
            // stable reference of what the class can eventually do.
            let (label, color) = if entry.action.is_some() {
                let label = match entry.count {
                    Some(n) => format!("{}) {} x{}", i + 1, entry.label, n),
                    None => format!("{}) {}", i + 1, entry.label),
                };
                (label, GREEN)
            } else {
                (format!("{}) {} (locked)", i + 1, entry.label), DARK_GRAY)
            };
            ctx.print_color(BOX_X + 1, box_y + 3 + i as i32, color, BLACK, &label);
        }
        // Restore console 2 - the enemy/player name+HP text above and
        // every match arm below assume it's active (it's set once, above
        // the whole match block, not re-set per arm).
        ctx.set_active_console(2);

        match battle.turn {
            BattleTurn::PlayerMenu => {
                if let Some(key) = ctx.key {
                    let chosen = number_key_index(key)
                        .and_then(|i| actions.get(i))
                        .and_then(|entry| entry.action);
                    if let Some(chosen) = chosen {
                        // Every technique's mechanical effect is resolved
                        // in one place (battle::apply_player_technique)
                        // rather than a match arm per item here - adding a
                        // new class's technique needs no main.rs change.
                        match chosen {
                            BattleAction::Attack => {
                                let mut dmg = player_attack_damage(&self.ecs, battle.player);
                                if battle.sneak_attack {
                                    dmg *= 3;
                                }
                                let dmg = apply_damage(&mut self.ecs, battle.enemy, dmg);
                                battle.show_enemy_damage(dmg);
                                battle.player_flash =
                                    Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
                                battle.enemy_flash =
                                    Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
                                battle.push_log(if dmg == 0 {
                                    "Dodge attack.".to_string()
                                } else {
                                    format!("Deal {} damage.", dmg)
                                });
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
                                let result =
                                    apply_player_technique(&mut self.ecs, &mut battle, item);
                                battle.push_log(result);
                            }
                        }
                        // Sneak attack is a one-shot ambush bonus for the
                        // guaranteed first action only - clear it here
                        // regardless of which action was actually chosen,
                        // so it can never linger and apply again later in
                        // the same fight.
                        battle.sneak_attack = false;
                        // The player is first_actor at the start of a round
                        // they act in unprompted; if the enemy already
                        // opened the round (first_actor == Enemy), this
                        // menu is the player's second action instead.
                        battle.enter_result(if battle.first_actor == Combatant::Player {
                            BattleTurn::FirstResult
                        } else {
                            BattleTurn::SecondResult
                        });
                    }
                }
            }
            BattleTurn::FirstResult => {
                // Auto-advances once result_timer_ms runs out (see
                // Battle::enter_result/RESULT_AUTO_ADVANCE_MS) - a keypress
                // still skips ahead immediately, it just isn't required.
                battle.result_timer_ms -= ctx.frame_time_ms;
                ctx.print_color_centered(51, YELLOW, BLACK, "(press any key to skip ahead)");
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if battle.fled {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    match battle.first_actor {
                        Combatant::Player => {
                            // Player went first and attacked the enemy -
                            // check whether that finished the fight before
                            // letting the enemy retaliate.
                            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                            if enemy_hp_now < 1 {
                                let mut rng = RandomNumberGenerator::new();
                                let loot = grant_random_battle_loot(
                                    &mut self.ecs,
                                    &mut rng,
                                    battle.player,
                                    battle.enemy,
                                );
                                let mut cb = CommandBuffer::new(&mut self.ecs);
                                cb.remove(battle.enemy);
                                cb.flush(&mut self.ecs);
                                self.resources.insert(Some(BattleVictory {
                                    player: battle.player,
                                    enemy_name: battle.enemy_name.clone(),
                                    loot,
                                }));
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::BattleVictory);
                                return;
                            }
                            resolve_enemy_attack(&mut self.ecs, &mut battle);
                            battle.enter_result(BattleTurn::SecondResult);
                        }
                        Combatant::Enemy => {
                            // Enemy went first (they're faster) and already
                            // attacked the player - check whether that
                            // ended things before the player gets a turn.
                            let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                            if player_hp_now < 1 {
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::GameOver);
                                return;
                            }

                            // A Counter Attack can kill the enemy as a
                            // side effect of their own attack (see
                            // resolve_enemy_attack) - check for that too,
                            // or the player gets an extra prompt against
                            // an already-dead enemy.
                            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                            if enemy_hp_now < 1 {
                                let mut rng = RandomNumberGenerator::new();
                                let loot = grant_random_battle_loot(
                                    &mut self.ecs,
                                    &mut rng,
                                    battle.player,
                                    battle.enemy,
                                );
                                let mut cb = CommandBuffer::new(&mut self.ecs);
                                cb.remove(battle.enemy);
                                cb.flush(&mut self.ecs);
                                self.resources.insert(Some(BattleVictory {
                                    player: battle.player,
                                    enemy_name: battle.enemy_name.clone(),
                                    loot,
                                }));
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::BattleVictory);
                                return;
                            }

                            battle.turn = BattleTurn::PlayerMenu;
                        }
                    }
                }
            }
            BattleTurn::SecondResult => {
                battle.result_timer_ms -= ctx.frame_time_ms;
                ctx.print_color_centered(51, YELLOW, BLACK, "(press any key to skip ahead)");
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if battle.fled {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    // Whoever acted second this round attacked whoever
                    // acted first - but a Counter Attack can also kill the
                    // enemy as a side effect of an enemy attack regardless
                    // of who that attack's "real" target was, so check
                    // both sides here rather than just the expected one.
                    let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                    let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);

                    if player_hp_now < 1 {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::GameOver);
                        return;
                    }

                    if enemy_hp_now < 1 {
                        let mut rng = RandomNumberGenerator::new();
                        let loot = grant_random_battle_loot(
                            &mut self.ecs,
                            &mut rng,
                            battle.player,
                            battle.enemy,
                        );
                        let mut cb = CommandBuffer::new(&mut self.ecs);
                        cb.remove(battle.enemy);
                        cb.flush(&mut self.ecs);
                        self.resources.insert(Some(BattleVictory {
                            player: battle.player,
                            enemy_name: battle.enemy_name.clone(),
                            loot,
                        }));
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::BattleVictory);
                        return;
                    }

                    battle.turn = BattleTurn::PlayerMenu;
                    battle.awaiting_order_decision = true;
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
        self.draw_battle_arena(None, player_render, None, None);

        ctx.set_active_console(2);
        ctx.print_color_centered(
            45,
            GREEN,
            BLACK,
            &format!("You defeated the {}!", victory.enemy_name),
        );
        match &victory.loot {
            Some(item) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {}!", item));
            }
            None => {
                ctx.print_color_centered(48, WHITE, BLACK, "No loot this time.");
            }
        }
        ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");

        if ctx.key.is_some() {
            self.resources.insert(None::<BattleVictory>);
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}
