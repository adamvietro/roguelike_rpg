use crate::prelude::*;
use crate::State;

/// One playable class's class-select entry: which key picks it, what its
/// button reads, the exact string passed to spawn_player/Class (must match
/// any `class:` tags in template.ron for techniques to gate correctly),
/// its placeholder letter-glyph icon, and its description. Adding a class
/// is one more entry here - class_select() needs no other changes.
struct ClassRosterEntry {
    key: VirtualKeyCode,
    key_label: &'static str,
    name: &'static str,
    icon_glyph: char,
    description: &'static str,
}

const CLASS_ROSTER: [ClassRosterEntry; 5] = [
    ClassRosterEntry {
        key: VirtualKeyCode::Key1,
        key_label: "1",
        name: "Barbarian",
        icon_glyph: '@',
        description: "A hardy melee fighter with devastating battle techniques: \
                       Deathblow, Quick Attack, Counter Attack, Rend.",
    },
    ClassRosterEntry {
        key: VirtualKeyCode::Key2,
        key_label: "2",
        name: "Rogue",
        icon_glyph: 'r',
        description: "A fast, evasive fighter (10% base Evasion). Battle \
                       techniques: Garrote, Dodge. Also carries the \
                       out-of-combat Stealth ability for ambush attacks.",
    },
    ClassRosterEntry {
        key: VirtualKeyCode::Key3,
        key_label: "3",
        name: "Amazon",
        icon_glyph: 'a',
        description: "A skirmisher fighting at range with spears (5% base \
                       Evasion). Battle techniques: Poison Spear, Battle \
                       Cry. Also carries the out-of-combat Throw Spear \
                       (damages the nearest visible enemy with no fight) \
                       and Trap (a placed hazard that damages the first \
                       enemy to step on it).",
    },
    ClassRosterEntry {
        key: VirtualKeyCode::Key4,
        key_label: "4",
        name: "Hunter",
        icon_glyph: 'B',
        description: "A ranged fighter wielding bows. Battle techniques: \
                       Poison Shot, Stun, Feint. Also carries the \
                       out-of-combat Shoot (damages the nearest visible \
                       enemy with no fight) and Freeze Trap (a placed \
                       hazard that freezes the first enemy to step on \
                       it).",
    },
    ClassRosterEntry {
        key: VirtualKeyCode::Key5,
        key_label: "5",
        name: "Mage",
        icon_glyph: 'm',
        description: "A fragile spellcaster wielding staffs (Defense -1). \
                       Battle techniques: Fireball, Burn. Also carries the \
                       out-of-combat Invisible Cloak and Ice Armor.",
    },
];

/// How long between each step of the decorative background enemies'
/// ambient wandering, in milliseconds - tuned to look like a deliberate,
/// unhurried stroll rather than the 30-steps-per-second scramble running
/// movement every rendered frame produced. Easy to retune here with no
/// other code change.
const BACKGROUND_MOVE_INTERVAL_MS: f32 = 400.0;

impl State {
    /// Builds a random, fully-revealed decorative dungeon (map + monsters,
    /// no real player) to show behind the title and class-select screens.
    /// Called once at startup and again every time the player returns to
    /// the title screen, so it's freshly randomized each time - but NOT
    /// regenerated between TitleScreen and ClassSelect, so both screens
    /// show the exact same map (neither of those two screens' tick
    /// methods call this - only State::new/return_to_title do). Called
    /// from main.rs (State::new, return_to_title), so this needs to be
    /// `pub` - a private fn here would only be visible to this module and
    /// its descendants, not to the crate root that's calling it.
    /// start_game wipes it outright when a real run begins.
    pub fn spawn_title_background(&mut self) {
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        map_builder
            .map
            .revealed_tiles
            .iter_mut()
            .for_each(|revealed| *revealed = true);

        spawn_level(&mut self.ecs, &mut rng, 0, &map_builder.monster_spawns);
        spawn_prefab_enemies(&mut self.ecs, &mut rng, 0, &map_builder.prefab_enemy_spawns);

        // Ambient wandering for the background enemies (see
        // build_title_background_scheduler, which now runs
        // random_move_system/movement_system). Real dungeon enemies only
        // get ChasingPlayer, not MovingRandomly (see
        // spawner::template::Templates::spawn_entities), so this world's
        // enemies wouldn't move at all otherwise - deliberately NOT using
        // chasing_system here instead, since that paths every enemy
        // toward the anchor entity below and could still trigger a battle
        // against it if reached.
        let mut commands = legion::systems::CommandBuffer::new(&mut self.ecs);
        <(Entity, &Enemy)>::query()
            .iter(&self.ecs)
            .for_each(|(entity, _)| {
                commands.add_component(*entity, MovingRandomly);
                // See DecorativeOnly's own doc comment - prevents these
                // wandering enemies from ever rendering on GLIDE_CONSOLE,
                // which would paint over the class-select icons/headlines.
                commands.add_component(*entity, DecorativeOnly);
            });
        commands.flush(&mut self.ecs);

        // An invisible anchor entity - Player + FieldOfView, deliberately
        // no Render component - so map_render/entity_render (which both
        // query for a Player's FieldOfView to know what's "visible") have
        // something to find without an actual hero glyph appearing over
        // the background. Its visible_tiles is pre-filled with every tile
        // on the map, so the whole dungeon and every monster in it renders
        // at full brightness - "everything explored", per your ask -
        // rather than the dimmer remembered-but-not-currently-seen look
        // real exploration uses.
        let mut fov = FieldOfView::new(0);
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                fov.visible_tiles.insert(Point::new(x, y));
            }
        }
        fov.is_dirty = false;
        let anchor = self
            .ecs
            .push((Player { map_level: 0 }, map_builder.player_start, fov));

        // Now that enemies actually move (see above), a wandering one
        // could in principle step onto this anchor's tile and trigger
        // random_move's normal "player" battle-start path - which would
        // be a real bug here, since this fake Player has none of the
        // Health/Damage/Speed/Defense/Evasion/Class components battle
        // code assumes a real player has. Marking it permanently
        // Invisible makes random_move treat it exactly like it already
        // treats a real Invisible player: movement onto its tile is
        // blocked like a wall, but no battle ever starts.
        let mut commands = legion::systems::CommandBuffer::new(&mut self.ecs);
        commands.add_component(
            anchor,
            Invisible {
                moves_remaining: i32::MAX,
            },
        );
        commands.flush(&mut self.ecs);

        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(map_builder.theme);
    }

    /// Redraws the decorative title/class-select background every frame,
    /// but only advances the wandering enemies' movement once every
    /// BACKGROUND_MOVE_INTERVAL_MS - see background_move_timer_ms. Shared
    /// by title_screen and class_select, which both show this same
    /// background. Only called from within this module, so this can stay
    /// private.
    fn tick_background(&mut self, ctx: &BTerm) {
        self.background_move_timer_ms += ctx.frame_time_ms;
        if self.background_move_timer_ms >= BACKGROUND_MOVE_INTERVAL_MS {
            self.background_move_timer_ms = 0.0;
            self.background_movement_systems
                .execute(&mut self.ecs, &mut self.resources);
        }
        self.background_systems
            .execute(&mut self.ecs, &mut self.resources);
    }

    pub fn title_screen(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(6, YELLOW, BLACK, "EVER SPACE RRPG");

        // HUD_CONSOLE (~12px cells) rather than the old console 2 (8px) -
        // matches the font size class_select already uses for its
        // descriptions, per the user's ask to bump the font on every
        // pre-game screen. Row numbers below are fresh picks for
        // HUD_CONSOLE's 67-row grid, not a mechanical conversion of the
        // old console 2 values (which assumed a 100-row grid) - see this
        // same note in adventure_select/options_tick/stats_view_tick.
        ctx.set_active_console(HUD_CONSOLE);
        ctx.print_color_centered(
            38,
            WHITE,
            BLACK,
            "A roguelike adventure into the dungeons below.",
        );
        ctx.print_color_centered(56, GREEN, BLACK, "Press any key to begin");
        ctx.print_color_centered(
            59,
            DARK_GRAY,
            BLACK,
            "(O for Options, H for History, Esc to quit)",
        );

        if let Some(key) = ctx.key {
            if key == VirtualKeyCode::Escape {
                ctx.quitting = true;
            } else if key == VirtualKeyCode::O {
                self.options_return_to = TurnState::TitleScreen;
                self.options_cursor = 0;
                self.resources.insert(TurnState::Options);
            } else if key == VirtualKeyCode::H {
                self.stats_view_mode = StatsViewMode::Overview;
                self.stats_view_cursor = 0;
                self.resources.insert(TurnState::StatsView);
            } else {
                self.adventure_select_cursor = 0;
                self.resources.insert(TurnState::AdventureSelect);
            }
        }
    }

    /// Dungeon Crawl vs. Battle Arena - the new step between the title
    /// screen and class select. Sets `self.adventure_mode`, which
    /// class_select reads to decide whether to call start_game or
    /// start_arena once a class is picked. Shares the same decorative
    /// background as title_screen/class_select.
    pub fn adventure_select(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(6, YELLOW, BLACK, "Choose Your Adventure");

        ctx.set_active_console(HUD_CONSOLE);
        self.adventure_select_cursor = menu_nav(ctx.key, self.adventure_select_cursor, 2);
        print_menu_row_centered(
            ctx,
            HUD_COLS,
            36,
            GREEN,
            "1) Dungeon Crawl",
            self.adventure_select_cursor == 0,
        );
        ctx.print_color_centered(
            39,
            WHITE,
            BLACK,
            "Explore a randomized dungeon, find the Amulet of Yala.",
        );
        print_menu_row_centered(
            ctx,
            HUD_COLS,
            48,
            GREEN,
            "2) Battle Arena",
            self.adventure_select_cursor == 1,
        );
        ctx.print_color_centered(
            51,
            WHITE,
            BLACK,
            "Clear waves of enemies and bosses across 3 levels, shopping between each.",
        );
        ctx.print_color_centered(62, DARK_GRAY, BLACK, "(Enter to select, Esc to go back)");

        let chosen = match ctx.key {
            Some(VirtualKeyCode::Key1) => Some(0),
            Some(VirtualKeyCode::Key2) => Some(1),
            Some(VirtualKeyCode::Return) => Some(self.adventure_select_cursor),
            _ => None,
        };
        // See pending_enter_release's own doc comment on State - armed
        // here (only when Enter, not a number key, is what actually
        // confirmed) so the same held key can't also immediately confirm
        // a class the instant Class Select appears.
        let confirmed_via_enter = ctx.key == Some(VirtualKeyCode::Return);
        match chosen {
            Some(0) => {
                self.adventure_mode = AdventureMode::DungeonCrawl;
                self.class_select_cursor = 0;
                if confirmed_via_enter {
                    self.pending_enter_release = true;
                }
                self.resources.insert(TurnState::ClassSelect);
            }
            Some(1) => {
                self.adventure_mode = AdventureMode::BattleArena;
                self.class_select_cursor = 0;
                if confirmed_via_enter {
                    self.pending_enter_release = true;
                }
                self.resources.insert(TurnState::ClassSelect);
            }
            _ => {
                if ctx.key == Some(VirtualKeyCode::Escape) {
                    self.resources.insert(TurnState::TitleScreen);
                }
            }
        }
    }

    /// Lists every playable class with a big name/key, a short
    /// description, and a big icon next to it (reuses draw_portrait, the
    /// same helper the battle screen uses to blow up a glyph), and starts
    /// a run with whichever one the player picks. The currently-
    /// highlighted class plays its real idle-loop breathing animation
    /// (resources/character_idle.png, via character_idle_glyph and
    /// CLASS_SELECT_IDLE_CONSOLE) in place of its plain static portrait;
    /// every other row still shows the static glyph icon
    /// (CLASS_ROSTER's own icon_glyph field, unaffected by the
    /// animation). New classes go here as one more CLASS_ROSTER entry -
    /// this function needs no other changes as long as the new class
    /// also gets a components::character_idle_row entry (falls back to
    /// the static icon forever otherwise).
    pub fn class_select(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(0, YELLOW, BLACK, "Choose Your Class");

        let previous_cursor = self.class_select_cursor;
        self.class_select_cursor = menu_nav(ctx.key, self.class_select_cursor, CLASS_ROSTER.len());
        if self.class_select_cursor != previous_cursor {
            // Restart the breathing cycle cleanly on the newly-highlighted
            // class rather than continuing mid-cycle from whichever frame
            // the previous one happened to be on.
            self.class_select_anim_frame = 0;
            self.class_select_anim_elapsed_ms = 0.0;
        }
        self.class_select_anim_elapsed_ms += ctx.frame_time_ms;
        if self.class_select_anim_elapsed_ms >= IDLE_FRAME_DURATION_MS {
            self.class_select_anim_elapsed_ms -= IDLE_FRAME_DURATION_MS;
            self.class_select_anim_frame += 1;
        }

        let mut icons = DrawBatch::new();
        icons.target(BATTLE_PORTRAIT_CONSOLE);
        let mut animated_icon = DrawBatch::new();
        animated_icon.target(CLASS_SELECT_IDLE_CONSOLE);
        let mut still_icon = DrawBatch::new();
        still_icon.target(CHARACTER_PORTRAIT_BIG_CONSOLE);

        for (i, entry) in CLASS_ROSTER.iter().enumerate() {
            let i = i as i32;
            // BIG_TEXT_CONSOLE is 25 rows tall; one ~5-row band per class,
            // matching console 3's 5 total rows (one icon row per class).
            // The headline sits near the TOP of its band (row 1 of 5, not
            // row 3) - it and the icon are side-by-side, not stacked (the
            // icon starts at column 0, headline/description start past
            // column 9), so there's no need to push the headline down to
            // clear the icon vertically. That leaves most of the band's
            // height free for the description below it - previously the
            // headline sat most of the way down the band, leaving so
            // little room the description overlapped it, and the last
            // class's description ran off the bottom of the screen
            // entirely with nowhere left to go.
            let headline_row = i * 5 + 1;
            print_menu_row_left(
                ctx,
                9,
                headline_row,
                GREEN,
                &format!("{}) {}", entry.key_label, entry.name.to_uppercase()),
                self.class_select_cursor == i as usize,
            );

            // Descriptions render on HUD_CONSOLE (~12px
            // cells - bigger than console 2's 8px fine text, smaller than
            // the headline's 32px) and wrap across multiple lines instead
            // of running off the right edge - Mage's description in
            // particular is long enough to overflow a single line.
            //
            // HUD_CONSOLE (107x67) and BIG_TEXT_CONSOLE (40x25, where
            // headline_row lives) cover the same physical 1280x800 window
            // but at different row counts, so converting the headline's
            // pixel bottom edge - not just its row index - into a
            // HUD_CONSOLE row is what actually guarantees no overlap:
            // (headline_row + 1) rows of 32px each, converted to
            // HUD_CONSOLE's ~11.94px rows,
            // rounded UP so the description never starts a fraction of a
            // row too early.
            const DESC_X: i32 = 24;
            const DESC_WRAP_WIDTH: usize = 65;
            let headline_bottom_px = (headline_row + 1) * 32;
            let desc_row_start = (headline_bottom_px * 67 + 799) / 800;
            ctx.set_active_console(HUD_CONSOLE);
            for (line_i, line) in wrap_text(entry.description, DESC_WRAP_WIDTH)
                .iter()
                .enumerate()
            {
                ctx.print_color(DESC_X, desc_row_start + line_i as i32, WHITE, BLACK, line);
            }
            ctx.set_active_console(BIG_TEXT_CONSOLE);

            // The highlighted class plays its real idle-loop animation
            // (CLASS_SELECT_IDLE_CONSOLE, registered after console 3 so
            // it always paints over the static icon there) instead of
            // the plain static portrait every other row still gets - see
            // components::character_idle_glyph.
            if self.class_select_cursor == i as usize {
                if let Some(glyph) = character_idle_glyph(entry.name, self.class_select_anim_frame)
                {
                    draw_portrait(
                        &mut animated_icon,
                        0,
                        i,
                        Render {
                            color: ColorPair::new(WHITE, BLACK),
                            glyph,
                        },
                    );
                    continue;
                }
            }
            // The still portrait (resources/character_portrait.png,
            // PixelLab's own south-facing rotation pose) replaces the
            // old dungeonfont icon_glyph for any class with a row there
            // - see components::character_portrait_glyph. Falls back to
            // the old dungeonfont glyph for anything without one yet.
            match character_portrait_glyph(entry.name) {
                Some(glyph) => draw_portrait(
                    &mut still_icon,
                    0,
                    i,
                    Render {
                        color: ColorPair::new(WHITE, BLACK),
                        glyph,
                    },
                ),
                None => draw_portrait(
                    &mut icons,
                    0,
                    i,
                    Render {
                        color: ColorPair::new(WHITE, BLACK),
                        glyph: to_cp437(entry.icon_glyph),
                    },
                ),
            }
        }
        icons.submit(0).expect("Batch error");
        animated_icon.submit(1).expect("Batch error");
        still_icon.submit(2).expect("Batch error");

        // pending_enter_release guards specifically against the SAME held
        // Enter that just confirmed a choice on Adventure Select also
        // immediately confirming a class here, the frame this screen
        // first appears - see that field's own doc comment on State.
        // Only Enter's own arm needs the guard; a direct class-letter key
        // (1-5) isn't affected by this particular hand-off, and falls
        // through to CLASS_ROSTER's own key match unconditionally.
        let chosen_entry = match ctx.key {
            Some(VirtualKeyCode::Return) => {
                if self.pending_enter_release {
                    None
                } else {
                    CLASS_ROSTER.get(self.class_select_cursor)
                }
            }
            Some(key) => CLASS_ROSTER.iter().find(|c| c.key == key),
            None => None,
        };
        if let Some(entry) = chosen_entry {
            match self.adventure_mode {
                AdventureMode::DungeonCrawl => self.start_game(entry.name),
                AdventureMode::BattleArena => self.start_arena(entry.name),
            }
        } else if ctx.key == Some(VirtualKeyCode::D) {
            // Hidden dev shortcut - deliberately NOT a CLASS_ROSTER
            // entry, so it never appears in the visible list or
            // description text. See spawner::class_base_stats("Debug")
            // and resources/starting_kits.ron for what this class
            // actually gets. A letter key, not the originally-tried
            // Backslash - punctuation keys are exactly the kind of
            // thing that can misbehave through this project's
            // WSLg/X11 stack (same family of issue as the documented
            // WINIT_UNIX_BACKEND quirk), while letter keys (G, Q) are
            // already proven working elsewhere in this codebase.
            //
            // Mirrors the same adventure_mode branch as chosen_entry
            // above - this predates Battle Arena mode and always called
            // start_game (Dungeon Crawl) unconditionally, so pressing D
            // from Battle Arena's own Class Select silently dropped into
            // a Dungeon Crawl run instead (found by the user trying to
            // reach Debug in the Arena to test Orc Warlord's art).
            match self.adventure_mode {
                AdventureMode::DungeonCrawl => self.start_game("Debug"),
                AdventureMode::BattleArena => self.start_arena("Debug"),
            }
        }
    }
}
