use crate::prelude::*;
use crate::State;

impl State {
    /// The Item Menu (press M during dungeon exploration) - a browsable,
    /// arrow-key-navigable list of the player's universal consumables
    /// (components::usable_item_groups - Healing Potion, Dungeon Map,
    /// any future item every class can carry). Class-restricted
    /// abilities never appear here - see components::ability_bar_slots/
    /// systems/hud.rs for those instead.
    ///
    /// Redraws the dungeon map beneath the list via pause_systems (same
    /// frozen-map trick Options/Pause already use - nothing should move
    /// while this menu is open), so simply OPENING or browsing this menu
    /// never costs a turn. Using an item DOES cost a turn - see the
    /// Enter/number-key handling below, which queues an ActivateItem
    /// exactly the way player_input's old number-key handling used to,
    /// then jumps straight to TurnState::PlayerTurn (not back through
    /// AwaitingInput) so the dungeon turn actually advances the same
    /// frame the item is used.
    ///
    /// Called from main.rs's tick() dispatcher, so this needs to be
    /// `pub`.
    pub fn item_menu_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .find_map(|(entity, _)| Some(*entity))
            .unwrap();
        let groups = usable_item_groups(&self.ecs, player);

        ctx.set_active_console(2);
        ctx.print_color_centered(40, YELLOW, BLACK, "-- Items --");

        if groups.is_empty() {
            ctx.print_color_centered(44, GRAY, BLACK, "Nothing to use right now.");
        } else {
            self.item_menu_cursor = menu_nav(ctx.key, self.item_menu_cursor, groups.len());
            for (i, (name, count, _)) in groups.iter().enumerate() {
                print_menu_row_centered(
                    ctx,
                    HUD_COLS,
                    44 + i as i32,
                    WHITE,
                    &format!("{}) {} x{}", i + 1, name, count),
                    self.item_menu_cursor == i,
                );
            }
        }
        ctx.print_color_centered(
            44 + groups.len().max(1) as i32 + 2,
            GRAY,
            BLACK,
            "Arrows to navigate, Enter to use, ESC to close",
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
            Some(VirtualKeyCode::Return) if !groups.is_empty() => Some(self.item_menu_cursor),
            _ => None,
        };

        if let Some((_, _, item_entity)) = chosen.and_then(|i| groups.get(i)) {
            let mut commands = CommandBuffer::new(&self.ecs);
            commands.push((
                (),
                ActivateItem {
                    used_by: player,
                    item: *item_entity,
                },
            ));
            commands.flush(&mut self.ecs);
            // Using an item genuinely costs a turn - straight to
            // PlayerTurn (not AwaitingInput), matching exactly how the
            // old direct number-key press used to advance the dungeon
            // turn. use_items (run during PlayerTurn) applies the actual
            // effect and consumes the item.
            self.resources.insert(TurnState::PlayerTurn);
        } else if ctx.key == Some(VirtualKeyCode::Escape) {
            // Just closing the menu - no item used, no turn spent.
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}
