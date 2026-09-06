use crate::prelude::*;
use crate::State;

impl State {
    /// The rebindable-keys screen, reached from Paused (press O) or the
    /// title screen (press O), returning to whichever one it was opened
    /// from on Escape - see options_return_to. Reuses pause_systems to
    /// redraw the map underneath, same as paused_tick - nothing should
    /// move or animate while this menu is open either; from the title
    /// screen this just redraws the same decorative background
    /// title_screen/class_select already show, since spawn_title_background
    /// leaves Map/Camera/theme in place for as long as this ecs/resources
    /// pair is alive (i.e. until start_game or return_to_title next
    /// wipes them).
    ///
    /// Two sub-modes, tracked by `self.options_awaiting`:
    /// - None (browsing): shows every Action and its current key, plus a
    ///   Battle Speed row and an ATB Mode row below them. A number key
    ///   1-4 selects an Action and enters capture mode; 5 cycles Battle
    ///   Speed (Slow -> Normal -> Fast -> Slow); 6 toggles ATB Mode (Wait
    ///   <-> True ATB) - see BattleSpeed::next/AtbMode::next. Neither 5
    ///   nor 6 needs a capture sub-mode, since both are a small fixed set
    ///   of choices rather than an arbitrary key. R resets every Action
    ///   to its default key (see Keymap::reset_to_defaults) without
    ///   entering capture mode at all.
    /// - Some(action) (capturing): shows a prompt; the next recognized
    ///   key (see keymap::is_rebindable_key) rebinds `action` to it,
    ///   saves to disk, and returns to browsing. Escape cancels back to
    ///   browsing without changing anything. Rebinding onto a key another
    ///   action already holds swaps the two rather than erroring or
    ///   crashing - see Keymap::rebind.
    pub fn options_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, YELLOW, BLACK, "-- Options --");

        // HUD_CONSOLE (~12px cells) for the body, matching every other
        // pre-game screen's font bump - see title_screen's comment on the
        // same change. Row numbers are fresh picks for HUD_CONSOLE's
        // 67-row grid, not a mechanical conversion of the old console 2
        // values.
        ctx.set_active_console(HUD_CONSOLE);

        match self.options_awaiting {
            None => {
                let total_rows = Action::ALL.len() + 3;
                self.options_cursor = menu_nav(ctx.key, self.options_cursor, total_rows);

                {
                    let keymap = self
                        .resources
                        .get::<Keymap>()
                        .expect("Keymap resource missing");
                    for (i, action) in Action::ALL.iter().enumerate() {
                        let row = 10 + i as i32;
                        let key_label = format!("{:?}", keymap.key_for(*action));
                        print_menu_row_left(
                            ctx,
                            30,
                            row,
                            WHITE,
                            &format!("{}) {}: {}", i + 1, action.label(), key_label),
                            self.options_cursor == i,
                        );
                    }
                } // keymap's borrow of self.resources ends here, before
                  // the possible self.resources.insert(...) below.

                // Battle Speed and ATB Mode - their own rows directly
                // below the rebind list, numbered as two more menu
                // entries (5, 6) rather than a separate section, so
                // "press a number to act on that row" stays a single
                // consistent rule across this whole screen. Same rows
                // are also reachable via the arrow-key cursor (indices
                // Action::ALL.len() and Action::ALL.len()+1).
                let battle_speed_row = 10 + Action::ALL.len() as i32;
                {
                    let battle_speed = self
                        .resources
                        .get::<BattleSpeed>()
                        .expect("BattleSpeed resource missing");
                    print_menu_row_left(
                        ctx,
                        30,
                        battle_speed_row,
                        WHITE,
                        &format!(
                            "{}) Battle Speed: {} (press to cycle)",
                            Action::ALL.len() + 1,
                            battle_speed.label()
                        ),
                        self.options_cursor == Action::ALL.len(),
                    );
                }
                let atb_mode_row = battle_speed_row + 1;
                {
                    let atb_mode = self
                        .resources
                        .get::<AtbMode>()
                        .expect("AtbMode resource missing");
                    print_menu_row_left(
                        ctx,
                        30,
                        atb_mode_row,
                        WHITE,
                        &format!(
                            "{}) ATB Mode: {} (press to toggle)",
                            Action::ALL.len() + 2,
                            atb_mode.label()
                        ),
                        self.options_cursor == Action::ALL.len() + 1,
                    );
                }
                // Battle menu cursor memory - whether the battle menu's
                // arrow-cursor should remember the last action chosen
                // per class and start there next time (see
                // settings::MenuMemory/LastBattleAction and
                // screens/battle.rs's one-time cursor seed). A third row
                // in this same numbered/arrow-navigable list, same
                // "press to toggle" shape as ATB Mode just above.
                let menu_memory_row = atb_mode_row + 1;
                {
                    let menu_memory = self
                        .resources
                        .get::<MenuMemory>()
                        .expect("MenuMemory resource missing");
                    print_menu_row_left(
                        ctx,
                        30,
                        menu_memory_row,
                        WHITE,
                        &format!(
                            "{}) Remember Last Battle Action: {} (press to toggle)",
                            Action::ALL.len() + 3,
                            menu_memory.label()
                        ),
                        self.options_cursor == Action::ALL.len() + 2,
                    );
                }

                ctx.print_color_centered(
                    10 + total_rows as i32 + 2,
                    GRAY,
                    BLACK,
                    "Press R to reset all keys to defaults",
                );
                ctx.print_color_centered(
                    10 + total_rows as i32 + 3,
                    GRAY,
                    BLACK,
                    "Arrows to navigate, Enter to select, ESC to go back",
                );

                let chosen_row = match ctx.key {
                    Some(VirtualKeyCode::Key1) => Some(0),
                    Some(VirtualKeyCode::Key2) => Some(1),
                    Some(VirtualKeyCode::Key3) => Some(2),
                    Some(VirtualKeyCode::Key4) => Some(3),
                    Some(VirtualKeyCode::Key5) => Some(4),
                    Some(VirtualKeyCode::Key6) => Some(5),
                    Some(VirtualKeyCode::Key7) => Some(6),
                    Some(VirtualKeyCode::Return) => Some(self.options_cursor),
                    _ => None,
                };

                if let Some(action) = chosen_row.and_then(|i| Action::ALL.get(i)) {
                    self.options_awaiting = Some(*action);
                } else if chosen_row == Some(Action::ALL.len()) {
                    let mut speed = self
                        .resources
                        .get_mut::<BattleSpeed>()
                        .expect("BattleSpeed resource missing");
                    *speed = speed.next();
                    let saved = *speed;
                    drop(speed);
                    saved.save();
                } else if chosen_row == Some(Action::ALL.len() + 1) {
                    let mut mode = self
                        .resources
                        .get_mut::<AtbMode>()
                        .expect("AtbMode resource missing");
                    *mode = mode.next();
                    let saved = *mode;
                    drop(mode);
                    saved.save();
                } else if chosen_row == Some(Action::ALL.len() + 2) {
                    let mut memory = self
                        .resources
                        .get_mut::<MenuMemory>()
                        .expect("MenuMemory resource missing");
                    *memory = memory.next();
                    let saved = *memory;
                    drop(memory);
                    saved.save();
                } else if ctx.key == Some(VirtualKeyCode::R) {
                    let mut keymap = self
                        .resources
                        .get_mut::<Keymap>()
                        .expect("Keymap resource missing");
                    keymap.reset_to_defaults();
                    keymap.save();
                } else if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.resources.insert(self.options_return_to);
                }
            }
            Some(action) => {
                ctx.print_color_centered(
                    10,
                    YELLOW,
                    BLACK,
                    &format!("Press a new key for {}...", action.label()),
                );
                ctx.print_color_centered(13, GRAY, BLACK, "(arrow keys or A-Z; ESC to cancel)");

                match ctx.key {
                    Some(VirtualKeyCode::Escape) => {
                        self.options_awaiting = None;
                    }
                    Some(key) if is_rebindable_key(key) => {
                        let mut keymap = self
                            .resources
                            .get_mut::<Keymap>()
                            .expect("Keymap resource missing");
                        keymap.rebind(action, key);
                        keymap.save();
                        drop(keymap);
                        self.options_awaiting = None;
                    }
                    _ => {}
                }
            }
        }
    }
}
