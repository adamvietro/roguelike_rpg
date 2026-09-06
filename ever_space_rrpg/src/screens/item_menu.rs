use crate::prelude::*;
use crate::State;

impl State {
    /// The Item Menu (press M during dungeon exploration) - a browsable,
    /// arrow-key-navigable list of the player's universal consumables
    /// (components::usable_item_groups - Healing Potion, Dungeon Map,
    /// any future item every class can carry). Class-restricted
    /// abilities never appear in this LIST (they still get their own
    /// reference panel on the right - see below), and are used via the
    /// Ability Bar's number keys instead - see components::
    /// ability_bar_slots/systems/hud.rs.
    ///
    /// Left side: the usable item list, with the currently-selected
    /// item's own Description shown below it, updating live as the
    /// cursor moves - the keyboard-driven counterpart to the Ability
    /// Bar's mouse-hover tooltip. Right side: a read-only reference
    /// panel of every carried Battle Attack, every class ability (owned
    /// or not, greyed out the same way the Ability Bar does), and every
    /// carried weapon - nothing here is selectable, it's just so a
    /// player checking their potions doesn't ALSO have to remember what
    /// else they're carrying. The dungeon-exploration screen itself
    /// deliberately shows none of this anymore (see systems/hud.rs) -
    /// this menu is now the one place to check it all.
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

        // Lower on the screen (console 2/FINE_TEXT_CONSOLE has 100 rows
        // total) rather than hovering around the middle - leaves the
        // actual dungeon view above it more visible while the menu's
        // open.
        const TITLE_ROW: i32 = 55;
        const LIST_START_ROW: i32 = 59;
        // Right-side reference panel - anchored to the console's own
        // right edge (print_color_right), same convention the dungeon
        // HUD's old Battle Attacks/Weapons panels used to use on
        // HUD_CONSOLE, just on this console's own wider 160-column grid
        // instead. A few columns of margin from the true right edge so
        // text doesn't print flush against it.
        const RIGHT_COL_X: i32 = FINE_TEXT_COLS - 4;
        const RIGHT_COL_START_ROW: i32 = TITLE_ROW;

        ctx.set_active_console(FINE_TEXT_CONSOLE);
        ctx.print_color_centered(TITLE_ROW, YELLOW, BLACK, "-- Items --");

        let mut selected_description: Option<String> = None;

        if groups.is_empty() {
            ctx.print_color_centered(LIST_START_ROW, GRAY, BLACK, "Nothing to use right now.");
        } else {
            self.item_menu_cursor = menu_nav(ctx.key, self.item_menu_cursor, groups.len());
            for (i, (name, count, entity)) in groups.iter().enumerate() {
                let selected = self.item_menu_cursor == i;
                // FINE_TEXT_COLS (this console's real width, 160), NOT
                // HUD_COLS (107, a different console's width) - passing
                // the wrong one here previously put the selection
                // pointer glyph at a nonsense column, nowhere near the
                // actual list.
                print_menu_row_centered(
                    ctx,
                    FINE_TEXT_COLS,
                    LIST_START_ROW + i as i32,
                    WHITE,
                    &format!("{}) {} x{}", i + 1, name, count),
                    selected,
                );
                if selected {
                    selected_description = Some(
                        self.ecs
                            .entry_ref(*entity)
                            .ok()
                            .and_then(|entry| entry.get_component::<Description>().ok().cloned())
                            .map(|d| d.0)
                            .unwrap_or_else(|| "No description.".to_string()),
                    );
                }
            }
        }

        let list_end_row = LIST_START_ROW + groups.len().max(1) as i32;
        let mut next_row = list_end_row + 1;
        if let Some(description) = selected_description {
            // Wrapped rather than a single print_color_centered call -
            // some items' real descriptions are long enough to run off
            // both edges of the screen on one line (same problem the
            // Ability Bar's hover tooltip had - see systems/hud.rs).
            for line in wrap_text(&description, 90) {
                ctx.print_color_centered(next_row, WHITE, BLACK, &line);
                next_row += 1;
            }
        }
        ctx.print_color_centered(
            next_row + 1,
            GRAY,
            BLACK,
            "Arrows to navigate, Enter to use, ESC to close",
        );

        // Right-side reference panel: every Battle Attack carried (used
        // from the battle menu, not out here - see components::
        // usable_carried_items' own BattleItem exclusion), every class
        // ability (greyed out if not currently owned, exactly like the
        // Ability Bar), and every carried weapon - read-only, nothing
        // here responds to the cursor or Enter. Moved here from the
        // dungeon-exploration HUD so that screen stays clean besides the
        // Ability Bar - see systems/hud.rs.
        let mut right_row = RIGHT_COL_START_ROW;
        let mut battle_item_counts: Vec<(String, i32)> = Vec::new();
        <(&Carried, &BattleItem, &Name)>::query()
            .iter(&self.ecs)
            .filter(|(carried, _, _)| carried.0 == player)
            .for_each(|(_, _, name)| {
                match battle_item_counts.iter_mut().find(|(n, _)| *n == name.0) {
                    Some(entry) => entry.1 += 1,
                    None => battle_item_counts.push((name.0.clone(), 1)),
                }
            });
        if !battle_item_counts.is_empty() {
            ctx.print_color_right(RIGHT_COL_X, right_row, YELLOW, BLACK, "Battle Attacks");
            right_row += 1;
            for (name, count) in &battle_item_counts {
                ctx.print_color_right(
                    RIGHT_COL_X,
                    right_row,
                    GREEN,
                    BLACK,
                    format!("{} x{}", name, count),
                );
                right_row += 1;
            }
            right_row += 1; // blank row before Abilities
        }

        if let Some(class) = entity_class(&self.ecs, player) {
            let roster = class_effect_names(&class);
            if !roster.is_empty() {
                ctx.print_color_right(RIGHT_COL_X, right_row, YELLOW, BLACK, "Abilities");
                right_row += 1;
                for slot in ability_bar_slots(&self.ecs, player, &class, &roster) {
                    let (count, color) = match slot.owned {
                        Some((count, _)) => (count, GREEN),
                        None => (0, DARK_GRAY),
                    };
                    ctx.print_color_right(
                        RIGHT_COL_X,
                        right_row,
                        color,
                        BLACK,
                        format!("{} x{}", slot.name, count),
                    );
                    right_row += 1;
                }
                right_row += 1; // blank row before Weapons
            }
        }

        let mut weapon_counts: Vec<(String, i32)> = Vec::new();
        <(&Carried, &Weapon, &Name)>::query()
            .iter(&self.ecs)
            .filter(|(carried, _, _)| carried.0 == player)
            .for_each(
                |(_, _, name)| match weapon_counts.iter_mut().find(|(n, _)| *n == name.0) {
                    Some(entry) => entry.1 += 1,
                    None => weapon_counts.push((name.0.clone(), 1)),
                },
            );
        if !weapon_counts.is_empty() {
            ctx.print_color_right(RIGHT_COL_X, right_row, YELLOW, BLACK, "Weapons");
            right_row += 1;
            for (name, count) in &weapon_counts {
                ctx.print_color_right(
                    RIGHT_COL_X,
                    right_row,
                    GREEN,
                    BLACK,
                    format!("{} x{}", name, count),
                );
                right_row += 1;
            }
        }

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
