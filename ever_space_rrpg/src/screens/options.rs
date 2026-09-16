use crate::prelude::*;
use crate::State;

/// Left/right column X positions and widths, and the gap between a
/// column's own stacked boxes - reuses `screens/item_menu.rs`'s exact
/// numbers (same HUD_CONSOLE grid, same "2 stacked columns" shape), not
/// independently re-tuned.
const LEFT_X: i32 = 3;
const LEFT_WIDTH: i32 = 50;
const RIGHT_X: i32 = 56;
const RIGHT_WIDTH: i32 = 48;
const BOX_GAP: i32 = 3;

/// How many rows each category's own list has - Audio/Video are fixed
/// placeholder counts (see this module's own doc comment above
/// `options_tick`); Hotkeys is `Action::ALL`'s own real length, not a
/// hardcoded guess that could drift out of sync with it.
const AUDIO_ROWS: usize = 3; // Volume, Music, Sound
const VIDEO_ROWS: usize = 2; // Fullscreen, Window
const HOTKEYS_ROWS: usize = Action::ALL.len();
const GAMEPLAY_ROWS: usize = 3; // Battle Speed, ATB Mode, Menu Memory

/// Box heights: `rows + 4`. Went through `+ 2` (copied from the
/// Actions-box/Battle-Log's own RAW-content formula by mistake - wrong
/// for these boxes, which use the standard inset) and `+ 3` (the real
/// SNUG height for standard-inset content - zero blank rows below the
/// last line, matching every other PanelBox conversion this session)
/// before landing here. `+ 3` looked fine for Video/Gameplay, which
/// both have unused row capacity (Video only fills 2 of the 3 rows this
/// height was sized for) - the leftover space read as padding by
/// accident. Audio and Hotkeys, which fill EVERY row this height was
/// sized for, had zero real margin at `+ 3` - direct feedback 2026-09-15
/// ("the audio doesn't have a bit of padding on the bottom"). `+ 4`
/// gives every box, fully populated or not, one genuine blank row
/// before the border - real padding, not leftover capacity that only
/// looked like it.
///
/// Audio and Video sit in the same top row, so BOTH use the taller of
/// the two (Audio's 3) so their bottom edges line up; same reasoning
/// for Hotkeys/Gameplay sharing `HOTKEYS_ROWS` (4, currently the taller
/// of the two) in the bottom row.
const TOP_HEIGHT: i32 = AUDIO_ROWS as i32 + 4;
const BOTTOM_HEIGHT: i32 = HOTKEYS_ROWS as i32 + 4;

const TOP_Y: i32 = 14;
const BOTTOM_Y: i32 = TOP_Y + TOP_HEIGHT + BOX_GAP;
const FOOTER_Y: i32 = BOTTOM_Y + BOTTOM_HEIGHT + 2;

impl State {
    /// The Options screen, reached from Paused (press O) or the title
    /// screen (press O), returning to whichever one it was opened from
    /// on Escape - see options_return_to. Reuses pause_systems to
    /// redraw the map underneath, same as paused_tick - nothing should
    /// move or animate while this menu is open either.
    ///
    /// Redesigned 2026-09-15 into 4 category boxes (Audio, Video,
    /// Hotkeys, Gameplay) - same 2-stacked-column shape `screens/
    /// item_menu.rs` already uses (Left column: Audio above Hotkeys.
    /// Right column: Video above Gameplay), navigated the identical
    /// way: Left/Right switches column (remembering each column's own
    /// row), Up/Down moves within the current column's combined list.
    /// `UiPanelTheme::Gears` (a brass/gunmetal steampunk-and-science
    /// frame, generated the same day) borders all 4 boxes.
    ///
    /// Audio (Volume, Music, Sound) and Video (Fullscreen, Window) are
    /// real, cursor-navigable rows with nothing functional behind them
    /// yet - no audio engine exists (item 6 in docs/ideas.md), and
    /// Fullscreen is PAUSED here as of 2026-09-15 despite `settings.rs`'s
    /// `FullscreenSetting`/`BTermBuilder::with_fullscreen` both being
    /// real and working - going fullscreen exposed a separate, real,
    /// pre-existing bug (bracket-terminal quantizes the visible content
    /// area to the largest registered font's tile size ACROSS THE WHOLE
    /// APP on every resize, and `battle_backgrounds.png` is registered
    /// at a full 1280x800 "tile size" so it can be drawn as one big
    /// glyph - that single registration inflates the quantization unit
    /// globally, producing much bigger black bars than any 16:9-vs-16:10
    /// mismatch alone would, confirmed by measuring the actual art for a
    /// baked-in margin first and finding none). Fixing that needs
    /// re-slicing the battle/Victory/Defeat background art into a normal
    /// small-tile grid - its own separate piece of work, not something
    /// to fold into wiring up a toggle. `FullscreenSetting` itself is
    /// kept as groundwork for whenever that's done, just not wired into
    /// `main()`'s builder chain or this screen's UI right now. Both rows
    /// are drawn permanently GRAY (never YELLOW, even when the cursor is
    /// on them) so they read as "here, but not yet doing anything"
    /// rather than broken, and Enter is a real no-op on either - see the
    /// `None` match arms below.
    ///
    /// Two sub-modes, tracked by `self.options_awaiting`:
    /// - None (browsing): the 4 boxes as described above. Enter on a
    ///   Hotkeys row enters capture mode (see below); on a Gameplay row
    ///   cycles/toggles that setting; on an Audio/Video row does
    ///   (Keymap::reset_to_defaults) without entering capture mode.
    /// - Some(action) (capturing): shows a prompt; the next recognized
    ///   key (see keymap::is_rebindable_key) rebinds `action` to it,
    ///   saves to disk, and returns to browsing. Escape cancels back to
    ///   browsing without changing anything. Rebinding onto a key
    ///   another action already holds swaps the two rather than
    ///   erroring or crashing - see Keymap::rebind.
    pub fn options_tick(&mut self, ctx: &mut BTerm) {
        self.pause_systems
            .execute(&mut self.ecs, &mut self.resources);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(2, YELLOW, BLACK, "-- Options --");

        ctx.set_active_console(HUD_CONSOLE);

        match self.options_awaiting {
            None => {
                let left_len = AUDIO_ROWS + HOTKEYS_ROWS;
                let right_len = VIDEO_ROWS + GAMEPLAY_ROWS;
                let current_len = if self.options_cursor.col == 0 {
                    left_len
                } else {
                    right_len
                };
                match ctx.key {
                    Some(VirtualKeyCode::Up) => self.options_cursor.move_vertical(-1, current_len),
                    Some(VirtualKeyCode::Down) => self.options_cursor.move_vertical(1, current_len),
                    Some(VirtualKeyCode::Left) => self.options_cursor.move_horizontal(0, left_len),
                    Some(VirtualKeyCode::Right) => self.options_cursor.move_horizontal(1, right_len),
                    _ => {}
                }

                // Which local row within EACH of the 4 boxes (if any)
                // the cursor currently sits on - None for every box
                // except whichever one actually contains it right now.
                // Same split item_menu.rs's own print_box call sites
                // already use for its 4 navigable boxes.
                let (audio_selected, hotkeys_selected) = if self.options_cursor.col == 0 {
                    let row = self.options_cursor.row;
                    if row < AUDIO_ROWS {
                        (Some(row), None)
                    } else {
                        (None, Some(row - AUDIO_ROWS))
                    }
                } else {
                    (None, None)
                };
                let (video_selected, gameplay_selected) = if self.options_cursor.col == 1 {
                    let row = self.options_cursor.row;
                    if row < VIDEO_ROWS {
                        (Some(row), None)
                    } else {
                        (None, Some(row - VIDEO_ROWS))
                    }
                } else {
                    (None, None)
                };

                // --- Audio (top-left) - placeholder rows, always GRAY.
                let mut audio_box = PanelBox::new(
                    LEFT_X,
                    TOP_Y,
                    LEFT_WIDTH,
                    TOP_HEIGHT,
                    UiPanelTheme::Gears,
                    PIXEL_BOX_TILE_SCALE,
                );
                audio_box.text_color_raw(2, -1, YELLOW, BLACK, " Audio ");
                for (i, label) in ["Volume", "Music", "Sound"].iter().enumerate() {
                    let prefix = if audio_selected == Some(i) { "> " } else { "" };
                    audio_box.text_color(0, i as i32, GRAY, BLACK, format!("{}{}: --", prefix, label));
                }
                audio_box.submit();

                // --- Video (top-right) - same placeholder treatment.
                let mut video_box = PanelBox::new(
                    RIGHT_X,
                    TOP_Y,
                    RIGHT_WIDTH,
                    TOP_HEIGHT,
                    UiPanelTheme::Gears,
                    PIXEL_BOX_TILE_SCALE,
                );
                video_box.text_color_raw(2, -1, YELLOW, BLACK, " Video ");
                // Fullscreen paused here 2026-09-15 (see this module's own
                // doc comment above) - reverted to the same placeholder
                // treatment as Window, even though FullscreenSetting/
                // with_fullscreen both work, until the real background-art
                // quantization bug is fixed first.
                for (i, label) in ["Fullscreen", "Window"].iter().enumerate() {
                    let prefix = if video_selected == Some(i) { "> " } else { "" };
                    video_box.text_color(0, i as i32, GRAY, BLACK, format!("{}{}: --", prefix, label));
                }
                video_box.submit();

                // --- Hotkeys (bottom-left) - the real rebind list.
                let mut hotkeys_box = PanelBox::new(
                    LEFT_X,
                    BOTTOM_Y,
                    LEFT_WIDTH,
                    BOTTOM_HEIGHT,
                    UiPanelTheme::Gears,
                    PIXEL_BOX_TILE_SCALE,
                );
                hotkeys_box.text_color_raw(2, -1, YELLOW, BLACK, " Hotkeys ");
                {
                    let keymap = self
                        .resources
                        .get::<Keymap>()
                        .expect("Keymap resource missing");
                    for (i, action) in Action::ALL.iter().enumerate() {
                        let key_label = format!("{:?}", keymap.key_for(*action));
                        let selected = hotkeys_selected == Some(i);
                        let color = if selected { YELLOW } else { WHITE };
                        let prefix = if selected { "> " } else { "" };
                        hotkeys_box.text_color(
                            0,
                            i as i32,
                            color,
                            BLACK,
                            format!("{}{}: {}", prefix, action.label(), key_label),
                        );
                    }
                } // keymap's borrow of self.resources ends here, before
                  // the possible self.resources.insert(...) below.
                hotkeys_box.submit();

                // --- Gameplay (bottom-right) - Battle Speed/ATB
                // Mode/Menu Memory, same rows the old flat list had.
                let mut gameplay_box = PanelBox::new(
                    RIGHT_X,
                    BOTTOM_Y,
                    RIGHT_WIDTH,
                    BOTTOM_HEIGHT,
                    UiPanelTheme::Gears,
                    PIXEL_BOX_TILE_SCALE,
                );
                gameplay_box.text_color_raw(2, -1, YELLOW, BLACK, " Gameplay ");
                {
                    let battle_speed = self
                        .resources
                        .get::<BattleSpeed>()
                        .expect("BattleSpeed resource missing");
                    let selected = gameplay_selected == Some(0);
                    let prefix = if selected { "> " } else { "" };
                    gameplay_box.text_color(
                        0,
                        0,
                        if selected { YELLOW } else { WHITE },
                        BLACK,
                        format!("{}Battle Speed: {}", prefix, battle_speed.label()),
                    );
                }
                {
                    let atb_mode = self
                        .resources
                        .get::<AtbMode>()
                        .expect("AtbMode resource missing");
                    let selected = gameplay_selected == Some(1);
                    let prefix = if selected { "> " } else { "" };
                    gameplay_box.text_color(
                        0,
                        1,
                        if selected { YELLOW } else { WHITE },
                        BLACK,
                        format!("{}ATB Mode: {}", prefix, atb_mode.label()),
                    );
                }
                {
                    let menu_memory = self
                        .resources
                        .get::<MenuMemory>()
                        .expect("MenuMemory resource missing");
                    let selected = gameplay_selected == Some(2);
                    let prefix = if selected { "> " } else { "" };
                    gameplay_box.text_color(
                        0,
                        2,
                        if selected { YELLOW } else { WHITE },
                        BLACK,
                        format!("{}Remember Last Battle Action: {}", prefix, menu_memory.label()),
                    );
                }
                gameplay_box.submit();

                ctx.print_color_centered(FOOTER_Y, GRAY, BLACK, "Press R to reset all keys to defaults");
                ctx.print_color_centered(
                    FOOTER_Y + 1,
                    GRAY,
                    BLACK,
                    "Arrows to navigate, Enter to select, ESC to go back",
                );

                if ctx.key == Some(VirtualKeyCode::Return) {
                    if let Some(i) = hotkeys_selected {
                        if let Some(action) = Action::ALL.get(i) {
                            self.options_awaiting = Some(*action);
                        }
                    } else if let Some(i) = gameplay_selected {
                        match i {
                            0 => {
                                let mut speed = self
                                    .resources
                                    .get_mut::<BattleSpeed>()
                                    .expect("BattleSpeed resource missing");
                                *speed = speed.next();
                                let saved = *speed;
                                drop(speed);
                                saved.save();
                            }
                            1 => {
                                let mut mode = self
                                    .resources
                                    .get_mut::<AtbMode>()
                                    .expect("AtbMode resource missing");
                                *mode = mode.next();
                                let saved = *mode;
                                drop(mode);
                                saved.save();
                            }
                            2 => {
                                let mut memory = self
                                    .resources
                                    .get_mut::<MenuMemory>()
                                    .expect("MenuMemory resource missing");
                                *memory = memory.next();
                                let saved = *memory;
                                drop(memory);
                                saved.save();
                            }
                            _ => {}
                        }
                    }
                    // audio_selected / video_selected: no rows there do
                    // anything yet - a deliberate no-op, not a missing
                    // case. (Fullscreen paused here 2026-09-15 - see this
                    // module's own doc comment above.)
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
