use crate::prelude::*;
use crate::State;

impl State {
    /// The play-history screen, reached from the title screen (press H),
    /// returning there on Escape. Reuses pause_systems to redraw a static
    /// backdrop, same trick options_tick already uses (this screen only
    /// exists at the title screen today, but pause_systems' map-render-
    /// only redraw works fine there too - the same decorative background
    /// title_screen/class_select show, just without their own wandering-
    /// enemy animation, which a menu screen doesn't need anyway).
    ///
    /// Three sub-modes, tracked by `self.stats_view_mode` (see
    /// StatsViewMode's own doc comment):
    /// - Overview: the two Dungeon/Arena summary lines, then a table with
    ///   one row per class that's actually been played at least once - a
    ///   class never played is simply absent rather than shown as a zero
    ///   row. A number key drills into that row's ability usage; I drills
    ///   into the global Items Used list.
    /// - ClassAbilities(class): every ability that class has used at
    ///   least once, with its count. Escape returns to Overview.
    /// - ItemUsage: every unrestricted item (Healing Potion, Dungeon Map,
    ///   ...) used at least once, with its count. Escape returns to
    ///   Overview.
    pub fn stats_view_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, YELLOW, BLACK, "-- History --");

        // HUD_CONSOLE (~12px cells) for the body, matching every other
        // pre-game screen's font bump - see title_screen's comment on the
        // same change. Row numbers below are fresh picks for HUD_CONSOLE's
        // 67-row grid, not a mechanical conversion of the old console 2
        // values (which assumed a 100-row grid).
        ctx.set_active_console(HUD_CONSOLE);

        // Cloned so the borrow of self.resources doesn't outlive this
        // point - every branch below mutates `self` (stats_view_mode or
        // self.resources itself), which a live borrow from
        // self.resources.get::<Stats>() would conflict with.
        let stats = self
            .resources
            .get::<Stats>()
            .expect("Stats resource missing")
            .clone();

        match self.stats_view_mode.clone() {
            StatsViewMode::Overview => {
                let win_rate = |won: u32, played: u32| {
                    if played > 0 {
                        format!(" ({:.0}%)", 100.0 * won as f32 / played as f32)
                    } else {
                        String::new()
                    }
                };
                ctx.print_color_centered(
                    8,
                    WHITE,
                    BLACK,
                    &format!(
                        "Dungeon Crawl: {} of {} runs won{}",
                        stats.games_won,
                        stats.games_played,
                        win_rate(stats.games_won, stats.games_played)
                    ),
                );
                ctx.print_color_centered(
                    10,
                    WHITE,
                    BLACK,
                    &format!(
                        "Battle Arena: {} of {} runs won{}",
                        stats.arena_games_won,
                        stats.arena_games_played,
                        win_rate(stats.arena_games_won, stats.arena_games_played)
                    ),
                );

                let mut classes: Vec<String> = stats.per_class.keys().cloned().collect();
                classes.sort();

                if classes.is_empty() {
                    ctx.print_color_centered(15, GRAY, BLACK, "No games played yet.");
                } else {
                    // Fixed-width columns, printed via print_color_centered -
                    // every row (header included) formats to the exact same
                    // total length, so centering each line independently
                    // still lines every column up under the one above it.
                    // A literal " " separator sits between every field
                    // (not just padding baked into the width specs) so a
                    // value that grows to fill or exceed its column still
                    // keeps a guaranteed gap from its neighbor instead of
                    // running straight into it - padding alone only
                    // guarantees a gap as long as nothing ever reaches the
                    // column's width, which is exactly what looked too
                    // tight before. Column widths also carry real headroom
                    // now (Runs/Kills to 6/7 digits, Class to 14 chars)
                    // rather than being sized to just barely fit today's
                    // numbers.
                    let header = format!(
                        "{:<3} {:<14} {:>6} {:>6} {:>7} {:>11} {:>9}",
                        "", "Class", "Runs", "Won", "Kills", "Dungeon Lvl", "Arena Lvl"
                    );
                    ctx.print_color_centered(13, YELLOW, BLACK, &header);

                    for (i, class) in classes.iter().enumerate() {
                        let cs = &stats.per_class[class];
                        // "-" once a class has never reached any Arena wave
                        // yet (arena_furthest_level 0) - "L0 W0" would read
                        // as a claim that isn't true.
                        let arena_col = if cs.arena_furthest_level > 0 {
                            format!("L{} W{}", cs.arena_furthest_level, cs.arena_furthest_wave)
                        } else {
                            "-".to_string()
                        };
                        let row = format!(
                            "{:<3} {:<14} {:>6} {:>6} {:>7} {:>11} {:>9}",
                            format!("{})", i + 1),
                            class,
                            cs.games_played,
                            cs.games_won,
                            cs.enemies_killed,
                            cs.deepest_level + 1,
                            arena_col,
                        );
                        ctx.print_color_centered(15 + i as i32, WHITE, BLACK, &row);
                    }
                    ctx.print_color_centered(
                        15 + classes.len() as i32 + 2,
                        GRAY,
                        BLACK,
                        "Press a number to see that class's ability usage",
                    );
                }
                ctx.print_color_centered(
                    15 + classes.len().max(1) as i32 + 3,
                    GRAY,
                    BLACK,
                    "Press I for items used, ESC to go back",
                );

                let chosen = match ctx.key {
                    Some(VirtualKeyCode::Key1) => Some(0),
                    Some(VirtualKeyCode::Key2) => Some(1),
                    Some(VirtualKeyCode::Key3) => Some(2),
                    Some(VirtualKeyCode::Key4) => Some(3),
                    Some(VirtualKeyCode::Key5) => Some(4),
                    Some(VirtualKeyCode::Key6) => Some(5),
                    Some(VirtualKeyCode::Key7) => Some(6),
                    Some(VirtualKeyCode::Key8) => Some(7),
                    Some(VirtualKeyCode::Key9) => Some(8),
                    _ => None,
                };
                if let Some(class) = chosen.and_then(|i| classes.get(i)) {
                    self.stats_view_mode = StatsViewMode::ClassAbilities(class.clone());
                } else if ctx.key == Some(VirtualKeyCode::I) {
                    self.stats_view_mode = StatsViewMode::ItemUsage;
                } else if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.resources.insert(TurnState::TitleScreen);
                }
            }
            StatsViewMode::ClassAbilities(class) => {
                ctx.print_color_centered(8, WHITE, BLACK, &format!("{} - Ability Usage", class));

                let empty = ClassStats::default();
                let cs = stats.per_class.get(&class).unwrap_or(&empty);
                let mut abilities: Vec<(&String, &u32)> = cs.ability_uses.iter().collect();
                abilities.sort_by(|a, b| a.0.cmp(b.0));

                if abilities.is_empty() {
                    ctx.print_color_centered(13, GRAY, BLACK, "No abilities used yet.");
                } else {
                    for (i, (name, count)) in abilities.iter().enumerate() {
                        ctx.print_color(
                            16,
                            13 + i as i32,
                            WHITE,
                            BLACK,
                            &format!("{}: {}", name, count),
                        );
                    }
                }
                ctx.print_color_centered(
                    13 + abilities.len() as i32 + 2,
                    GRAY,
                    BLACK,
                    "Press ESC to go back",
                );

                if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.stats_view_mode = StatsViewMode::Overview;
                }
            }
            StatsViewMode::ItemUsage => {
                ctx.print_color_centered(8, WHITE, BLACK, "Items Used");

                let mut items: Vec<(&String, &u32)> = stats.item_uses.iter().collect();
                items.sort_by(|a, b| a.0.cmp(b.0));

                if items.is_empty() {
                    ctx.print_color_centered(13, GRAY, BLACK, "No items used yet.");
                } else {
                    for (i, (name, count)) in items.iter().enumerate() {
                        ctx.print_color(
                            16,
                            13 + i as i32,
                            WHITE,
                            BLACK,
                            &format!("{}: {}", name, count),
                        );
                    }
                }
                ctx.print_color_centered(
                    13 + items.len() as i32 + 2,
                    GRAY,
                    BLACK,
                    "Press ESC to go back",
                );

                if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.stats_view_mode = StatsViewMode::Overview;
                }
            }
        }
    }
}
