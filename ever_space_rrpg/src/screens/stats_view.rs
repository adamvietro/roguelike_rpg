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

        ctx.set_active_console(2);
        ctx.print_color_centered(45, YELLOW, BLACK, "-- History --");

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
                    46,
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
                    47,
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
                    ctx.print_color_centered(50, GRAY, BLACK, "No games played yet.");
                } else {
                    // Fixed-width columns, printed via print_color_centered -
                    // every row (header included) formats to the exact same
                    // total length, so centering each line independently
                    // still lines every column up under the one above it.
                    // Widths: "#) " (3) + Class (12) + Runs (6) + Won (6) +
                    // Kills (7) + Dungeon Lvl (11) + Arena Lvl (10) = 55.
                    let header = format!(
                        "{:<3}{:<12}{:>6}{:>6}{:>7}{:>11}{:>10}",
                        "", "Class", "Runs", "Won", "Kills", "Dungeon Lvl", "Arena Lvl"
                    );
                    ctx.print_color_centered(49, YELLOW, BLACK, &header);

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
                            "{:<3}{:<12}{:>6}{:>6}{:>7}{:>11}{:>10}",
                            format!("{})", i + 1),
                            class,
                            cs.games_played,
                            cs.games_won,
                            cs.enemies_killed,
                            cs.deepest_level + 1,
                            arena_col,
                        );
                        ctx.print_color_centered(50 + i as i32, WHITE, BLACK, &row);
                    }
                    ctx.print_color_centered(
                        50 + classes.len() as i32 + 1,
                        GRAY,
                        BLACK,
                        "Press a number to see that class's ability usage",
                    );
                }
                ctx.print_color_centered(
                    50 + classes.len() as i32 + 2,
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
                ctx.print_color_centered(47, WHITE, BLACK, &format!("{} - Ability Usage", class));

                let empty = ClassStats::default();
                let cs = stats.per_class.get(&class).unwrap_or(&empty);
                let mut abilities: Vec<(&String, &u32)> = cs.ability_uses.iter().collect();
                abilities.sort_by(|a, b| a.0.cmp(b.0));

                if abilities.is_empty() {
                    ctx.print_color_centered(50, GRAY, BLACK, "No abilities used yet.");
                } else {
                    for (i, (name, count)) in abilities.iter().enumerate() {
                        ctx.print_color(
                            16,
                            50 + i as i32,
                            WHITE,
                            BLACK,
                            &format!("{}: {}", name, count),
                        );
                    }
                }
                ctx.print_color_centered(
                    50 + abilities.len() as i32 + 2,
                    GRAY,
                    BLACK,
                    "Press ESC to go back",
                );

                if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.stats_view_mode = StatsViewMode::Overview;
                }
            }
            StatsViewMode::ItemUsage => {
                ctx.print_color_centered(47, WHITE, BLACK, "Items Used");

                let mut items: Vec<(&String, &u32)> = stats.item_uses.iter().collect();
                items.sort_by(|a, b| a.0.cmp(b.0));

                if items.is_empty() {
                    ctx.print_color_centered(50, GRAY, BLACK, "No items used yet.");
                } else {
                    for (i, (name, count)) in items.iter().enumerate() {
                        ctx.print_color(
                            16,
                            50 + i as i32,
                            WHITE,
                            BLACK,
                            &format!("{}: {}", name, count),
                        );
                    }
                }
                ctx.print_color_centered(
                    50 + items.len() as i32 + 2,
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
