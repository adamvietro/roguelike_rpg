#![warn(clippy::pedantic)]

mod battle;
mod camera;
mod components;
mod map;
mod map_builder;
mod spawner;
mod systems;
mod turn_state;

mod prelude {
    pub use bracket_lib::prelude::*;
    pub use legion::systems::CommandBuffer;
    pub use legion::world::SubWorld;
    pub use legion::*;
    pub const SCREEN_WIDTH: i32 = 80;
    pub const SCREEN_HEIGHT: i32 = 50;
    pub const DISPLAY_WIDTH: i32 = SCREEN_WIDTH / 2;
    pub const DISPLAY_HEIGHT: i32 = SCREEN_HEIGHT / 2;
    // Battle portrait console: same physical 1280x800 window, a much
    // coarser grid, so a single glyph drawn in one cell renders far bigger
    // than the dungeon view's 32px tiles (256x160px per cell here).
    pub const BATTLE_PORTRAIT_COLS: i32 = 5;
    pub const BATTLE_PORTRAIT_ROWS: i32 = 5;
    // Dungeon HUD console (health bar, item lists, tooltips): same
    // physical 1280x800 window as everything else, but ~1.5x bigger cells
    // than the old 8px text (console 2, still used by battle screens),
    // so exploration-view text reads bigger without touching battle UI.
    pub const HUD_COLS: i32 = 107;
    pub const HUD_ROWS: i32 = 67;
    // Console indices, in BTermBuilder registration order (see main()):
    // 0 dungeon tiles+bg, 1 dungeon entities, 2 fine 8px text (battle/
    // game-over/victory/pause), 3 battle portraits, 4 HUD, 5 big title/
    // class-select text below (console 5 - referenced as a plain literal
    // at call sites, matching how every other console index in this file
    // is already written, since set_active_console's exact parameter type
    // isn't confirmed here). Big text console: same physical 1280x800
    // window, DISPLAY_WIDTH x DISPLAY_HEIGHT cols/rows (the dungeon view's
    // own grid) on the small text font instead of the dungeon font - lands
    // at 32x32px cells, 4x console 2's 8px text.
    /// Console 6: a "fancy console" (supports DrawBatch::set_fancy, incl.
    /// rotation) - same DISPLAY_WIDTH x DISPLAY_HEIGHT grid and dungeonfont
    /// as console 0, so cells are the same 32x32px squares, giving a clean
    /// (non-stretched) rotation. Used only by
    /// draw_end_screen_fallen_portrait for the GameOver screen's fallen
    /// hero - see main()'s builder chain and State::tick's console-clear
    /// block.
    pub const END_SCREEN_FALLEN_CONSOLE: usize = 6;
    /// How much to blow up the fallen hero's glyph on the GameOver screen
    /// - set_fancy's `scale` parameter, a multiplier on the glyph's native
    /// 32x32px size (uniform x/y so a 90-degree rotation stays square,
    /// not stretched). 1.0 (native size) reads as tiny against a
    /// 1280x800 window - this is the equivalent of the battle portraits'
    /// "5x scale-up via a coarse grid" trick, just done through
    /// set_fancy's own scale parameter instead, since that trick isn't
    /// available on a fancy console (positions there aren't snapped to a
    /// grid the way draw_portrait's are).
    pub const END_SCREEN_FALLEN_SCALE: f32 = 6.0;
    pub use crate::battle::*;
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::spawner::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}

use prelude::*;

/// Draws `render`'s glyph in a single cell at (col, row) in the given
/// DrawBatch's target console coordinate space. Used to render scaled-up
/// battle portraits on the coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS
/// console (see main()): because that console's cells are much bigger than
/// the dungeon view's 32px tiles, a single glyph drawn there renders as a
/// large, stretched version of the same sprite - no repetition needed.
/// (A tiled block of many small cells, as an earlier version of this
/// function did, does NOT scale the sprite up - it repeats the same small
/// icon as a grid pattern, since tiling only produces one coherent bigger
/// image if the source art is itself split into matching fragments, which
/// our dungeonfont.png icons are not.)
fn draw_portrait(batch: &mut DrawBatch, col: i32, row: i32, render: Render) {
    batch.set(Point::new(col, row), render.color, render.glyph);
}

/// Greedily wraps `text` into lines no longer than `width` characters,
/// breaking only at word boundaries (never mid-word). Used for the
/// class-select descriptions, which vary a lot in length - some are short
/// placeholders, others (Mage's) are long enough to run off the screen
/// printed as a single line - see class_select.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Draws a hollow rectangular border - plain '-'/'|'/'+' characters, built
/// from the same DrawBatch::set + to_cp437 primitives already proven
/// throughout this file (map/portrait/arena rendering), rather than
/// reaching for a higher-level box-drawing API this project hasn't used
/// anywhere else. Used to frame the battle-menu action list - see
/// BattleTurn::PlayerMenu in battle_tick.
fn draw_hollow_box(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: ColorPair,
) {
    let dash = to_cp437('-');
    let pipe = to_cp437('|');
    let corner = to_cp437('+');
    for dx in 0..width {
        batch.set(Point::new(x + dx, y), color, dash);
        batch.set(Point::new(x + dx, y + height - 1), color, dash);
    }
    for dy in 0..height {
        batch.set(Point::new(x, y + dy), color, pipe);
        batch.set(Point::new(x + width - 1, y + dy), color, pipe);
    }
    batch.set(Point::new(x, y), color, corner);
    batch.set(Point::new(x + width - 1, y), color, corner);
    batch.set(Point::new(x, y + height - 1), color, corner);
    batch.set(Point::new(x + width - 1, y + height - 1), color, corner);
}

/// Tints `base`'s foreground color for a brief post-action flash - white
/// for Attacking, red for Hit - or returns it unchanged once the flash has
/// expired or was never set. Background is left untouched: console 3 is a
/// plain no_bg console, so its background is never actually rendered
/// anyway. See Battle::enemy_flash/player_flash.
fn flash_tint(base: ColorPair, flash: Option<(FlashKind, f32)>) -> ColorPair {
    if let Some((kind, remaining)) = flash {
        if remaining > 0.0 {
            let flash_color = match kind {
                FlashKind::Attacking => WHITE,
                FlashKind::Hit => RED,
            };
            return ColorPair::new(flash_color, base.bg);
        }
    }
    base
}

/// Draws a stylized tree/feature silhouette - a stepped triangular canopy
/// over a short trunk - centered at (cx, cy) on whatever console the given
/// DrawBatch targets. Fills both background and foreground (a brighter
/// shade of the same color) so it reads as a solid shape rather than the
/// thin scattered marks a foreground-only glyph gives.
fn draw_tree(
    batch: &mut DrawBatch,
    glyph: FontCharType,
    canopy_color: RGB,
    trunk_color: RGB,
    cx: i32,
    cy: i32,
) {
    let canopy_fg = RGB::from_f32(
        (canopy_color.r * 1.3).min(1.0),
        (canopy_color.g * 1.3).min(1.0),
        (canopy_color.b * 1.3).min(1.0),
    );
    let widths = [1, 3, 5, 7, 5, 3, 1];
    for (i, &w) in widths.iter().enumerate() {
        let row = cy - 3 + i as i32;
        let half = w / 2;
        for dx in -half..=half {
            batch.set(
                Point::new(cx + dx, row),
                ColorPair::new(canopy_fg, canopy_color),
                glyph,
            );
        }
    }

    let trunk_fg = RGB::from_f32(
        (trunk_color.r * 1.3).min(1.0),
        (trunk_color.g * 1.3).min(1.0),
        (trunk_color.b * 1.3).min(1.0),
    );
    for row in (cy + 4)..=(cy + 5) {
        batch.set(
            Point::new(cx, row),
            ColorPair::new(trunk_fg, trunk_color),
            glyph,
        );
    }
}

/// Blends `base` brighter near the center of a w x h grid (a "clearing")
/// and darker toward the edges (deeper shadow), rather than only ever
/// darkening outward from a neutral center.
fn vignette(base: RGB, x: i32, y: i32, w: i32, h: i32) -> RGB {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let dx = (x as f32 - cx) / cx;
    let dy = (y as f32 - cy) / cy;
    let dist = (dx * dx + dy * dy).sqrt().min(1.0);
    let factor = 1.35 - dist * 0.8;
    RGB::from_f32(
        (base.r * factor).min(1.0),
        (base.g * factor).min(1.0),
        (base.b * factor).min(1.0),
    )
}

/// Component-wise multiplies `base` by `tint` (each in 0.0-1.0), clamped -
/// used to recolor the same floor/wall palette a level's theme already
/// defines rather than hardcoding a whole separate palette for the
/// GameOver/Victory backgrounds. E.g. a reddish tint darkens/desaturates
/// toward red for defeat; a warm gold tint brightens toward gold for
/// victory.
fn tint_color(base: RGB, tint: RGB) -> RGB {
    RGB::from_f32(
        (base.r * tint.r).min(1.0),
        (base.g * tint.g).min(1.0),
        (base.b * tint.b).min(1.0),
    )
}

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
        description: "(Placeholder - Attack/Defend/Flee only, abilities coming soon.)",
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

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
    pause_systems: Schedule,
    background_systems: Schedule,
    /// See build_title_background_movement_scheduler - the decorative
    /// background enemies' movement, run separately from
    /// background_systems (which redraws every frame) so it can be
    /// throttled by background_move_timer_ms instead of moving a full
    /// tile 30 times a second.
    background_movement_systems: Schedule,
    /// Accumulates real elapsed time (ms) while the title/class-select
    /// background is on screen; background_movement_systems only
    /// actually runs once this passes BACKGROUND_MOVE_INTERVAL_MS, then
    /// it resets to 0 - see title_screen/class_select.
    background_move_timer_ms: f32,
}

/// How long between each step of the decorative background enemies'
/// ambient wandering, in milliseconds - tuned to look like a deliberate,
/// unhurried stroll rather than the 30-steps-per-second scramble running
/// movement every rendered frame produced. Easy to retune here with no
/// other code change.
const BACKGROUND_MOVE_INTERVAL_MS: f32 = 400.0;

impl State {
    fn new() -> Self {
        let mut resources = Resources::default();
        resources.insert(TurnState::TitleScreen);
        // random_move_system (part of background_movement_systems, see
        // build_title_background_movement_scheduler) reads this resource -
        // it must exist before the very first
        // background_movement_systems.execute() call, which happens from
        // title_screen()'s tick right after this constructor returns.
        // Previously nothing in the background schedules touched
        // Option<Battle> at all, so its absence here was harmless; it no
        // longer is.
        resources.insert(None::<Battle>);
        let mut state = Self {
            ecs: World::default(),
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
            pause_systems: build_pause_scheduler(),
            background_systems: build_title_background_scheduler(),
            background_movement_systems: build_title_background_movement_scheduler(),
            background_move_timer_ms: 0.0,
        };
        state.spawn_title_background();
        state
    }

    /// Builds a fresh game world for a new run, with the player spawned as
    /// `class` - called once when leaving ClassSelect, and again any time
    /// the player returns to the title screen and picks a class to start
    /// over. Replaces the old reset_game_state, which always hardcoded
    /// "Barbarian" at spawn_player instead of taking a chosen class.
    fn start_game(&mut self, class: &str) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        let player = spawn_player(&mut self.ecs, map_builder.player_start, class);
        grant_starting_items(&mut self.ecs, player, class);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut self.ecs, &mut rng, 0, &map_builder.monster_spawns);
        spawn_prefab_enemies(&mut self.ecs, &mut rng, 0, &map_builder.prefab_enemy_spawns);
        spawn_prefab_weapon(
            &mut self.ecs,
            &mut rng,
            0,
            map_builder.prefab_weapon_spawn,
            class,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
    }

    /// Builds a random, fully-revealed decorative dungeon (map + monsters,
    /// no real player) to show behind the title and class-select screens.
    /// Called once at startup and again every time the player returns to
    /// the title screen, so it's freshly randomized each time - but NOT
    /// regenerated between TitleScreen and ClassSelect, so both screens
    /// show the exact same map (neither of those two screens' tick
    /// methods call this - only State::new/return_to_title do).
    /// start_game wipes it outright when a real run begins.
    fn spawn_title_background(&mut self) {
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

    /// Tears down the current run (if any) and returns to the title
    /// screen - called when the player dismisses the GameOver or Victory
    /// screen, instead of immediately starting a new run with whatever
    /// class they last had. ClassSelect (via start_game) is now the only
    /// place a run actually begins.
    fn return_to_title(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        self.spawn_title_background();
        self.resources.insert(TurnState::TitleScreen);
        // Resources::default() above wipes Option<Battle> along with
        // everything else - background_movement_systems (random_move_system)
        // needs it present before the next
        // background_movement_systems.execute() call, same reasoning as
        // State::new().
        self.resources.insert(None::<Battle>);
        self.background_move_timer_ms = 0.0;
    }

    /// Redraws the decorative title/class-select background every frame,
    /// but only advances the wandering enemies' movement once every
    /// BACKGROUND_MOVE_INTERVAL_MS - see background_move_timer_ms. Shared
    /// by title_screen and class_select, which both show this same
    /// background.
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

    fn title_screen(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(5);
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
    fn class_select(&mut self, ctx: &mut BTerm) {
        self.tick_background(ctx);

        ctx.set_active_console(5);
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

            // Descriptions render on console 4 (the HUD console, ~12px
            // cells - bigger than console 2's 8px fine text, smaller than
            // the headline's 32px) and wrap across multiple lines instead
            // of running off the right edge - Mage's description in
            // particular is long enough to overflow a single line.
            //
            // Console 4 (107x67) and console 5 (40x25, where headline_row
            // lives) cover the same physical 1280x800 window but at
            // different row counts, so converting the headline's pixel
            // bottom edge - not just its row index - into a console-4 row
            // is what actually guarantees no overlap: (headline_row + 1)
            // rows of 32px each, converted to console 4's ~11.94px rows,
            // rounded UP so the description never starts a fraction of a
            // row too early.
            const DESC_X: i32 = 24;
            const DESC_WRAP_WIDTH: usize = 65;
            let headline_bottom_px = (headline_row + 1) * 32;
            let desc_row_start = (headline_bottom_px * 67 + 799) / 800;
            ctx.set_active_console(4);
            for (line_i, line) in wrap_text(entry.description, DESC_WRAP_WIDTH)
                .iter()
                .enumerate()
            {
                ctx.print_color(DESC_X, desc_row_start + line_i as i32, WHITE, BLACK, line);
            }
            ctx.set_active_console(5);

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

    /// Redraws the dungeon map beneath a pause overlay - see
    /// build_pause_scheduler for why this only re-runs map_render rather
    /// than the full input schedule (nothing should move, animate, or
    /// otherwise change while paused). Escape resumes; Q quits to the
    /// title screen, tearing down the current run.
    fn paused_tick(&mut self, ctx: &mut BTerm) {
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

    fn advance_level(&mut self) {
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&mut self.ecs)
            .nth(0)
            .unwrap();

        use std::collections::HashSet;
        let mut entities_to_keep = HashSet::new();
        entities_to_keep.insert(player_entity);
        <(Entity, &Carried)>::query()
            .iter(&self.ecs)
            .filter(|(_e, carry)| carry.0 == player_entity)
            .map(|(e, _carry)| *e)
            .for_each(|e| {
                entities_to_keep.insert(e);
            });
        let mut cb = CommandBuffer::new(&mut self.ecs);
        for e in Entity::query().iter(&self.ecs) {
            if !entities_to_keep.contains(e) {
                cb.remove(*e);
            }
        }
        cb.flush(&mut self.ecs);

        <&mut FieldOfView>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|fov| fov.is_dirty = true);

        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        let mut map_level = 0;
        <(&mut Player, &mut Point)>::query()
            .iter_mut(&mut self.ecs)
            .for_each(|(player, pos)| {
                player.map_level += 1;
                map_level = player.map_level;
                pos.x = map_builder.player_start.x;
                pos.y = map_builder.player_start.y;
            });
        if map_level == 2 {
            spawn_amulet_of_yala(&mut self.ecs, map_builder.amulet_start);
        } else {
            let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
            map_builder.map.tiles[exit_idx] = TileType::Exit;
        }
        spawn_level(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            &map_builder.monster_spawns,
        );
        spawn_prefab_enemies(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            &map_builder.prefab_enemy_spawns,
        );
        let player_class = entity_class(&self.ecs, player_entity).unwrap_or_default();
        spawn_prefab_weapon(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            map_builder.prefab_weapon_spawn,
            &player_class,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
    }

    /// Draws the battle arena background (console 0: the current theme's
    /// floor/walls/scenery) and both combatant portraits (console 3).
    /// Callers look up Render live during an active battle, or pass a
    /// value captured before an entity was removed (see
    /// battle_victory_tick, where the enemy no longer exists in the ECS).
    /// Fills console 0 with the current run's dungeon theme (floor/wall
    /// tiles + vignette - same ingredients as draw_battle_arena's
    /// background, minus the battle-specific scenery/border), tinted
    /// toward `tint`. Used by game_over/victory so those screens show a
    /// moody dimmed/glowing version of the actual dungeon instead of a
    /// flat black backdrop - the map/theme resources are still the
    /// current run's, since neither death nor victory wipes them
    /// (only return_to_title does, once the player dismisses the screen).
    fn draw_end_screen_background(&mut self, tint: RGB) {
        let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap();
        let floor_glyph = theme.tile_to_render(TileType::Floor);
        let wall_glyph = theme.tile_to_render(TileType::Wall);
        let floor_base = tint_color(theme.floor_color(), tint);
        let wall_base = tint_color(theme.wall_color(), tint);
        drop(theme);

        let mut arena = DrawBatch::new();
        arena.target(0);
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let is_border =
                    x == 0 || y == 0 || x == DISPLAY_WIDTH - 1 || y == DISPLAY_HEIGHT - 1;
                let (glyph, base) = if is_border {
                    (wall_glyph, wall_base)
                } else {
                    (floor_glyph, floor_base)
                };
                let bg = vignette(base, x, y, DISPLAY_WIDTH, DISPLAY_HEIGHT);
                let fg = RGB::from_f32(
                    (bg.r * 1.4).min(1.0),
                    (bg.g * 1.4).min(1.0),
                    (bg.b * 1.4).min(1.0),
                );
                arena.set(Point::new(x, y), ColorPair::new(fg, bg), glyph);
            }
        }
        arena.submit(0).expect("Batch error");
    }

    /// Draws the player's own glyph, big, on the battle-portrait console
    /// (console 3) - same trick draw_portrait already uses during battle -
    /// recolored solid `tint` rather than the entity's normal sprite
    /// color, so it reads as a silhouette (grey for defeat, gold for
    /// victory) instead of looking like an active battle portrait.
    /// Position is centered horizontally, placed in the console's middle
    /// row, clear of the console-2 text used above it in game_over/victory.
    /// Used by victory (upright); game_over uses
    /// draw_end_screen_fallen_portrait instead, which is rotated.
    fn draw_end_screen_portrait(&mut self, tint: RGB) {
        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .map(|(e, _)| *e)
            .nth(0);
        let render = player.and_then(|p| entity_render_component(&self.ecs, p));
        if let Some(render) = render {
            let mut portrait = DrawBatch::new();
            portrait.target(3);
            portrait.set(
                Point::new(BATTLE_PORTRAIT_COLS / 2, BATTLE_PORTRAIT_ROWS / 2),
                ColorPair::new(tint, BLACK),
                render.glyph,
            );
            portrait.submit(0).expect("Batch error");
        }
    }

    /// Draws the player's own glyph rotated 90 degrees - lying on its side,
    /// for the GameOver screen specifically. draw_portrait/
    /// draw_end_screen_portrait can only place a glyph on a fixed grid
    /// cell, upright - there's no rotation available on a plain
    /// "simple console". Actual rotation needs bracket-terminal's "fancy
    /// console" feature (DrawBatch::set_fancy, on END_SCREEN_FALLEN_CONSOLE
    /// - see main()), which nothing in this project has used before now.
    ///
    /// set_fancy's rotation parameter needs `Into<Radians>`, not a plain
    /// f32 - confirmed by your build's own compiler error, which also
    /// confirmed bracket-geometry's `Degrees` type is the thing that
    /// converts into it.
    ///
    /// THE UNTESTED PART: a flat opaque background quad (from two earlier
    /// attempts, both confirmed by screenshot) can't blend into the
    /// radial vignette behind it no matter what color it's given - a flat
    /// rectangle inside a gradient always shows a seam. The real fix is a
    /// genuinely transparent background instead of a matched one. This
    /// tries that via `RGBA` with alpha 0, betting that ColorPair is
    /// actually built on RGBA under the hood (with plain RGB silently
    /// converting to alpha=1 opaque, which is consistent with every
    /// ColorPair::new(RGB, RGB) call elsewhere in this file compiling
    /// fine) rather than RGB-only - nothing in this codebase has needed
    /// alpha before, so this specific call is a genuine guess, not a
    /// proven pattern. If it doesn't compile, the error will say exactly
    /// what type is actually expected, and I can fix it precisely from
    /// that - or fall back to the "deliberate framed plaque" approach if
    /// transparency turns out not to be available here at all.
    fn draw_end_screen_fallen_portrait(&mut self, icon_tint: RGB) {
        let player = <(Entity, &Player)>::query()
            .iter(&self.ecs)
            .map(|(e, _)| *e)
            .nth(0);
        let render = player.and_then(|p| entity_render_component(&self.ecs, p));
        if let Some(render) = render {
            let cx = DISPLAY_WIDTH / 2;
            let cy = DISPLAY_HEIGHT / 2;

            let mut fallen = DrawBatch::new();
            fallen.target(END_SCREEN_FALLEN_CONSOLE);
            // Center of the console's square-celled DISPLAY_WIDTH x
            // DISPLAY_HEIGHT grid (same 32px cells as the main dungeon
            // view, so a 90-degree turn doesn't stretch/squash the glyph).
            let fg: RGBA = icon_tint.into();
            let bg: RGBA = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
            fallen.set_fancy(
                PointF::new(cx as f32, cy as f32),
                0,
                Degrees::new(90.0),
                PointF::new(END_SCREEN_FALLEN_SCALE, END_SCREEN_FALLEN_SCALE),
                ColorPair::new(fg, bg),
                render.glyph,
            );
            fallen.submit(0).expect("Batch error");
        }
    }

    fn draw_battle_arena(
        &mut self,
        enemy_render: Option<Render>,
        player_render: Option<Render>,
        enemy_flash: Option<(FlashKind, f32)>,
        player_flash: Option<(FlashKind, f32)>,
    ) {
        // --- Arena background: the current dungeon theme's floor/wall
        // tiles, tinted with that theme's palette and framed with a border,
        // plus a soft vignette that brightens toward the center (a
        // "clearing") and darkens toward the edges. Console 0 is otherwise
        // blank outside battle-related states, so this is free real estate.
        //
        // Cell backgrounds (not just the thin foreground glyph) carry the
        // tint, since a small character like '.' or ';' only covers a
        // fraction of a cell's pixels - foreground-only color reads as
        // scattered specks on black rather than an actual colored floor.
        {
            let theme = self.resources.get::<Box<dyn MapTheme>>().unwrap();
            let floor_glyph = theme.tile_to_render(TileType::Floor);
            let wall_glyph = theme.tile_to_render(TileType::Wall);
            let floor_base = theme.floor_color();
            let wall_base = theme.wall_color();
            let scenery = theme.battle_scenery();
            drop(theme);

            let mut arena = DrawBatch::new();
            arena.target(0);
            for y in 0..DISPLAY_HEIGHT {
                for x in 0..DISPLAY_WIDTH {
                    let is_border =
                        x == 0 || y == 0 || x == DISPLAY_WIDTH - 1 || y == DISPLAY_HEIGHT - 1;
                    let (glyph, base) = if is_border {
                        (wall_glyph, wall_base)
                    } else {
                        (floor_glyph, floor_base)
                    };
                    let bg = vignette(base, x, y, DISPLAY_WIDTH, DISPLAY_HEIGHT);
                    let fg = RGB::from_f32(
                        (bg.r * 1.4).min(1.0),
                        (bg.g * 1.4).min(1.0),
                        (bg.b * 1.4).min(1.0),
                    );
                    arena.set(Point::new(x, y), ColorPair::new(fg, bg), glyph);
                }
            }

            match scenery {
                BattleScenery::ScatteredTrees => {
                    // A handful of large tree/foliage silhouettes,
                    // hand-placed clear of the portraits, their labels,
                    // and the message/menu panel. Drawn at full-strength
                    // color (not vignetted) so they read as distinct
                    // features wherever they land on the light/dark
                    // gradient above.
                    let canopy_color = RGB::from_f32(
                        (floor_base.r * 1.5).min(1.0),
                        (floor_base.g * 1.5).min(1.0),
                        (floor_base.b * 1.5).min(1.0),
                    );
                    let trunk_color =
                        RGB::from_f32(wall_base.r * 0.85, wall_base.g * 0.85, wall_base.b * 0.85);
                    for &(tx, ty) in &[(6, 3), (18, 3), (35, 15), (22, 19)] {
                        draw_tree(&mut arena, wall_glyph, canopy_color, trunk_color, tx, ty);
                    }
                }
                BattleScenery::RoomWalls => {
                    // Thick stone walls down the left/right sides, so the
                    // arena reads as an enclosed room rather than open
                    // ground. Full-strength color (not vignetted) - these
                    // are structural, not lighting, so they stay solid
                    // regardless of the floor's center-lit gradient.
                    const SIDE_WALL_THICKNESS: i32 = 4;
                    let fg = RGB::from_f32(
                        (wall_base.r * 1.4).min(1.0),
                        (wall_base.g * 1.4).min(1.0),
                        (wall_base.b * 1.4).min(1.0),
                    );
                    let wall_color_pair = ColorPair::new(fg, wall_base);
                    for y in 0..DISPLAY_HEIGHT {
                        for x in 0..SIDE_WALL_THICKNESS {
                            arena.set(Point::new(x, y), wall_color_pair, wall_glyph);
                            let rx = DISPLAY_WIDTH - 1 - x;
                            arena.set(Point::new(rx, y), wall_color_pair, wall_glyph);
                        }
                    }
                }
            }

            arena.submit(0).expect("Batch error");
        }

        // --- Portraits: each creature's own glyph, drawn once on the
        // coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS console, so it
        // renders far larger than its normal dungeon-map size. Enemy sits
        // top-right, player sits bottom-left.
        let mut portraits = DrawBatch::new();
        portraits.target(3);
        if let Some(render) = enemy_render {
            let tinted = Render {
                color: flash_tint(render.color, enemy_flash),
                glyph: render.glyph,
            };
            draw_portrait(&mut portraits, 3, 1, tinted);
        }
        if let Some(render) = player_render {
            let tinted = Render {
                color: flash_tint(render.color, player_flash),
                glyph: render.glyph,
            };
            draw_portrait(&mut portraits, 1, 3, tinted);
        }
        portraits.submit(0).expect("Batch error");
    }

    fn battle_tick(&mut self, ctx: &mut BTerm) {
        let battle_snapshot = self.resources.get::<Option<Battle>>().unwrap().clone();
        let mut battle = match battle_snapshot {
            Some(b) => b,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        // Tick down any active post-action portrait flash (see
        // Battle::enemy_flash/player_flash and flash_tint).
        if let Some((_, remaining)) = &mut battle.enemy_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.enemy_flash = None;
            }
        }
        if let Some((_, remaining)) = &mut battle.player_flash {
            *remaining -= ctx.frame_time_ms;
            if *remaining <= 0.0 {
                battle.player_flash = None;
            }
        }

        // Tick down any active floating damage number (see
        // Battle::enemy_damage_popup/player_damage_popup) the same way.
        if let Some(popup) = &mut battle.enemy_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.enemy_damage_popup = None;
            }
        }
        if let Some(popup) = &mut battle.player_damage_popup {
            popup.remaining_ms -= ctx.frame_time_ms;
            if popup.remaining_ms <= 0.0 {
                battle.player_damage_popup = None;
            }
        }

        // --- Initiative: decided once at the start of each round, from
        // Speed. Any active damage-over-time effect (Rend, Burn, etc.)
        // ticks first, before initiative is even decided - it's a
        // lingering wound, not an action. If the enemy is faster, they
        // attack immediately here - no menu shown - before the player
        // ever gets a choice this round.
        if battle.awaiting_order_decision {
            let dot_message = tick_dot(&mut self.ecs, &mut battle);

            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
            if enemy_hp_now < 1 {
                let mut rng = RandomNumberGenerator::new();
                let loot = grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player);
                let mut cb = CommandBuffer::new(&mut self.ecs);
                cb.remove(battle.enemy);
                cb.flush(&mut self.ecs);
                self.resources.insert(Some(BattleVictory {
                    player: battle.player,
                    enemy_name: battle.enemy_name.clone(),
                    loot,
                }));
                self.resources.insert(None::<Battle>);
                self.resources.insert(TurnState::BattleVictory);
                return;
            }

            let player_speed = entity_speed(&self.ecs, battle.player);
            let enemy_speed = entity_speed(&self.ecs, battle.enemy);
            battle.first_actor = if battle.sneak_attack || player_speed >= enemy_speed {
                Combatant::Player
            } else {
                Combatant::Enemy
            };
            battle.awaiting_order_decision = false;

            if battle.first_actor == Combatant::Enemy {
                if let Some(dot_message) = dot_message {
                    battle.push_log(dot_message);
                }
                resolve_enemy_attack(&mut self.ecs, &mut battle);
                battle.enter_result(BattleTurn::FirstResult);
            } else if let Some(dot_message) = dot_message {
                battle.push_log(dot_message);
            }
        }

        let (enemy_hp, enemy_max) = entity_health(&self.ecs, battle.enemy);
        let (player_hp, player_max) = entity_health(&self.ecs, battle.player);

        let enemy_render = entity_render_component(&self.ecs, battle.enemy);
        let player_render = entity_render_component(&self.ecs, battle.player);
        self.draw_battle_arena(
            enemy_render,
            player_render,
            battle.enemy_flash,
            battle.player_flash,
        );

        // --- Text: name + HP bar anchored next to each portrait, and a
        // message/menu panel centered in the gap between them.
        ctx.set_active_console(2);

        ctx.print_color(96, 41, YELLOW, BLACK, &battle.enemy_name);
        ctx.print_color(
            96,
            42,
            YELLOW,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(enemy_hp, enemy_max, 16),
                enemy_hp.max(0),
                enemy_max
            ),
        );

        ctx.print_color(32, 58, WHITE, BLACK, "You");
        ctx.print_color(
            32,
            59,
            WHITE,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(player_hp, player_max, 16),
                player_hp.max(0),
                player_max
            ),
        );

        // --- Active-status lines: previously Defending, Ice Armor, an
        // active counter, and enemy damage-over-time all existed as real
        // state with zero visual presence. One combined line per
        // combatant, shown whenever any of that combatant's statuses are
        // active. Enemy's goes below its HP bar (clear of the portrait,
        // which ends at pixel y=320 / row 40). Player's goes ABOVE its
        // name/HP block instead of below: the player portrait starts at
        // pixel y=480 / row 60, so a status line at row 60 would sit
        // directly under the portrait on console 3 (registered after
        // console 2) and never actually be visible - row 57 keeps clear.
        if let Some(dot) = &battle.enemy_dot {
            ctx.print_color(
                96,
                43,
                RED,
                BLACK,
                &format!("{} ({} turns left)", dot.label, dot.turns_remaining),
            );
        }

        let mut player_statuses = Vec::new();
        if battle.player_defending {
            player_statuses.push("Defending".to_string());
        }
        if let Some(armor) = entity_ice_armor(&self.ecs, battle.player) {
            player_statuses.push(format!("Ice Armor ({} left)", armor.attacks_remaining));
        }
        if battle.countering.is_some() {
            player_statuses.push("Countering".to_string());
        }
        if !player_statuses.is_empty() {
            ctx.print_color(32, 57, CYAN, BLACK, &player_statuses.join(" | "));
        }

        // --- Battle log: up to MAX_LOG_LINES most-recent lines, in a
        // bordered box centered above the player (not the whole screen) -
        // the player portrait spans console-2 columns 32-64, centered on
        // column 48, so the box is centered there too. Sits in the gap
        // between the enemy's text block (ends row 43) and the player's
        // status/name/HP block (starts row 57), with a line of padding on
        // both sides.
        const MSG_BOX_X: i32 = 36;
        const MSG_BOX_Y: i32 = 45;
        const MSG_BOX_WIDTH: i32 = 24;
        const MSG_BOX_HEIGHT: i32 = MAX_LOG_LINES as i32 + 2;

        let mut log_batch = DrawBatch::new();
        log_batch.target(2);
        draw_hollow_box(
            &mut log_batch,
            MSG_BOX_X,
            MSG_BOX_Y,
            MSG_BOX_WIDTH,
            MSG_BOX_HEIGHT,
            ColorPair::new(WHITE, BLACK),
        );
        log_batch.submit(0).expect("Batch error");

        for (i, line) in battle.log.iter().enumerate() {
            ctx.print_color(MSG_BOX_X + 2, MSG_BOX_Y + 1 + i as i32, WHITE, BLACK, line);
        }

        // --- Floating damage numbers: bigger (console 5's 32px cells,
        // same "big text" console used for title/class-select screens),
        // and centered directly over each portrait now rather than off to
        // the side - big enough now to read clearly on top of the sprite
        // instead of needing to dodge it. Console 5 is registered last, so
        // it renders above the portraits, and every console gets
        // ctx.cls()'d at the top of every frame (see State::tick), so
        // nothing lingers once a popup's timer expires.
        //
        // Both the portrait console (5x5) and this one (40x25) cover the
        // same physical 1280x800 window. Enemy portrait spans columns
        // 24-32 (center 28), rows 5-10 (center 7). Player portrait spans
        // columns 8-16 (center 12), rows 15-20 (center 17). print_color
        // draws left-to-right from the given column, so the start column
        // is nudged left by half the number's length to actually center
        // it rather than just its left edge.
        ctx.set_active_console(5);
        if let Some(popup) = &battle.enemy_damage_popup {
            let text = format!("-{}", popup.amount);
            let start_col = 28 - (text.chars().count() as i32) / 2;
            ctx.print_color(start_col, 7, RED, BLACK, &text);
        }
        if let Some(popup) = &battle.player_damage_popup {
            let text = format!("-{}", popup.amount);
            let start_col = 12 - (text.chars().count() as i32) / 2;
            ctx.print_color(start_col, 17, RED, BLACK, &text);
        }
        ctx.set_active_console(2);

        // --- Actions box, on the HUD console (107x67 grid, ~12px cells -
        // the same "1.5x" size used for the dungeon HUD) rather than the
        // fine-text console (8px, too small) or the big-text title console
        // (32px, too big) - a middle ground per your feedback. BOX_X=44 is
        // deliberate: the player portrait is drawn at column 1 of the
        // 5-column portrait console (256-512px), and 44*~12=528px clears
        // that portrait's right edge (512px) with a little margin, at any
        // box height, since only the box's top edge moves with action
        // count.
        //
        // BOX_Y aligns the box's top edge with the player portrait's top
        // edge instead of bottom-anchoring to the HUD console. The player
        // portrait sits at row 3 of the 5-row portrait console (both
        // consoles cover the same physical 1280x800 window): row 3 starts
        // at 3 * (800/5) = 480px down, which lands at HUD row
        // 480 / (800/67) = ~40 on the HUD console's finer grid.
        //
        // Drawn here, before the match on battle.turn, so it's visible on
        // every battle_tick frame - PlayerMenu, FirstResult, and
        // SecondResult alike - rather than disappearing while a result
        // message is on screen. `actions` is computed here too since both
        // the box's labels and PlayerMenu's key-selection logic below need
        // the same list.
        let actions = available_actions(&self.ecs, battle.player);

        const BOX_X: i32 = 44;
        const BOX_WIDTH: i32 = 26;
        const BOX_Y: i32 = 40;
        let box_height = actions.len() as i32 + 4;
        // Clamp so a tall action list (more techniques than fit below row
        // 40) never runs off the bottom of the console.
        let box_y = BOX_Y.min(HUD_ROWS - box_height);

        let mut menu_batch = DrawBatch::new();
        menu_batch.target(4);
        draw_hollow_box(
            &mut menu_batch,
            BOX_X,
            box_y,
            BOX_WIDTH,
            box_height,
            ColorPair::new(GREEN, BLACK),
        );
        menu_batch.submit(0).expect("Batch error");

        ctx.set_active_console(4);
        ctx.print_color(BOX_X + 1, box_y + 1, YELLOW, BLACK, "Actions");
        for (i, entry) in actions.iter().enumerate() {
            // Every action this class could ever have is always listed
            // (see battle::available_actions) - one not currently owned
            // shows greyed out and isn't selectable, rather than
            // disappearing from the menu entirely, so the list stays a
            // stable reference of what the class can eventually do.
            let (label, color) = if entry.action.is_some() {
                let label = match entry.count {
                    Some(n) => format!("{}) {} x{}", i + 1, entry.label, n),
                    None => format!("{}) {}", i + 1, entry.label),
                };
                (label, GREEN)
            } else {
                (format!("{}) {} (locked)", i + 1, entry.label), DARK_GRAY)
            };
            ctx.print_color(BOX_X + 1, box_y + 3 + i as i32, color, BLACK, &label);
        }
        // Restore console 2 - the enemy/player name+HP text above and
        // every match arm below assume it's active (it's set once, above
        // the whole match block, not re-set per arm).
        ctx.set_active_console(2);

        match battle.turn {
            BattleTurn::PlayerMenu => {
                if let Some(key) = ctx.key {
                    let chosen = number_key_index(key)
                        .and_then(|i| actions.get(i))
                        .and_then(|entry| entry.action);
                    if let Some(chosen) = chosen {
                        // Every technique's mechanical effect is resolved
                        // in one place (battle::apply_player_technique)
                        // rather than a match arm per item here - adding a
                        // new class's technique needs no main.rs change.
                        match chosen {
                            BattleAction::Attack => {
                                let mut dmg = player_attack_damage(&self.ecs, battle.player);
                                if battle.sneak_attack {
                                    dmg *= 3;
                                }
                                let dmg = apply_damage(&mut self.ecs, battle.enemy, dmg);
                                battle.show_enemy_damage(dmg);
                                battle.player_flash =
                                    Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
                                battle.enemy_flash =
                                    Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
                                battle.push_log(if dmg == 0 {
                                    "Dodge attack.".to_string()
                                } else {
                                    format!("Deal {} damage.", dmg)
                                });
                            }
                            BattleAction::Defend => {
                                battle.player_defending = true;
                                battle.push_log("Defend.".to_string());
                            }
                            BattleAction::Flee => {
                                battle.fled = true;
                                battle.push_log("Flee.".to_string());
                            }
                            BattleAction::Technique(item) => {
                                let result =
                                    apply_player_technique(&mut self.ecs, &mut battle, item);
                                battle.push_log(result);
                            }
                        }
                        // Sneak attack is a one-shot ambush bonus for the
                        // guaranteed first action only - clear it here
                        // regardless of which action was actually chosen,
                        // so it can never linger and apply again later in
                        // the same fight.
                        battle.sneak_attack = false;
                        // The player is first_actor at the start of a round
                        // they act in unprompted; if the enemy already
                        // opened the round (first_actor == Enemy), this
                        // menu is the player's second action instead.
                        battle.enter_result(if battle.first_actor == Combatant::Player {
                            BattleTurn::FirstResult
                        } else {
                            BattleTurn::SecondResult
                        });
                    }
                }
            }
            BattleTurn::FirstResult => {
                // Auto-advances once result_timer_ms runs out (see
                // Battle::enter_result/RESULT_AUTO_ADVANCE_MS) - a keypress
                // still skips ahead immediately, it just isn't required.
                battle.result_timer_ms -= ctx.frame_time_ms;
                ctx.print_color_centered(51, YELLOW, BLACK, "(press any key to skip ahead)");
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if battle.fled {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    match battle.first_actor {
                        Combatant::Player => {
                            // Player went first and attacked the enemy -
                            // check whether that finished the fight before
                            // letting the enemy retaliate.
                            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                            if enemy_hp_now < 1 {
                                let mut rng = RandomNumberGenerator::new();
                                let loot = grant_random_battle_loot(
                                    &mut self.ecs,
                                    &mut rng,
                                    battle.player,
                                );
                                let mut cb = CommandBuffer::new(&mut self.ecs);
                                cb.remove(battle.enemy);
                                cb.flush(&mut self.ecs);
                                self.resources.insert(Some(BattleVictory {
                                    player: battle.player,
                                    enemy_name: battle.enemy_name.clone(),
                                    loot,
                                }));
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::BattleVictory);
                                return;
                            }
                            resolve_enemy_attack(&mut self.ecs, &mut battle);
                            battle.enter_result(BattleTurn::SecondResult);
                        }
                        Combatant::Enemy => {
                            // Enemy went first (they're faster) and already
                            // attacked the player - check whether that
                            // ended things before the player gets a turn.
                            let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                            if player_hp_now < 1 {
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::GameOver);
                                return;
                            }

                            // A Counter Attack can kill the enemy as a
                            // side effect of their own attack (see
                            // resolve_enemy_attack) - check for that too,
                            // or the player gets an extra prompt against
                            // an already-dead enemy.
                            let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                            if enemy_hp_now < 1 {
                                let mut rng = RandomNumberGenerator::new();
                                let loot = grant_random_battle_loot(
                                    &mut self.ecs,
                                    &mut rng,
                                    battle.player,
                                );
                                let mut cb = CommandBuffer::new(&mut self.ecs);
                                cb.remove(battle.enemy);
                                cb.flush(&mut self.ecs);
                                self.resources.insert(Some(BattleVictory {
                                    player: battle.player,
                                    enemy_name: battle.enemy_name.clone(),
                                    loot,
                                }));
                                self.resources.insert(None::<Battle>);
                                self.resources.insert(TurnState::BattleVictory);
                                return;
                            }

                            battle.turn = BattleTurn::PlayerMenu;
                        }
                    }
                }
            }
            BattleTurn::SecondResult => {
                battle.result_timer_ms -= ctx.frame_time_ms;
                ctx.print_color_centered(51, YELLOW, BLACK, "(press any key to skip ahead)");
                if ctx.key.is_some() || battle.result_timer_ms <= 0.0 {
                    if battle.fled {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    // Whoever acted second this round attacked whoever
                    // acted first - but a Counter Attack can also kill the
                    // enemy as a side effect of an enemy attack regardless
                    // of who that attack's "real" target was, so check
                    // both sides here rather than just the expected one.
                    let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                    let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);

                    if player_hp_now < 1 {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::GameOver);
                        return;
                    }

                    if enemy_hp_now < 1 {
                        let mut rng = RandomNumberGenerator::new();
                        let loot = grant_random_battle_loot(&mut self.ecs, &mut rng, battle.player);
                        let mut cb = CommandBuffer::new(&mut self.ecs);
                        cb.remove(battle.enemy);
                        cb.flush(&mut self.ecs);
                        self.resources.insert(Some(BattleVictory {
                            player: battle.player,
                            enemy_name: battle.enemy_name.clone(),
                            loot,
                        }));
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::BattleVictory);
                        return;
                    }

                    battle.turn = BattleTurn::PlayerMenu;
                    battle.awaiting_order_decision = true;
                }
            }
        }

        self.resources.insert(Some(battle));
    }

    fn battle_victory_tick(&mut self, ctx: &mut BTerm) {
        let victory_snapshot = self
            .resources
            .get::<Option<BattleVictory>>()
            .unwrap()
            .clone();
        let victory = match victory_snapshot {
            Some(v) => v,
            None => {
                // Shouldn't happen, but don't get stuck if it does.
                self.resources.insert(TurnState::AwaitingInput);
                return;
            }
        };

        let player_render = entity_render_component(&self.ecs, victory.player);
        self.draw_battle_arena(None, player_render, None, None);

        ctx.set_active_console(2);
        ctx.print_color_centered(
            45,
            GREEN,
            BLACK,
            &format!("You defeated the {}!", victory.enemy_name),
        );
        match &victory.loot {
            Some(item) => {
                ctx.print_color_centered(48, YELLOW, BLACK, &format!("You found: {}!", item));
            }
            None => {
                ctx.print_color_centered(48, WHITE, BLACK, "No loot this time.");
            }
        }
        ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");

        if ctx.key.is_some() {
            self.resources.insert(None::<BattleVictory>);
            self.resources.insert(TurnState::AwaitingInput);
        }
    }

    fn game_over(&mut self, ctx: &mut BTerm) {
        // Red, dimmed version of the actual dungeon the run ended in,
        // plus the fallen hero's own glyph - rotated onto its side, tinted
        // red, with a genuinely transparent background this time (see
        // draw_end_screen_fallen_portrait) instead of an opaque quad
        // matched to the arena color.
        self.draw_end_screen_background(RGB::from_f32(1.0, 0.4, 0.4));
        self.draw_end_screen_fallen_portrait(RED.into());

        // Header on the big-text console (console 5, 32px cells - same
        // one the title screen's "EVER SPACE RRPG" uses). Body text below
        // it moved from console 2 (fine 8px) to console 4 (the HUD
        // console, ~12px cells - the same "1.5x bigger" text already used
        // for the dungeon HUD) so it isn't dwarfed by the header, and row
        // positions are worked out in pixels (not row counts) so nothing
        // overlaps across these differently-scaled consoles: header row 2
        // on console 5 bottoms out at (2+1)*32 = 96px -> console 4 row 9
        // (~108px) clears it; the fallen portrait (see
        // draw_end_screen_fallen_portrait) is centered at y=400px and, at
        // END_SCREEN_FALLEN_SCALE, spans roughly 304-496px -> console 4
        // row 45 (~537px) clears its bottom edge with margin.
        ctx.set_active_console(5);
        ctx.print_color_centered(2, RED, BLACK, "Your quest has ended.");

        ctx.set_active_console(4);
        ctx.print_color_centered(
            10,
            WHITE,
            BLACK,
            "Slain by a monster, your hero's journey has come to a premature end.",
        );
        ctx.print_color_centered(
            13,
            WHITE,
            BLACK,
            "The Amulet of Yala remains unclaimed, and your home town is not saved.",
        );
        // Below this point: the fallen portrait, centered on-screen (see
        // draw_end_screen_fallen_portrait). These two lines sit clear
        // beneath it.
        ctx.print_color_centered(
            45,
            YELLOW,
            BLACK,
            "Don't worry, you can always try again with a new hero.",
        );
        ctx.print_color_centered(48, GREEN, BLACK, "Press 1 to return to the title screen.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.return_to_title();
        }
    }

    fn victory(&mut self, ctx: &mut BTerm) {
        // Warm gold version of the actual dungeon the run was won in,
        // plus the hero's own glyph glowing gold - see
        // draw_end_screen_background/draw_end_screen_portrait.
        self.draw_end_screen_background(RGB::from_f32(1.0, 0.85, 0.45));
        self.draw_end_screen_portrait(YELLOW.into());

        ctx.set_active_console(2);
        ctx.print_color_centered(2, GREEN, BLACK, "You have won!");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "You put on the Amulet of Yala and feel its power course through your veins.",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "Your town is saved, and you can return to your normal life.",
        );
        ctx.print_color_centered(7, GREEN, BLACK, "Press 1 to return to the title screen.");
        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.return_to_title();
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(0);
        ctx.cls();
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(2);
        ctx.cls();
        ctx.set_active_console(3);
        ctx.cls();
        ctx.set_active_console(4);
        ctx.cls();
        ctx.set_active_console(5);
        ctx.cls();
        ctx.set_active_console(END_SCREEN_FALLEN_CONSOLE);
        ctx.cls();
        self.resources.insert(ctx.key);
        self.resources.insert(FrameTime(ctx.frame_time_ms));
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        ctx.set_active_console(4);
        self.resources
            .insert(HudMousePos(Point::from_tuple(ctx.mouse_pos())));
        ctx.set_active_console(0);
        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::TitleScreen => {
                self.title_screen(ctx);
            }
            TurnState::ClassSelect => {
                self.class_select(ctx);
            }
            TurnState::AwaitingInput => self
                .input_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::PlayerTurn => {
                self.player_systems
                    .execute(&mut self.ecs, &mut self.resources);
            }
            TurnState::MonsterTurn => self
                .monster_systems
                .execute(&mut self.ecs, &mut self.resources),
            TurnState::Paused => {
                self.paused_tick(ctx);
            }
            TurnState::InBattle => {
                self.battle_tick(ctx);
            }
            TurnState::BattleVictory => {
                self.battle_victory_tick(ctx);
            }
            TurnState::GameOver => {
                self.game_over(ctx);
            }
            TurnState::Victory => {
                self.victory(ctx);
            }
            TurnState::NextLevel => {
                self.advance_level();
            }
        }
        render_draw_buffer(ctx).expect("Render error");
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_title("Ever Space RRPG")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "terminal8x8.png")
        .with_simple_console_no_bg(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "dungeonfont.png",
        )
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        // Console 6 (END_SCREEN_FALLEN_CONSOLE): a "fancy console" -
        // supports DrawBatch::set_fancy (sub-pixel position + rotation +
        // scale), unlike every other console above which is a plain
        // "simple console". Same square 32x32px cells as console 0/1
        // (dungeonfont at native size on the DISPLAY_WIDTH x
        // DISPLAY_HEIGHT grid), so a 90-degree rotation doesn't
        // stretch/squash the glyph. Used only by
        // draw_end_screen_fallen_portrait for the GameOver screen.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_vsync(false)
        .build()?;

    main_loop(context, State::new())
}
