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
        // top-right, player sits bottom-left. Whichever side is currently
        // "Attacking" gets a small shake instead of the plain draw - see
        // draw_wiggling_portrait below.
        let mut portraits = DrawBatch::new();
        portraits.target(3);
        let mut wiggle = DrawBatch::new();
        wiggle.target(BATTLE_PORTRAIT_WIGGLE_CONSOLE);
        if let Some(render) = enemy_render {
            let tinted = Render {
                color: flash_tint(render.color, enemy_flash),
                glyph: render.glyph,
            };
            if !draw_wiggling_portrait(&mut wiggle, 3, 1, tinted, enemy_flash) {
                draw_portrait(&mut portraits, 3, 1, tinted);
            }
        }
        if let Some(render) = player_render {
            let tinted = Render {
                color: flash_tint(render.color, player_flash),
                glyph: render.glyph,
            };
            if !draw_wiggling_portrait(&mut wiggle, 1, 3, tinted, player_flash) {
                draw_portrait(&mut portraits, 1, 3, tinted);
            }
        }
        portraits.submit(0).expect("Batch error");
        wiggle.submit(1).expect("Batch error");
    }

    /// Shared "the enemy just died in battle" ending - grants loot,
    /// removes the enemy entity, records an enemies_killed stat for the
    /// player's class, and transitions to TurnState::BattleVictory. This
    /// exact sequence used to be pasted at all four points in battle_tick
    /// where an enemy's Health can drop below 1 (the player's own attack,
    /// a Counter Attack reacting to an enemy hit whichever order the
    /// round went, and a DoT tick before initiative is even decided) -
    /// one shared landing point instead of four copies that would
    /// otherwise all need the same one more line added to them.
    fn finish_battle_victory(&mut self, battle: &Battle) {
        if let Some(class) = entity_class(&self.ecs, battle.player) {
            if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                stats.record_enemy_killed(&class);
            }
        }

        // Read the enemy's Boss tag and the player's current Gold total
        // BEFORE the enemy is removed below - Gold's own presence (not a
        // separate Option<ArenaRun> check) is what decides whether this
        // is a Battle Arena kill at all, per that component's own doc
        // comment.
        let is_boss = self
            .ecs
            .entry_ref(battle.enemy)
            .map(|e| e.get_component::<Boss>().is_ok())
            .unwrap_or(false);
        let current_gold = self
            .ecs
            .entry_ref(battle.player)
            .ok()
            .and_then(|e| e.get_component::<Gold>().ok().copied());

        // In the Battle Arena, a kill's reward is gold ONLY - the old
        // random ability-drop loot is deliberately not granted alongside
        // it (this was the actual gap: gold got added on top of the
        // existing drop instead of replacing it, so Arena battles kept
        // handing out ability items neither priced nor meant to still be
        // free). A Dungeon Crawl kill (no Gold component at all) keeps
        // the original loot roll exactly as before - gold doesn't exist
        // there, so there's nothing to replace it with.
        let mut rng = RandomNumberGenerator::new();
        let loot = if current_gold.is_some() {
            None
        } else {
            grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player, battle.enemy)
        };

        let mut cb = CommandBuffer::new(&mut self.ecs);
        let gold_earned = current_gold.map(|Gold(amount)| {
            let reward = gold_reward_for_kill(is_boss);
            cb.add_component(battle.player, Gold(amount + reward));
            reward
        });
        cb.remove(battle.enemy);
        cb.flush(&mut self.ecs);
        self.resources.insert(Some(BattleVictory {
            player: battle.player,
            enemy_name: battle.enemy_name.clone(),
            loot,
            gold_earned,
        }));
        self.resources.insert(None::<Battle>);
        self.resources.insert(TurnState::BattleVictory);
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

        // --- ATB gauges: fill continuously from Speed while turn ==
        // Filling (see BattleTurn::Filling's doc comment and
        // atb_fill_rate) - the FFVII-style replacement for the old fixed
        // "whoever's faster goes first this round" system. Both gauges
        // freeze the instant either reaches ATB_GAUGE_MAX (Wait-ATB) for
        // the ensuing PlayerMenu/ActionResult exchange, and only resume
        // once back in Filling - see enter_result, which is what returns
        // things to Filling.
        if battle.turn == BattleTurn::Filling {
            battle.player_gauge = (battle.player_gauge
                + atb_fill_rate(&self.ecs, battle.player) * ctx.frame_time_ms)
                .min(ATB_GAUGE_MAX);
            battle.enemy_gauge = (battle.enemy_gauge
                + atb_fill_rate(&self.ecs, battle.enemy) * ctx.frame_time_ms)
                .min(ATB_GAUGE_MAX);

            // A same-frame tie always favors the player - simplest
            // deterministic rule, and it means the player is never the
            // one left waiting an extra frame purely due to check order.
            if battle.player_gauge >= ATB_GAUGE_MAX {
                battle.turn = BattleTurn::PlayerMenu;
            } else if battle.enemy_gauge >= ATB_GAUGE_MAX {
                // The enemy acts automatically the moment its gauge
                // fills - no menu, same as the old "enemy is faster"
                // branch. Any active Dot ticks first (a lingering wound,
                // not an action in its own right) and can end the fight
                // before the enemy ever gets to swing.
                let dot_message = tick_dot(&mut self.ecs, &mut battle);
                if let Some(dot_message) = dot_message {
                    battle.push_log(dot_message);
                }

                let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                if enemy_hp_now < 1 {
                    self.finish_battle_victory(&battle);
                    return;
                }

                resolve_enemy_attack(&mut self.ecs, &mut battle);
                battle.enter_result(Combatant::Enemy);
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
        // ATB gauge, drawn as the same bracket-style bar as the HP bar
        // just above it - reuses hp_bar_string's ratio/width logic
        // directly by treating the gauge as a "current/max" pair of its
        // own. CYAN (rather than the HP bar's implicit yellow-on-black)
        // so the two bars read as different things at a glance. Full-ready
        // shows in GREEN instead, as a clear "it's ready" signal distinct
        // from "it's filling."
        ctx.print_color(
            96,
            43,
            if battle.enemy_gauge >= ATB_GAUGE_MAX {
                GREEN
            } else {
                CYAN
            },
            BLACK,
            &hp_bar_string(battle.enemy_gauge as i32, ATB_GAUGE_MAX as i32, 16),
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
        if let Some(ActiveStatus::Dot {
            label,
            turns_remaining,
            ..
        }) = battle.enemy_statuses.get(StatusKind::Dot)
        {
            ctx.print_color(
                96,
                44,
                RED,
                BLACK,
                &format!("{} ({} turns left)", label, turns_remaining),
            );
        }

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

        // --- Floating damage numbers: bigger (BIG_TEXT_CONSOLE's 32px cells,
        // same "big text" console used for title/class-select screens),
        // and centered directly over each portrait now rather than off to
        // the side - big enough now to read clearly on top of the sprite
        // instead of needing to dodge it. BIG_TEXT_CONSOLE is registered after
        // the portrait console, so it renders above the portraits, and
        // every console gets
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
        ctx.set_active_console(BIG_TEXT_CONSOLE);
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

        // Column widths sized to THIS class's own current labels (not a
        // fixed guess), so the box fits snugly whether it's Archer (no
        // techniques - right column stays empty) or Barbarian (4). Left
        // column labels are always short ("N) Defend"), but computing it
        // the same way keeps both sides consistent if that ever changes.
        let left_width = main_actions
            .iter()
            .map(|(i, entry)| menu_row_label(*i, entry).0.chars().count())
            .max()
            .unwrap_or(0) as i32;
        // max()'d against "Techniques".len() too - every current class's
        // technique labels are already longer than that header once a
        // number prefix/count suffix is added, but this keeps the header
        // from ever overflowing the box if a future class's techniques
        // all happen to have very short names.
        let right_width = other_actions
            .iter()
            .map(|(i, entry)| menu_row_label(*i, entry).0.chars().count())
            .max()
            .unwrap_or(0)
            .max(if other_actions.is_empty() {
                0
            } else {
                "Techniques".len()
            }) as i32;
        const COLUMN_GAP: i32 = 3;
        let right_col_x_offset = left_width + COLUMN_GAP;

        const BOX_X: i32 = 44;
        const BOX_Y: i32 = 40;
        // +2 for the left/right border columns, +1 so the right column's
        // text never touches the right border.
        let box_width = right_col_x_offset + right_width + 3;
        let box_content_rows = main_actions.len().max(other_actions.len()) as i32;
        let box_height = box_content_rows + 4;
        // Clamp so a tall action list (more techniques than fit below row
        // 40) never runs off the bottom of the console.
        let box_y = BOX_Y.min(HUD_ROWS - box_height);

        let mut menu_batch = DrawBatch::new();
        menu_batch.target(HUD_CONSOLE);
        draw_ascii_box(
            &mut menu_batch,
            BOX_X,
            box_y,
            box_width,
            box_height,
            ColorPair::new(GREEN, BLACK),
        );
        menu_batch.submit(0).expect("Batch error");

        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color(BOX_X + 1, box_y + 1, YELLOW, BLACK, "Actions");
        for (row, (i, entry)) in main_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            ctx.print_color(BOX_X + 1, box_y + 3 + row as i32, color, BLACK, &label);
        }
        // Only label the right column if this class actually has any
        // techniques at all (Archer/Debug don't) - an empty "Techniques"
        // header over nothing would just be more clutter, not less.
        if !other_actions.is_empty() {
            ctx.print_color(
                BOX_X + 1 + right_col_x_offset,
                box_y + 1,
                YELLOW,
                BLACK,
                "Techniques",
            );
        }
        for (row, (i, entry)) in other_actions.iter().enumerate() {
            let (label, color) = menu_row_label(*i, entry);
            ctx.print_color(
                BOX_X + 1 + right_col_x_offset,
                box_y + 3 + row as i32,
                color,
                BLACK,
                &label,
            );
        }
        // Restore console 2 - the enemy/player name+HP text above and
        // every match arm below assume it's active (it's set once, above
        // the whole match block, not re-set per arm).
        ctx.set_active_console(2);

        match battle.turn {
            BattleTurn::Filling => {
                // Nothing to show beyond the gauges already drawn above -
                // there's no menu to interact with and no result to
                // dismiss while both sides are still racing to full.
            }
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
                                let dmg = damage::strike(
                                    &mut self.ecs,
                                    &mut battle,
                                    Combatant::Player,
                                    dmg,
                                );
                                battle.push_log(damage::strike_message(dmg));
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
                                // Recorded BEFORE apply_player_technique
                                // runs - it removes `item` from the ECS
                                // as part of consuming it, so its Name/
                                // Class have to be read while it's still
                                // there. Keyed off the item's own class,
                                // same reasoning as
                                // Stats::record_ability_used's doc
                                // comment.
                                if let Some(class) = entity_class(&self.ecs, item) {
                                    let name = entity_name(&self.ecs, item);
                                    if let Some(mut stats) = self.resources.get_mut::<Stats>() {
                                        stats.record_ability_used(&class, &name);
                                    }
                                }
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
                        battle.enter_result(Combatant::Player);
                    }
                }
            }
            // Payload intentionally unused: unlike the old FirstResult/
            // SecondResult split, an ActionResult's handling doesn't
            // actually depend on which combatant acted - by the time
            // either one lands here, their action (attack, technique,
            // resolve_enemy_attack, whatever) has already fully
            // resolved, including any Counter Attack side effect. This
            // one block just checks whether EITHER side is now dead
            // (matching the old SecondResult's own "check both, not just
            // the expected one" logic) and returns to Filling if not.
            // Kept as a payload rather than a bare unit variant since a
            // future feature (a per-actor result flavor, a camera focus
            // cue, etc.) would want to know without restructuring this
            // enum again.
            BattleTurn::ActionResult(_acting) => {
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

                    let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                    let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);

                    if player_hp_now < 1 {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::GameOver);
                        return;
                    }

                    if enemy_hp_now < 1 {
                        self.finish_battle_victory(&battle);
                        return;
                    }

                    battle.turn = BattleTurn::Filling;
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
        match (&victory.loot, victory.gold_earned) {
            (Some(item), _) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {}!", item));
            }
            (None, Some(gold)) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {} gold!", gold));
            }
            (None, None) => {
                ctx.print_color_centered(48, WHITE, BLACK, "No loot this time.");
            }
        }
        ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");

        if ctx.key.is_some() {
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
