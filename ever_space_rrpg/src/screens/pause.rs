use crate::prelude::*;
use crate::State;

impl State {
    /// Redraws the dungeon map beneath a pause overlay - see
    /// build_pause_scheduler for why this only re-runs map_render rather
    /// than the full input schedule (nothing should move, animate, or
    /// otherwise change while paused). Escape resumes; Q quits to the
    /// title screen, tearing down the current run. Called from main.rs's
    /// tick() dispatcher, so this needs to be `pub`.
    pub fn paused_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        ctx.set_active_console(2);
        ctx.print_color_centered(45, YELLOW, BLACK, "-- Paused --");
        ctx.print_color_centered(48, WHITE, BLACK, "Press ESC to resume");
        ctx.print_color_centered(49, WHITE, BLACK, "Press Q to quit to the title screen");

        match ctx.key {
            Some(VirtualKeyCode::Escape) => {
                self.resources.insert(TurnState::AwaitingInput);
            }
            Some(VirtualKeyCode::Q) => {
                self.return_to_title();
            }
            _ => {}
        }
    }
}