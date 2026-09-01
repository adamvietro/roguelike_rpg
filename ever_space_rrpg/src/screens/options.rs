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
                {
                    let keymap = self
                        .resources
                        .get::<Keymap>()
                        .expect("Keymap resource missing");
                    for (i, action) in Action::ALL.iter().enumerate() {
                        let row = 10 + i as i32;
                        let key_label = format!("{:?}", keymap.key_for(*action));
                        ctx.print_color(
                            30,
                            row,
                            WHITE,
                            BLACK,
                            &format!("{}) {}: {}", i + 1, action.label(), key_label),
                        );
                    }
                } // keymap's borrow of self.resources ends here, before
                  // the possible self.resources.insert(...) below.

                // Battle Speed and ATB Mode - their own rows directly
                // below the rebind list, numbered as two more menu
                // entries (5, 6) rather than a separate section, so
                // "press a number to act on that row" stays a single
                // consistent rule across this whole screen.
                let battle_speed_row = 10 + Action::ALL.len() as i32;
                {
                    let battle_speed = self
                        .resources
                        .get::<BattleSpeed>()
                        .expect("BattleSpeed resource missing");
                    ctx.print_color(
                        30,
                        battle_speed_row,
                        WHITE,
                        BLACK,
                        &format!(
                            "{}) Battle Speed: {} (press to cycle)",
                            Action::ALL.len() + 1,
                            battle_speed.label()
                        ),
                    );
                }
                let atb_mode_row = battle_speed_row + 1;
                {
                    let atb_mode = self
                        .resources
                        .get::<AtbMode>()
                        .expect("AtbMode resource missing");
                    ctx.print_color(
                        30,
                        atb_mode_row,
                        WHITE,
                        BLACK,
                        &format!(
                            "{}) ATB Mode: {} (press to toggle)",
                            Action::ALL.len() + 2,
                            atb_mode.label()
                        ),
                    );
                }

                let total_rows = Action::ALL.len() as i32 + 2;
                ctx.print_color_centered(
                    10 + total_rows + 2,
                    GRAY,
                    BLACK,
                    "Press R to reset all keys to defaults",
                );
                ctx.print_color_centered(10 + total_rows + 3, GRAY, BLACK, "Press ESC to go back");

                let chosen = match ctx.key {
                    Some(VirtualKeyCode::Key1) => Action::ALL.get(0),
                    Some(VirtualKeyCode::Key2) => Action::ALL.get(1),
                    Some(VirtualKeyCode::Key3) => Action::ALL.get(2),
                    Some(VirtualKeyCode::Key4) => Action::ALL.get(3),
                    _ => None,
                };
                if let Some(action) = chosen {
                    self.options_awaiting = Some(*action);
                } else if ctx.key == Some(VirtualKeyCode::Key5) {
                    let mut speed = self
                        .resources
                        .get_mut::<BattleSpeed>()
                        .expect("BattleSpeed resource missing");
                    *speed = speed.next();
                    let saved = *speed;
                    drop(speed);
                    saved.save();
                } else if ctx.key == Some(VirtualKeyCode::Key6) {
                    let mut mode = self
                        .resources
                        .get_mut::<AtbMode>()
                        .expect("AtbMode resource missing");
                    *mode = mode.next();
                    let saved = *mode;
                    drop(mode);
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
