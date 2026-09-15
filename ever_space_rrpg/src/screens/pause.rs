use crate::prelude::*;
use crate::State;

/// Rotating "did you know" tips shown on the Paused screen (see
/// paused_tick) - the general how-to-play hint that used to sit
/// permanently on the dungeon-exploration HUD moved here instead, plus
/// several more, cycling one at a time rather than trying to cram all of
/// them onto the dungeon view at once. Order is arbitrary; add/remove
/// freely - nothing else in the game indexes into this by position.
const PAUSE_HINTS: &[&str] = &[
    "Use the arrow keys to move.",
    "Press 'M' to access the Item Menu.",
    "Potions and Maps can be clicked on the Item Bar.",
    "Dungeon Abilities can be clicked, or used with their hotkey.",
    "If you need to heal in a battle, Flee and then use a Potion.",
    "Press Space to wait a turn - any enemy nearby will join the fight.",
    "Hover over any bar icon to see what it does.",
    "Walking onto an item picks it up automatically.",
    "Your portrait tints red when your health is low.",
    "A Dungeon Map reveals the whole level's layout when used.",
    "In battle, use arrow keys and Enter, or number keys, to act.",
];

/// How long each PAUSE_HINTS entry stays on screen before advancing to
/// the next one.
const PAUSE_HINT_INTERVAL_MS: f32 = 4000.0;

/// The Paused screen's arrow-key-navigable menu, in cursor order - matches
/// every other menu screen's convention (Adventure Select, Class Select,
/// Options, History, Item Menu) instead of being the one screen still
/// using individual dedicated keys only. ESC still resumes directly as a
/// shortcut too (see paused_tick), it just isn't the only way anymore.
const PAUSE_MENU_ITEMS: [&str; 3] = ["Resume", "Options", "Quit to the Title Screen"];
const PAUSE_MENU_START_ROW: i32 = 20;

/// The hints box, lower third of the screen (HUD_CONSOLE is 67 rows tall;
/// 45 is roughly 800 * 2/3 converted to this console's row units) -
/// separate from the menu above it with plenty of empty space between, so
/// the frozen dungeon map is still visible through the middle of the
/// pause overlay the way it always has been.
const HINT_BOX_WIDTH: i32 = 90;
const HINT_BOX_HEIGHT: i32 = 7;
const HINT_BOX_X: i32 = (HUD_COLS - HINT_BOX_WIDTH) / 2;
const HINT_BOX_Y: i32 = 45;

impl State {
    /// Redraws the dungeon map beneath a pause overlay - see
    /// build_pause_scheduler for why this only re-runs map_render rather
    /// than the full input schedule (nothing should move, animate, or
    /// otherwise change while paused). Called from main.rs's tick()
    /// dispatcher, so this needs to be `pub`.
    pub fn paused_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        // Header on BIG_TEXT_CONSOLE (32px cells), body on HUD_CONSOLE
        // (~12px cells) - the same split every other pre-game/menu screen
        // (Options, History) already uses for a font bump over the old
        // tiny 8px console 2 this screen used to be drawn on entirely.
        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, YELLOW, BLACK, "-- Paused --");

        ctx.set_active_console(HUD_CONSOLE);

        // pause_cursor deliberately does NOT reset to 0 every time Paused
        // is freshly entered - same "small, harmless convenience" call
        // already made for item_menu_cursor, and for the same structural
        // reason: the transition INTO this screen happens inside
        // player_input's ECS system (on Escape), which has no access to
        // plain State fields like this one to reset them, only Paused's
        // own return_to_title reset point does.
        self.pause_cursor = menu_nav(ctx.key, self.pause_cursor, PAUSE_MENU_ITEMS.len());
        for (i, label) in PAUSE_MENU_ITEMS.iter().enumerate() {
            print_menu_row_centered(
                ctx,
                HUD_COLS,
                PAUSE_MENU_START_ROW + i as i32,
                WHITE,
                label,
                self.pause_cursor == i,
            );
        }
        ctx.print_color_centered(
            PAUSE_MENU_START_ROW + PAUSE_MENU_ITEMS.len() as i32 + 1,
            GRAY,
            BLACK,
            "Arrow keys + Enter to select - ESC always resumes",
        );

        // Advances pause_hint_index on a real-time timer rather than a
        // per-frame counter, so the rotation speed doesn't depend on
        // frame rate - same reasoning background_move_timer_ms already
        // uses for the title screen's decorative background.
        self.pause_hint_timer_ms += ctx.frame_time_ms;
        if self.pause_hint_timer_ms >= PAUSE_HINT_INTERVAL_MS {
            self.pause_hint_timer_ms -= PAUSE_HINT_INTERVAL_MS;
            self.pause_hint_index = (self.pause_hint_index + 1) % PAUSE_HINTS.len();
        }
        let hint = PAUSE_HINTS[self.pause_hint_index];

        // The real PixelLab panel border (item 10 in docs/ideas.md),
        // Swamp - direct request 2026-09-14 ("I want the tooltips to be
        // the wooden and green corners. As well as the hints bar.").
        // Converted to render_helpers::PanelBox 2026-09-14 alongside its
        // wider rollout - `text_color_centered` centers on PANEL_TEXT_
        // CONSOLE's own full width, not box-relative, which lines up
        // with the box visually only because the box itself is ALSO
        // centered on that same HUD_COLS-wide grid (see HINT_BOX_X) -
        // see that method's own doc comment in render_helpers.rs.
        let mut hint_box = PanelBox::new(
            HINT_BOX_X,
            HINT_BOX_Y,
            HINT_BOX_WIDTH,
            HINT_BOX_HEIGHT,
            UiPanelTheme::Swamp,
            PIXEL_BOX_TILE_SCALE,
        );
        hint_box.text_color_centered(0, YELLOW, BLACK, "Hints");
        hint_box.text_color_centered(2, WHITE, BLACK, hint);
        hint_box.submit();

        if ctx.key == Some(VirtualKeyCode::Escape) {
            self.resources.insert(TurnState::AwaitingInput);
            return;
        }

        // O and Q keep working directly too, same as before - purely
        // additive, same convention Adventure Select/Class Select already
        // established when THEY gained cursor navigation (see DEVLOG):
        // old shortcuts never stop working just because a cursor exists
        // now.
        if ctx.key == Some(VirtualKeyCode::O) {
            self.options_return_to = TurnState::Paused;
            self.options_cursor = MenuCursor::new();
            self.resources.insert(TurnState::Options);
            return;
        }
        if ctx.key == Some(VirtualKeyCode::Q) {
            self.return_to_title();
            return;
        }

        if ctx.key == Some(VirtualKeyCode::Return) {
            match self.pause_cursor {
                0 => {
                    self.resources.insert(TurnState::AwaitingInput);
                }
                1 => {
                    self.options_return_to = TurnState::Paused;
                    self.options_cursor = MenuCursor::new();
                    self.resources.insert(TurnState::Options);
                }
                2 => {
                    self.return_to_title();
                }
                _ => {}
            }
        }
    }
}
