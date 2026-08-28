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
        name: "Archer",
        icon_glyph: 'B',
        description: "(Placeholder - Attack/Defend/Flee only, abilities coming soon.)",
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

        ctx.set_active_console(2);
        ctx.print_color_centered(
            60,
            WHITE,
            BLACK,
            "A roguelike adventure into the dungeons below.",
        );
        ctx.print_color_centered(90, GREEN, BLACK, "Press any key to begin");
        ctx.print_color_centered(92, DARK_GRAY, BLACK, "(Esc to quit)");

        if let Some(key) = ctx.key {
            if key == VirtualKeyCode::Escape {
                ctx.quitting = true;
            } else {
                self.resources.insert(TurnState::ClassSelect);
            }
        }
    }

    /// Lists every playable class with a big name/key, a short description,
    /// and a big letter-glyph "icon" (a placeholder for real sprite art -
    /// reuses draw_portrait, the same helper the battle screen uses to
    /// blow up a glyph) next to it, and starts a run with whichever one
    /// the player picks. New classes go here as one more CLASS_ROSTER
    /// entry - this function needs no other changes.
    pub fn class_select(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.print_color_centered(0, YELLOW, BLACK, "Choose Your Class");

        let mut icons = DrawBatch::new();
        icons.target(3);

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
            ctx.print_color(
                9,
                headline_row,
                GREEN,
                BLACK,
                &format!("{}) {}", entry.key_label, entry.name.to_uppercase()),
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

            draw_portrait(
                &mut icons,
                0,
                i,
                Render {
                    color: ColorPair::new(WHITE, BLACK),
                    glyph: to_cp437(entry.icon_glyph),
                },
            );
        }
        icons.submit(0).expect("Batch error");

        if let Some(key) = ctx.key {
            if let Some(entry) = CLASS_ROSTER.iter().find(|c| c.key == key) {
                self.start_game(entry.name);
            } else if key == VirtualKeyCode::D {
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
                self.start_game("Debug");
            }
        }
    }
}
