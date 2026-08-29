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
    /// Two sub-modes, tracked by `self.stats_selected_class`:
    /// - None (overview): overall win rate, then one row per class
    ///   that's actually been played at least once - a class never
    ///   played is simply absent rather than shown at 0/0. A number key
    ///   selects a row to drill into that class's ability usage.
    /// - Some(class) (drill-down): every ability that class has used at
    ///   least once, with its count. Escape returns to the overview.
    pub fn stats_view_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        ctx.set_active_console(2);
        ctx.print_color_centered(45, YELLOW, BLACK, "-- History --");

        // Cloned so the borrow of self.resources doesn't outlive this
        // point - both branches below mutate `self` (stats_selected_class
        // or self.resources itself), which a live borrow from
        // self.resources.get::<Stats>() would conflict with.
        let stats = self
            .resources
            .get::<Stats>()
            .expect("Stats resource missing")
            .clone();

        match self.stats_selected_class.clone() {
            None => {
                let win_rate = if stats.games_played > 0 {
                    format!(
                        " ({:.0}%)",
                        100.0 * stats.games_won as f32 / stats.games_played as f32
                    )
                } else {
                    String::new()
                };
                ctx.print_color_centered(
                    47,
                    WHITE,
                    BLACK,
                    &format!(
                        "Overall: {} of {} runs won{}",
                        stats.games_won, stats.games_played, win_rate
                    ),
                );

                let mut classes: Vec<String> = stats.per_class.keys().cloned().collect();
                classes.sort();

                if classes.is_empty() {
                    ctx.print_color_centered(50, GRAY, BLACK, "No games played yet.");
                } else {
                    for (i, class) in classes.iter().enumerate() {
                        let cs = &stats.per_class[class];
                        ctx.print_color(
                            16,
                            50 + i as i32,
                            WHITE,
                            BLACK,
                            &format!(
                                "{}) {}: {} of {} won, {} kills, deepest Level {}",
                                i + 1,
                                class,
                                cs.games_won,
                                cs.games_played,
                                cs.enemies_killed,
                                cs.deepest_level + 1,
                            ),
                        );
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
                    "Press ESC to go back",
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
                    self.stats_selected_class = Some(class.clone());
                } else if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.resources.insert(TurnState::TitleScreen);
                }
            }
            Some(class) => {
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
                    self.stats_selected_class = None;
                }
            }
        }
    }
}
