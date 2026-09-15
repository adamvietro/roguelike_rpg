use crate::prelude::*;
use crate::State;

/// The Item Menu (press M during dungeon exploration) - a full character
/// dashboard, not just a potion list: Items (1), Equipped Items (2),
/// Stats (3), Battle Actions (4), and Dungeon Actions (5), plus a shared
/// description panel (1.1) for whichever entry the cursor currently
/// sits on. All on HUD_CONSOLE (no BIG_TEXT_CONSOLE header this screen -
/// tried one first, but the two consoles' cells overlap in real pixel
/// terms and a header positioned as casually as this one was visibly
/// collided with the top boxes' border; simpler to drop it than solve
/// that clearance math for a title that didn't add much anyway).
///
/// Layout, in HUD_CONSOLE cells (a 107x67 grid):
/// - Left column (LEFT_X..LEFT_X+LEFT_WIDTH): box 1 (Items) then box 2
///   (Equipped Items) stacked, top to bottom.
/// - Right column (RIGHT_X..RIGHT_X+RIGHT_WIDTH): box 4 (Battle Actions)
///   then box 5 (Dungeon Actions) stacked, aligned with the left column's
///   two boxes.
/// - A full-width description strip (1.1) between the top and bottom
///   rows of boxes, shared by both columns.
/// - Box 3 (Stats) sits inside the left column's lower half, box 5
///   spans the full height of the right column's lower half (matching
///   box 2 + box 3's combined height) - see BOTTOM_HEIGHT.
/// - TOP_Y is picked so the whole block (TOP_Y through the footer hint,
///   49 rows total) sits vertically centered in HUD_CONSOLE's 67-row
///   height - see TOP_Y's own doc comment for the arithmetic.
///
/// All first-pass pixel guesses (see CLAUDE.md's bracket-lib layout
/// gotcha) pending a screenshot round.
const LEFT_X: i32 = 3;
const LEFT_WIDTH: i32 = 50;
const RIGHT_X: i32 = 56;
const RIGHT_WIDTH: i32 = 48;

/// Gap between the top row of boxes and the description strip, and
/// between the description strip and the bottom row - was 1 row (a hair
/// wide enough for the old plain ASCII border, but too tight once real
/// pixel-art panel borders shipped 2026-09-14 - two stone frames only a
/// single row apart read as touching/overlapping). Widened to 3 per
/// direct feedback ("add more space between the boxes").
const BOX_GAP: i32 = 3;
/// Gap between Equipped Items and Stats specifically - these two stack
/// directly in the left column's own lower half (see this module's doc
/// comment), and had NO gap at all before 2026-09-14's pixel-art
/// borders shipped (their two frames shared a seam outright). Narrower
/// than BOX_GAP since this is a sub-stack within one column, not a full
/// section break.
const SUB_GAP: i32 = 2;

/// (67 - 49) / 2 = 9 - centers the whole 49-row block (see this module's
/// doc comment) in HUD_CONSOLE's 67 rows. 49 is TOP_HEIGHT(11) +
/// BOX_GAP(3) + DESC_HEIGHT(8) + BOX_GAP(3) + EQUIPPED_HEIGHT(6) +
/// SUB_GAP(2) + STATS_HEIGHT(13) + gap(2) + the footer hint's own row(1),
/// matching the actual constants below - recompute this by hand if any
/// of those change, since it's not (yet) derived from them automatically.
const TOP_Y: i32 = 9;
const TOP_HEIGHT: i32 = 11;

const DESC_Y: i32 = TOP_Y + TOP_HEIGHT + BOX_GAP;
const DESC_HEIGHT: i32 = 8;
const DESC_X: i32 = LEFT_X;
const DESC_WIDTH: i32 = RIGHT_X + RIGHT_WIDTH - LEFT_X;

const BOTTOM_Y: i32 = DESC_Y + DESC_HEIGHT + BOX_GAP;
const EQUIPPED_HEIGHT: i32 = 6;
const STATS_Y: i32 = BOTTOM_Y + EQUIPPED_HEIGHT + SUB_GAP;
const STATS_HEIGHT: i32 = 13;
/// Box 5 (Dungeon Actions) spans this full height on its own - matches
/// box 2 + SUB_GAP + box 3's combined height exactly, so the two
/// columns' bottom edges line up.
const BOTTOM_HEIGHT: i32 = EQUIPPED_HEIGHT + SUB_GAP + STATS_HEIGHT;

const FOOTER_Y: i32 = STATS_Y + STATS_HEIGHT + 2;

/// Prints one box's title, border, and entry list - shared by all four
/// navigable boxes (Items/Equipped Items/Battle Actions/Dungeon Actions)
/// since group_items/ability_bar_slots/battle_bar_slots/
/// equipped_weapon_slots all already return the same AbilityBarSlot
/// shape. `selected` is this list's own local index of the cursor, if
/// the cursor is currently anywhere in THIS list (None otherwise) - see
/// selected_local_index in item_menu_tick.
///
/// The real PixelLab border (item 10 in docs/ideas.md) via
/// `render_helpers::PanelBox` - every box on this screen uses it now
/// (converted from a shared hand-rolled `panel_batch`/`text_batch` pair
/// to `PanelBox` 2026-09-14, once that helper was built and confirmed
/// working on the Battle Log box; see `docs/journal.md`'s same-day
/// entries). `PanelBox` already targets the right two consoles
/// internally (`UI_PANEL_CONSOLE` for fill+border, `PANEL_TEXT_CONSOLE`
/// for text) - see that struct's own doc comment in render_helpers.rs
/// for why text needs a console registered LATER than whatever drew the
/// fill it sits on top of, not just later in the same batch.
///
/// Title prints at `dy = -1` (one row ABOVE `y`, i.e. above the box's
/// own top border) rather than `dy = 0` - applying the fix
/// `docs/UI_Panel_Sheet_Guide.md`'s "Still open" list had already
/// diagnosed and left pending: at this screen's box heights/scale
/// (`PIXEL_BOX_TILE_SCALE`, the default, not `_COMPACT`), `dy = 0`
/// landed the title ON the border's own real footprint rather than
/// clearly inside it - a real, confirmed-live regression from the
/// center-shift fix (`pixel_box_tiles`), not the SAME "erased outright"
/// failure an earlier, now-stale attempt at `y + 1` hit before that fix
/// existed (`PANEL_TEXT_CONSOLE` is registered AFTER `UI_PANEL_CONSOLE`,
/// so text on it was never actually at risk of being painted over by
/// the border - only ever a positioning question, not a z-order one).
fn print_box(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    title: &str,
    slots: &[AbilityBarSlot],
    selected: Option<usize>,
) {
    let mut panel_box = PanelBox::new(x, y, width, height, UiPanelTheme::Dungeon, PIXEL_BOX_TILE_SCALE);
    panel_box.text_color_raw(2, -1, YELLOW, BLACK, format!(" {} ", title));

    if slots.is_empty() {
        panel_box.text_color(0, 0, GRAY, BLACK, "Nothing here.");
        panel_box.submit();
        return;
    }

    for (i, slot) in slots.iter().enumerate() {
        let row = i as i32;
        let owned = slot.owned.is_some();
        let label = match slot.owned {
            Some((count, _)) if count > 1 => format!("{} x{}", slot.name, count),
            _ => slot.name.clone(),
        };
        let is_selected = selected == Some(i);
        let color = if is_selected {
            YELLOW
        } else if owned {
            WHITE
        } else {
            GRAY
        };
        if is_selected {
            panel_box.text_color(0, row, color, BLACK, format!("> {}", label));
        } else {
            panel_box.text_color(2, row, color, BLACK, label);
        }
    }
    panel_box.submit();
}

impl State {
    /// See this module's own doc comment for the full layout. Redraws
    /// the dungeon map beneath the menu via pause_systems (same frozen-
    /// map trick Options/Pause already use) - opening or browsing never
    /// costs a turn. Using an Item or a Dungeon Action DOES cost a turn
    /// (queues an ActivateItem and jumps straight to TurnState::
    /// PlayerTurn) - Equipped Items and Battle Actions are browse/
    /// description-only, same restriction they already had before this
    /// redesign (a weapon isn't manually "used," and a Technique can
    /// only ever be used from the battle menu).
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
        let class = entity_class(&self.ecs, player);

        let items: Vec<AbilityBarSlot> = usable_item_groups(&self.ecs, player)
            .into_iter()
            .map(|(name, count, entity)| AbilityBarSlot {
                name,
                owned: Some((count, entity)),
            })
            .collect();
        let equipped = equipped_weapon_slots(&self.ecs, player);

        let (battle_actions, dungeon_actions) = match &class {
            Some(class) => {
                let technique_roster = class_technique_names(class);
                let effect_roster = class_effect_names(class);
                (
                    battle_bar_slots(&self.ecs, player, &technique_roster),
                    ability_bar_slots(&self.ecs, player, class, &effect_roster),
                )
            }
            None => (Vec::new(), Vec::new()),
        };

        let left_len = items.len() + equipped.len();
        let right_len = battle_actions.len() + dungeon_actions.len();

        // Arrow-key navigation - same shape battle::MenuCursor already
        // drives the battle menu with (see screens/battle.rs): Left/Right
        // switches side (remembering the row on each), Up/Down moves
        // within the current side's combined list.
        let current_side_len = if self.item_menu_cursor.col == 0 {
            left_len
        } else {
            right_len
        };
        match ctx.key {
            Some(VirtualKeyCode::Up) => self.item_menu_cursor.move_vertical(-1, current_side_len),
            Some(VirtualKeyCode::Down) => self.item_menu_cursor.move_vertical(1, current_side_len),
            Some(VirtualKeyCode::Left) => self.item_menu_cursor.move_horizontal(0, left_len),
            Some(VirtualKeyCode::Right) => self.item_menu_cursor.move_horizontal(1, right_len),
            _ => {}
        }

        // Which local index within EACH of the 4 lists (if any) the
        // cursor currently sits on - None for every list except
        // whichever one actually contains the cursor's row right now.
        let (items_selected, equipped_selected) = if self.item_menu_cursor.col == 0 {
            let row = self.item_menu_cursor.row;
            if row < items.len() {
                (Some(row), None)
            } else {
                (None, Some(row - items.len()))
            }
        } else {
            (None, None)
        };
        let (battle_selected, dungeon_selected) = if self.item_menu_cursor.col == 1 {
            let row = self.item_menu_cursor.row;
            if row < battle_actions.len() {
                (Some(row), None)
            } else {
                (None, Some(row - battle_actions.len()))
            }
        } else {
            (None, None)
        };

        // The single AbilityBarSlot the cursor is currently on, if any -
        // drives both the description panel and Enter's use-behavior.
        let selected_slot: Option<&AbilityBarSlot> = items_selected
            .and_then(|i| items.get(i))
            .or_else(|| equipped_selected.and_then(|i| equipped.get(i)))
            .or_else(|| battle_selected.and_then(|i| battle_actions.get(i)))
            .or_else(|| dungeon_selected.and_then(|i| dungeon_actions.get(i)));

        ctx.set_active_console(HUD_CONSOLE);
        let mut batch = DrawBatch::new();
        batch.target(HUD_CONSOLE);

        print_box(
            LEFT_X,
            TOP_Y,
            LEFT_WIDTH,
            TOP_HEIGHT,
            "Items",
            &items,
            items_selected,
        );
        print_box(
            RIGHT_X,
            TOP_Y,
            RIGHT_WIDTH,
            TOP_HEIGHT,
            "Battle Actions",
            &battle_actions,
            battle_selected,
        );
        print_box(
            LEFT_X,
            BOTTOM_Y,
            LEFT_WIDTH,
            EQUIPPED_HEIGHT,
            "Equipped Items",
            &equipped,
            equipped_selected,
        );
        print_box(
            RIGHT_X,
            BOTTOM_Y,
            RIGHT_WIDTH,
            BOTTOM_HEIGHT,
            "Dungeon Actions",
            &dungeon_actions,
            dungeon_selected,
        );

        // Box 3 (Stats) - static, never gets the cursor (see this
        // module's doc comment). Sits directly below Equipped Items,
        // filling the rest of the left column's lower half. Converted to
        // PanelBox alongside print_box above - title at dy=-1, same
        // pending fix, same reasoning (see print_box's own doc comment).
        let mut stats_box = PanelBox::new(
            LEFT_X,
            STATS_Y,
            LEFT_WIDTH,
            STATS_HEIGHT,
            UiPanelTheme::Dungeon,
            PIXEL_BOX_TILE_SCALE,
        );
        stats_box.text_color_raw(2, -1, YELLOW, BLACK, " Stats ");
        let health = self
            .ecs
            .entry_ref(player)
            .ok()
            .and_then(|e| e.get_component::<Health>().ok().copied());
        let damage = self
            .ecs
            .entry_ref(player)
            .ok()
            .and_then(|e| e.get_component::<Damage>().ok().copied());
        let defense = self
            .ecs
            .entry_ref(player)
            .ok()
            .and_then(|e| e.get_component::<Defense>().ok().copied());
        let evasion = self
            .ecs
            .entry_ref(player)
            .ok()
            .and_then(|e| e.get_component::<Evasion>().ok().copied());
        let speed = self
            .ecs
            .entry_ref(player)
            .ok()
            .and_then(|e| e.get_component::<Speed>().ok().copied());
        let gold = <&Gold>::query()
            .filter(component::<Player>())
            .iter(&self.ecs)
            .next()
            .map_or(0, |g| g.0);
        let map_level = <&Player>::query()
            .iter(&self.ecs)
            .next()
            .map_or(0, |p| p.map_level);
        // Battle Arena's own level/wave tracking is a completely
        // different concept from Player.map_level (which stays stuck at
        // whatever it was spawned with during an Arena run - see hud.rs's
        // own Gold-vs-Dungeon-Level swap in the top-right corner, same
        // reasoning applied here).
        let arena_run = self
            .resources
            .get::<Option<ArenaRun>>()
            .and_then(|run| *run);

        let mut stats_dy = 0;
        let mut print_stat = |label: &str, value: String| {
            stats_box.text_color(0, stats_dy, WHITE, BLACK, format!("{}: {}", label, value));
            stats_dy += 1;
        };
        if let Some(class) = &class {
            print_stat("Class", class.clone());
        }
        if let Some(health) = health {
            print_stat("Health", format!("{} / {}", health.current, health.max));
        }
        if let Some(damage) = damage {
            print_stat("Damage", damage.0.to_string());
        }
        if let Some(defense) = defense {
            print_stat("Defense", defense.0.to_string());
        }
        if let Some(evasion) = evasion {
            print_stat("Evasion", evasion.0.to_string());
        }
        if let Some(speed) = speed {
            print_stat("Speed", speed.0.to_string());
        }
        print_stat("Gold", gold.to_string());
        match arena_run {
            Some(run) => {
                print_stat("Arena Level", run.level.to_string());
                let wave_display = if run.boss_active {
                    "Boss".to_string()
                } else if run.wave == 0 {
                    "Shop".to_string()
                } else {
                    run.wave.to_string()
                };
                print_stat("Wave", wave_display);
            }
            None => print_stat("Dungeon Level", (map_level + 1).to_string()),
        }
        stats_box.submit();

        // Box 1.1 - the shared description panel, full width, for
        // whichever slot the cursor currently sits on (across any of
        // the 4 navigable boxes). Title at dy=-1 - same pending fix as
        // print_box/Stats above.
        let mut desc_box = PanelBox::new(
            DESC_X,
            DESC_Y,
            DESC_WIDTH,
            DESC_HEIGHT,
            UiPanelTheme::Dungeon,
            PIXEL_BOX_TILE_SCALE,
        );
        desc_box.text_color_raw(2, -1, YELLOW, BLACK, " Description ");
        match selected_slot {
            Some(slot) => {
                let description = description_for_item_name(&slot.name)
                    .unwrap_or_else(|| "No description.".to_string());
                for (i, line) in wrap_text(&description, (DESC_WIDTH - 4) as usize)
                    .iter()
                    .enumerate()
                {
                    desc_box.text_color(0, i as i32, WHITE, BLACK, line.clone());
                }
            }
            // Nothing under the cursor right now (an empty list, or the
            // cursor landed on a box with no entries) - the panel used to
            // just sit blank in this case, which read as broken/unfinished
            // now that it's a real filled stone panel rather than an
            // unobtrusive hollow ASCII box. Same GRAY "Nothing here."
            // convention the empty lists themselves already use.
            None => {
                desc_box.text_color(
                    0,
                    0,
                    GRAY,
                    BLACK,
                    "Select an item or action to see its description.",
                );
            }
        }
        desc_box.submit();

        // Footer stays on plain HUD_CONSOLE (via batch) - it sits in the
        // open area below every box, never on top of a fill, so it was
        // never affected by the cell-replacement bug the rest of this
        // screen's text had.
        batch.print_color_centered(
            FOOTER_Y,
            "Arrows to navigate, Enter to use, ESC to close",
            ColorPair::new(GRAY, BLACK),
        );
        batch.submit(0).expect("Batch error");

        // Enter only actions the two USABLE boxes (Items, Dungeon
        // Actions) - Equipped Items and Battle Actions stay browse-only,
        // same restriction they already had before this redesign.
        if ctx.key == Some(VirtualKeyCode::Return) {
            let to_activate = items_selected
                .and_then(|i| items.get(i))
                .or_else(|| dungeon_selected.and_then(|i| dungeon_actions.get(i)))
                .and_then(|slot| slot.owned);
            if let Some((_, entity)) = to_activate {
                let mut commands = CommandBuffer::new(&self.ecs);
                commands.push((
                    (),
                    ActivateItem {
                        used_by: player,
                        item: entity,
                    },
                ));
                commands.flush(&mut self.ecs);
                // Using an item/ability genuinely costs a turn - straight
                // to PlayerTurn (not AwaitingInput), matching exactly how
                // the old direct number-key press used to advance the
                // dungeon turn. use_items (run during PlayerTurn) applies
                // the actual effect and consumes the item.
                self.resources.insert(TurnState::PlayerTurn);
            }
        } else if ctx.key == Some(VirtualKeyCode::Escape) {
            // Just closing the menu - no item used, no turn spent.
            self.resources.insert(TurnState::AwaitingInput);
        }
    }
}
