use crate::prelude::*;
use crate::State;

const LOOT_HEADER_ROW: i32 = 2;
const LOOT_BODY_START_ROW: i32 = 24;

impl State {
    /// Called from main.rs's tick() dispatcher, so this needs to be `pub`.
    /// Styled exactly like Paused per the user's request: reuses
    /// pause_systems (see build_pause_scheduler), which only runs
    /// map_render, so the dungeon's tiles stay frozen underneath while
    /// the player/enemy/item sprites drawn on top of them last frame
    /// simply aren't redrawn - the same "everything but the map
    /// disappears" effect Pause already relies on, rather than building a
    /// second, identical scheduler just for this screen.
    pub fn chest_loot_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        let loot_snapshot = self.resources.get::<Option<ChestLoot>>().unwrap().clone();
        let loot = match loot_snapshot {
            Some(l) => l,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        // Header on BIG_TEXT_CONSOLE, body on HUD_CONSOLE - the same split
        // Paused/Options/History already use for a real font size instead
        // of the old tiny 8px console.
        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(LOOT_HEADER_ROW, YELLOW, BLACK, "Treasure Chest!");

        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color_centered(
            LOOT_BODY_START_ROW,
            YELLOW,
            BLACK,
            &format!("You found: {} gold, {}!", loot.gold, loot.items.join(", ")),
        );
        ctx.print_color_centered(
            LOOT_BODY_START_ROW + 3,
            YELLOW,
            BLACK,
            "Press ENTER to continue.",
        );

        if ctx.key == Some(VirtualKeyCode::Return) {
            self.resources.insert(None::<ChestLoot>);
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}
