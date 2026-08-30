#![warn(clippy::pedantic)]

mod arena;
mod battle;
mod camera;
mod components;
mod keymap;
mod map;
mod map_builder;
mod render_helpers;
mod screens;
mod spawner;
mod stats;
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
    // game-over/victory/pause), 3 battle portraits, 4 MAP_SCROLL_CONSOLE,
    // 5 ENTITY_SCROLL_CONSOLE, 6 HUD_CONSOLE, 7 BIG_TEXT_CONSOLE, 8
    // END_SCREEN_FALLEN_CONSOLE, 9 GLIDE_CONSOLE, 10
    // BATTLE_PORTRAIT_WIGGLE_CONSOLE. HUD_CONSOLE and BIG_TEXT_CONSOLE
    // used to be plain literals 4 and 5 - promoted to named constants
    // (like everything from END_SCREEN_FALLEN_CONSOLE on already was)
    // specifically because MAP_SCROLL_CONSOLE/ENTITY_SCROLL_CONSOLE
    // needed to be inserted BEFORE them in registration order (see their
    // own doc comments below for why), which pushed every console from
    // the old 4 onward up by two - a plain-literal "4" or "5" anywhere
    // in the codebase would have silently kept meaning the OLD console
    // after this change, not the new one at that slot.
    /// Console 4: a "fancy console" (supports DrawBatch::set_fancy), same
    /// DISPLAY_WIDTH x DISPLAY_HEIGHT grid and dungeonfont as console 0,
    /// with an opaque background exactly like console 0's (unlike every
    /// other fancy console in this file, which deliberately uses a
    /// transparent one - see GLIDE_CONSOLE). Used only by map_render
    /// (systems/map_render.rs) for the small fraction of frames where
    /// camera_render_offset (components.rs) returns Some - i.e. the
    /// player is mid-glide and the camera itself needs to visibly pan
    /// rather than snap. A plain console like console 0 can only ever be
    /// drawn to at integer cell positions, so there's no way to give it a
    /// sub-pixel scroll offset directly; every tile has to be redrawn via
    /// set_fancy at a fractional position instead, which needs a fancy
    /// console the same way the entity glide did. Registered right after
    /// console 3 (battle portraits) and before HUD_CONSOLE specifically
    /// so it keeps sitting BELOW the HUD in z-order exactly like console
    /// 0 always has - registering it after HUD_CONSOLE instead would have
    /// meant the (fully opaque, full-screen) scrolling map painting over
    /// the health bar and item list on every single step.
    pub const MAP_SCROLL_CONSOLE: usize = 4;
    /// Console 5: a second new "fancy console", same grid as console 4
    /// above, transparent background (the GLIDE_CONSOLE trick). Used by
    /// entity_render (systems/entity_render.rs) on the same frames as
    /// MAP_SCROLL_CONSOLE, for the same reason: once the camera itself is
    /// panning, EVERY entity - not just one that's individually
    /// mid-glide - needs to be drawn at a fractional position derived
    /// from that same pan, or a stationary entity would stay rigidly
    /// snapped to its old integer screen cell while the map slides
    /// underneath it. Kept as its own console rather than reusing
    /// GLIDE_CONSOLE because GLIDE_CONSOLE is registered AFTER
    /// HUD_CONSOLE/BIG_TEXT_CONSOLE/etc. (see those consoles' own
    /// history) and this needs to land in the same "below the HUD" slot
    /// as MAP_SCROLL_CONSOLE just above it - the two are only ever drawn
    /// to on the same frames as each other, never mixed with
    /// GLIDE_CONSOLE's own (different) use case in a single frame.
    pub const ENTITY_SCROLL_CONSOLE: usize = 5;
    /// Console 6 (was console 4 before MAP_SCROLL_CONSOLE/
    /// ENTITY_SCROLL_CONSOLE were inserted above it): the dungeon HUD -
    /// health bar, item lists, tooltips. See HUD_COLS/HUD_ROWS above.
    pub const HUD_CONSOLE: usize = 6;
    /// Console 7 (was console 5): big title/class-select text, and the
    /// large text used by the end/victory screens - DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT cols/rows (the dungeon view's own grid) on the
    /// small text font instead of the dungeon font, landing at 32x32px
    /// cells, 4x console 2's 8px text.
    pub const BIG_TEXT_CONSOLE: usize = 7;
    /// Console 8 (was console 6): a "fancy console" (supports
    /// DrawBatch::set_fancy, incl. rotation) - same DISPLAY_WIDTH x
    /// DISPLAY_HEIGHT grid and dungeonfont as console 0, so cells are the
    /// same 32x32px squares, giving a clean (non-stretched) rotation.
    /// Used only by draw_end_screen_fallen_portrait for the GameOver
    /// screen's fallen hero - see main()'s builder chain and State::tick's
    /// console-clear block.
    pub const END_SCREEN_FALLEN_CONSOLE: usize = 8;
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
    /// Console 9 (was console 7): a second "fancy console", same
    /// DISPLAY_WIDTH x DISPLAY_HEIGHT grid/32x32px cells as console
    /// 0/1/8. Used by entity_render (systems/entity_render.rs) to draw
    /// any entity currently mid-tile-glide (see
    /// components::gliding_position / MovingAnimation) at a sub-pixel
    /// position instead of snapping straight to its destination tile on
    /// console 1, for the (more common) case where the camera ITSELF
    /// isn't panning - see camera_render_offset and
    /// ENTITY_SCROLL_CONSOLE above for the "camera is also panning"
    /// case, which uses a different console instead of this one.
    /// Registered after console 1, so it paints over it - the gliding
    /// entity is deliberately NOT also drawn on console 1 for the same
    /// frame, so there's no double-draw, just a handoff between the two
    /// consoles for the duration of the glide. Relies on the same
    /// genuinely-transparent-background trick END_SCREEN_FALLEN_CONSOLE
    /// proved out (RGBA alpha 0) - without it, a fancy console's normally
    /// opaque background quad would paint a visible box sliding over the
    /// map every time something moved.
    pub const GLIDE_CONSOLE: usize = 9;
    /// Console 10 (was console 8): a third "fancy console", sharing
    /// BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS's coarse 5x5 grid with
    /// console 3 (same font, same physical window) - since a low
    /// column/row count over the same window automatically yields huge
    /// cells (that's the entire "big portrait" trick console 3 already
    /// uses), this needs no separate scale factor the way
    /// END_SCREEN_FALLEN_CONSOLE did. Used only by draw_battle_arena
    /// (screens/battle.rs) to give the currently "Attacking" portrait a
    /// small shake (see render_helpers::attack_wiggle_offset) via
    /// sub-pixel positioning - something console 3's plain per-cell
    /// `set()` can't do. Relies on the same transparent-background trick
    /// as GLIDE_CONSOLE/END_SCREEN_FALLEN_CONSOLE (RGBA alpha 0): this
    /// was tried once before, early in this project, and abandoned
    /// specifically because a fancy console's normally-opaque background
    /// revealed a visibly sliding box over the static arena behind it -
    /// see the battle-portrait jiggle history in journal.md. That's the
    /// one thing that's changed since; the rest of this console's setup
    /// is otherwise identical in spirit to that first attempt.
    pub const BATTLE_PORTRAIT_WIGGLE_CONSOLE: usize = 10;
    pub use crate::arena::*;
    pub use crate::battle::*;
    pub use crate::camera::*;
    pub use crate::components::*;
    pub use crate::keymap::*;
    pub use crate::map::*;
    pub use crate::map_builder::*;
    pub use crate::render_helpers::*;
    pub use crate::spawner::*;
    pub use crate::stats::*;
    pub use crate::systems::*;
    pub use crate::turn_state::*;
}

use prelude::*;

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
    /// Which Action the Options screen is currently waiting for a new
    /// key for, if any - None means the screen is just showing the list
    /// (see screens/options.rs). Kept as a plain State field rather than
    /// an ECS resource since it's transient screen-navigation state, not
    /// anything gameplay systems need to see, and State's other
    /// Resources::default() reset points (start_game/return_to_title)
    /// have no reason to touch it either way.
    options_awaiting: Option<Action>,
    /// Which screen Options should return to on Escape - TitleScreen or
    /// Paused, whichever one it was opened from (see title.rs's and
    /// pause.rs's 'O' handlers, which both set this right before
    /// entering TurnState::Options). Same "plain State field, not a
    /// resource" reasoning as options_awaiting above.
    options_return_to: TurnState,
    /// Which sub-view the History screen is currently showing - see
    /// StatsViewMode's own doc comment. Same "plain State field, not a
    /// resource" reasoning as options_awaiting.
    stats_view_mode: StatsViewMode,
    /// Which adventure type was picked at the new AdventureSelect screen
    /// - read by class_select to decide whether to call start_game
    /// (Dungeon Crawl) or start_arena (Battle Arena). Same "plain State
    /// field, not a resource" reasoning as options_awaiting - this is
    /// screen-navigation state, not anything a gameplay system reads.
    /// Defaults to DungeonCrawl so the Debug hidden shortcut in
    /// class_select (which can be reached without ever visiting
    /// AdventureSelect, if that ever changes) keeps its old behavior.
    adventure_mode: AdventureMode,
}

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
        // Same reasoning as Option<Battle> just above -
        // movement_system now reads Option<ShoppingActive> too (see
        // systems/movement.rs's auto-pickup gate), so it must exist
        // before the first background_movement_systems.execute() call.
        resources.insert(None::<ShoppingActive>);
        // Same reasoning again - movement_system also reads
        // Option<ArenaRun> now (see the LOS-freeze fix in
        // systems/movement.rs). Missing this is exactly what caused a
        // startup panic: legion's resource fetch has no "missing means
        // None" fallback the way an Option value's own None does - the
        // RESOURCE SLOT ITSELF has to exist, or the fetch panics
        // outright, even though the type being fetched is an Option.
        resources.insert(None::<ArenaRun>);
        // player_input_system and hud_system (both part of
        // build_input_scheduler, run during TurnState::AwaitingInput) now
        // read Option<ShopMessage> - unlike Option<Battle>/ArenaRun/
        // ShoppingActive above, that scheduler never runs during the
        // title-background schedules, so this isn't strictly needed here
        // for THAT reason, but every full Resources::default() reset point
        // gets it anyway for the same defensive consistency (start_game/
        // start_arena/return_to_title all do too) - cheap insurance
        // against the exact "forgot one reset point" class of startup
        // panic this project has already hit once for a different
        // resource.
        resources.insert(None::<ShopMessage>);
        // Loaded once here and re-inserted after every Resources::default()
        // reset point below (start_game/return_to_title also wipe every
        // resource) - Keymap::load reads from disk each time, so a rebind
        // made in one run is still there after starting a fresh one or
        // returning to the title screen, without needing a separate
        // long-lived copy on State itself.
        resources.insert(Keymap::load());
        resources.insert(Stats::load());
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
            options_awaiting: None,
            options_return_to: TurnState::TitleScreen,
            stats_view_mode: StatsViewMode::Overview,
            adventure_mode: AdventureMode::DungeonCrawl,
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
        spawn_boss(&mut self.ecs, &mut rng, 0, map_builder.amulet_start);
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(Keymap::load());
        // Always present (see systems/end_turn.rs's Exit-tile branch) -
        // None here means "this is an ordinary dungeon crawl", not
        // "unknown". start_arena is the only place this is ever Some.
        self.resources.insert(None::<ArenaRun>);
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ShopMessage>);

        // Counts as "this class was chosen" the instant a run actually
        // begins, regardless of how it later ends (won, lost, or
        // abandoned via quit-to-title) - see Stats::record_game_started.
        let mut stats = Stats::load();
        stats.record_game_started(class, AdventureMode::DungeonCrawl);
        self.resources.insert(stats);
    }

    /// Builds a fresh Battle Arena run for `class` - the arena-mode
    /// counterpart to start_game, called from class_select when
    /// adventure_mode is BattleArena instead of DungeonCrawl. Rather than
    /// a normal dungeon floor, this drops the player straight into the
    /// starting shop (level 0 gear) - see MapBuilder::new_arena_shop.
    /// Deliberately does NOT call grant_starting_items: in Arena mode the
    /// shop itself is the player's starting kit, so granting the normal
    /// dungeon-crawl kit on top would be a double allocation.
    fn start_arena(&mut self, class: &str) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let arena_run = ArenaRun::new(1);
        let items = roll_arena_shop_items(&mut rng, class, arena_run.template_level());
        let (
            mut map_builder,
            item_points,
            shopkeeper_point,
            reveal_x,
            reveal_y,
            reveal_w,
            reveal_h,
        ) = MapBuilder::new_arena_shop(&mut rng, items.len());
        // No starting kit to grant here (see this fn's doc comment), but
        // the returned entity IS needed below to force full visibility.
        let player = spawn_player(&mut self.ecs, map_builder.player_start, class);

        // No fog of war inside the reveal rectangle - deliberately NOT
        // the whole 80x50 map (every Map is that size regardless of what
        // any one MapBuilder actually uses, and revealing all of it is
        // what made the arena shop look like a tiny room floating in an
        // enormous dungeon). Leaving everything outside this rectangle
        // unrevealed means nothing renders there at all (see
        // components::tile_render_at's bounds/reveal check) - a plain
        // black boundary, not more wall - which is what actually makes
        // this map read as small. Same underlying trick
        // spawn_title_background uses for the decorative title-screen
        // background (revealed_tiles true + the entity's own
        // FieldOfView pre-filled and frozen with is_dirty = false so
        // it's never recomputed), just bounded to a small rectangle
        // instead of the whole map. This also still matters for the
        // shopkeeper specifically: it sits behind the counter (a
        // Counter tile, opaque like a Wall), and real shadowcasting
        // would never let the player see past that counter at all, no
        // matter how close they stood.
        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                map_builder.map.revealed_tiles[map_idx(x, y)] = true;
            }
        }
        let mut full_fov = FieldOfView::new(8);
        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                full_fov.visible_tiles.insert(Point::new(x, y));
            }
        }
        full_fov.is_dirty = false;
        let mut commands = legion::systems::CommandBuffer::new(&self.ecs);
        commands.add_component(player, full_fov);
        // Only a Battle Arena player ever gets a Gold component at all -
        // see Gold's own doc comment for why every gold codepath treats
        // that presence, not a separate mode check, as the source of
        // truth.
        commands.add_component(player, Gold(ARENA_STARTING_GOLD));
        commands.flush(&mut self.ecs);

        // Shopkeeper - purely decorative for now (no dialogue/trade
        // logic, the items themselves are what's interactive). Glyph
        // 'W' was picked because it's the one letter glyph documented as
        // "free and unassigned" in Dungeon_Font_Glyph_to_Cell_Map.md -
        // every other letter already has real custom art for a class,
        // enemy, or boss. Renders as a plain default 'W' until a future
        // art pass draws real shopkeeper pixel art into that cell (row
        // 5, col 7 - see the master map).
        self.ecs.push((
            Name("Shopkeeper".to_string()),
            shopkeeper_point,
            Render {
                color: ColorPair::new(YELLOW, BLACK),
                glyph: to_cp437('W'),
            },
        ));

        self.spawn_arena_shop_items(&items, &item_points);

        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;

        self.resources.insert(map_builder.map);
        // Plain Camera::new - the standard 40x25 dungeon viewport, same
        // as every other map. A custom smaller camera was tried here
        // first, but it didn't address the actual problem (the map
        // itself still being 80x50 underneath) and complicated other
        // things unnecessarily - the reveal-rectangle approach above is
        // what actually makes this map read as small.
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(Keymap::load());
        self.resources.insert(Some(arena_run));
        self.resources.insert(Some(ShoppingActive));
        self.resources.insert(None::<ShopMessage>);

        let mut stats = Stats::load();
        stats.record_game_started(class, AdventureMode::BattleArena);
        self.resources.insert(stats);
    }

    /// Places this shop's already-decided stock (see
    /// arena::roll_arena_shop_items, which runs BEFORE the room is even
    /// built, so MapBuilder::new_arena_shop can size the counter to fit)
    /// onto `item_points` - one ShopStock counter marker per entry, in
    /// order, each carrying its own Price. Split out of start_arena so the
    /// next slice (a shop reached after a boss kill, not just the starting
    /// one) can call this same logic against a freshly-built shop map
    /// without duplicating it.
    fn spawn_arena_shop_items(&mut self, items: &[(String, i32, i32)], item_points: &[Point]) {
        for ((name, quantity, price), &pt) in items.iter().zip(item_points.iter()) {
            spawn_shop_stock_at(&mut self.ecs, name, pt, *quantity, *price);
        }
    }

    /// Reached by stepping on a shop's stairs tile while ArenaRun is
    /// active (see TurnState::ArenaTransition / systems/end_turn.rs) -
    /// always means "begin wave 1 of whatever level this shop was
    /// preparing you for", whether that shop was the very first one or
    /// one reached after a boss kill. One-shot, like advance_level: runs
    /// once, changes TurnState away from ArenaTransition so it doesn't
    /// repeat next frame.
    fn arena_transition_tick(&mut self, _ctx: &mut BTerm) {
        let run = self
            .resources
            .get::<Option<ArenaRun>>()
            .unwrap()
            .clone()
            .expect("ArenaTransition reached without an active ArenaRun");
        self.arena_begin_wave(run.level, 1);
    }

    /// Reached when systems/end_turn.rs detects the last Enemy died via a
    /// non-battle mechanism (a Throw Spear/Shoot ranged strike, or a
    /// placed Trap) while an Arena wave or boss encounter was active -
    /// see TurnState::ArenaWaveCleared's own doc comment for the full
    /// reasoning. Just reuses handle_arena_kill, the exact same
    /// orchestration the normal battle-victory dismissal path already
    /// runs - this is only a different way of REACHING that call, not a
    /// second copy of its logic.
    fn arena_wave_cleared_tick(&mut self) {
        let run = self
            .resources
            .get::<Option<ArenaRun>>()
            .unwrap()
            .clone()
            .expect("ArenaWaveCleared reached without an active ArenaRun");
        self.handle_arena_kill(run);
    }

    /// Removes every entity except the player and whatever they're
    /// carrying - the same entity-preservation pattern advance_level
    /// already uses to move a dungeon-crawl player to a fresh floor
    /// without losing their inventory. Shared by every Arena transition
    /// that needs a clean map but the SAME player/inventory: starting a
    /// new wave, and moving on to the next level's shop.
    fn arena_rebuild_keep_player(&mut self) -> Entity {
        let player_entity = *<Entity>::query()
            .filter(component::<Player>())
            .iter(&self.ecs)
            .nth(0)
            .unwrap();

        let mut entities_to_keep = std::collections::HashSet::new();
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

        player_entity
    }

    /// Builds a fresh wave arena map for `level`/`wave`, keeping the
    /// same player entity and inventory (see arena_rebuild_keep_player),
    /// spawns that wave's enemies, and forces full visibility across the
    /// map's reveal rectangle - the same "no fog of war" trick
    /// start_arena uses for the shop, applied here because the design
    /// calls for the whole arena being visible at once rather than
    /// explored tile by tile. Updates the ArenaRun resource to reflect
    /// the new wave and stores this map's boss_spawn point for later
    /// (see ArenaRun::boss_spawn's doc comment).
    fn arena_begin_wave(&mut self, level: u8, wave: u8) {
        let player_entity = self.arena_rebuild_keep_player();
        let mut rng = RandomNumberGenerator::new();
        let template_level = (level - 1) as usize;
        let enemy_count = ARENA_WAVE_ENEMY_COUNTS[(wave - 1) as usize] as usize;
        let (mut map_builder, enemy_spawns, boss_spawn, reveal_x, reveal_y, reveal_w, reveal_h) =
            MapBuilder::new_arena_wave(&mut rng, enemy_count);

        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, map_builder.player_start);
        cb.flush(&mut self.ecs);

        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                map_builder.map.revealed_tiles[map_idx(x, y)] = true;
            }
        }
        let mut full_fov = FieldOfView::new(8);
        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                full_fov.visible_tiles.insert(Point::new(x, y));
            }
        }
        full_fov.is_dirty = false;
        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, full_fov);
        cb.flush(&mut self.ecs);

        spawn_prefab_enemies(&mut self.ecs, &mut rng, template_level, &enemy_spawns);
        self.boost_arena_enemy_fov();

        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ShopMessage>);
        self.resources.insert(Some(ArenaRun {
            level,
            wave,
            boss_active: false,
            boss_spawn,
        }));
    }

    /// Spawns this level's boss onto the SAME map wave 3 was just fought
    /// on (at the boss_spawn point that map was built with), rather than
    /// building a fresh one - the design calls for the boss appearing
    /// once the last wave clears, not a separate arena of its own.
    fn arena_spawn_boss_on_current_map(&mut self, run: ArenaRun) {
        let mut rng = RandomNumberGenerator::new();
        spawn_boss(
            &mut self.ecs,
            &mut rng,
            run.template_level(),
            run.boss_spawn,
        );
        self.boost_arena_enemy_fov();
        self.resources.insert(Some(ArenaRun {
            boss_active: true,
            ..run
        }));
        self.resources.insert(TurnState::AwaitingInput);
    }

    /// Overrides every current Enemy entity's FieldOfView with
    /// ARENA_ENEMY_FOV_RADIUS - called right after spawning a wave's
    /// enemies or a level's boss, both of which start with the small
    /// dungeon-tuned radius spawn_entity gives every enemy by default.
    /// Safe to apply indiscriminately to every Enemy in the world at
    /// that point: arena_begin_wave already cleared every non-player
    /// entity before spawning this wave's enemies, and this is called
    /// immediately after spawning, so the only Enemy entities that can
    /// possibly exist yet are the ones just spawned this call.
    fn boost_arena_enemy_fov(&mut self) {
        let mut cb = CommandBuffer::new(&mut self.ecs);
        <(Entity, &Enemy)>::query()
            .iter(&self.ecs)
            .for_each(|(e, _)| {
                cb.add_component(*e, FieldOfView::new(ARENA_ENEMY_FOV_RADIUS));
            });
        cb.flush(&mut self.ecs);
    }

    /// Moves the player on to `next_level`'s shop after clearing the
    /// previous level's boss - same player/inventory (see
    /// arena_rebuild_keep_player), a freshly rolled and stocked shop
    /// (see start_arena, which this necessarily duplicates a fair amount
    /// of - a shared "build an arena shop world" helper would be a
    /// reasonable follow-up cleanup, not done here to keep this change
    /// focused on wave/boss orchestration).
    fn arena_advance_to_next_shop(&mut self, next_level: u8) {
        let player_entity = self.arena_rebuild_keep_player();
        let class = entity_class(&self.ecs, player_entity).unwrap_or_default();
        let mut rng = RandomNumberGenerator::new();
        let template_level = (next_level - 1) as usize;
        let items = roll_arena_shop_items(&mut rng, &class, template_level);
        let (
            mut map_builder,
            item_points,
            shopkeeper_point,
            reveal_x,
            reveal_y,
            reveal_w,
            reveal_h,
        ) = MapBuilder::new_arena_shop(&mut rng, items.len());

        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, map_builder.player_start);
        cb.flush(&mut self.ecs);

        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                map_builder.map.revealed_tiles[map_idx(x, y)] = true;
            }
        }
        let mut full_fov = FieldOfView::new(8);
        for y in reveal_y..(reveal_y + reveal_h) {
            for x in reveal_x..(reveal_x + reveal_w) {
                full_fov.visible_tiles.insert(Point::new(x, y));
            }
        }
        full_fov.is_dirty = false;
        let mut cb = CommandBuffer::new(&mut self.ecs);
        cb.add_component(player_entity, full_fov);
        cb.flush(&mut self.ecs);

        self.ecs.push((
            Name("Shopkeeper".to_string()),
            shopkeeper_point,
            Render {
                color: ColorPair::new(YELLOW, BLACK),
                glyph: to_cp437('W'),
            },
        ));

        self.spawn_arena_shop_items(&items, &item_points);

        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;

        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(Some(ArenaRun::new(next_level)));
        self.resources.insert(Some(ShoppingActive));
        self.resources.insert(None::<ShopMessage>);
    }

    /// The single decision point for what happens after an Arena kill -
    /// called from battle.rs's battle_victory_tick when ArenaRun is
    /// active, once the player dismisses the "You defeated X!" screen.
    /// ALSO called from arena_wave_cleared_tick below, for a kill that
    /// happened outside of battle entirely - both are just different
    /// TRIGGERS for this same orchestration. Counts surviving Enemy
    /// entities directly (rather than a separate hand-maintained counter)
    /// so this can never drift out of sync with what's actually still
    /// alive on the map.
    fn handle_arena_kill(&mut self, run: ArenaRun) {
        let enemies_left = <&Enemy>::query().iter(&self.ecs).count();
        if enemies_left > 0 {
            // Still more to fight in this wave/boss encounter - nothing
            // to advance yet.
            self.resources.insert(TurnState::AwaitingInput);
            return;
        }

        if !run.boss_active {
            let next_wave = run.wave + 1;
            if (next_wave as usize) <= ARENA_WAVE_ENEMY_COUNTS.len() {
                self.arena_begin_wave(run.level, next_wave);
            } else {
                // Wave 3 just cleared - the boss appears now, on this
                // same map.
                self.arena_spawn_boss_on_current_map(run);
            }
        } else if run.level < 3 {
            self.arena_advance_to_next_shop(run.level + 1);
        } else {
            // Level 3's boss just fell - the whole run is won.
            self.resources.insert(TurnState::Victory);
        }
    }

    /// Tears down the current run (if any) and returns to the title
    /// screen - called when the player dismisses the GameOver or Victory
    /// screen, instead of immediately starting a new run with whatever
    /// class they last had. ClassSelect (via start_game) is now the only
    /// place a run actually begins. spawn_title_background is defined in
    /// screens/title.rs (a descendant module), which is why it needed to
    /// be marked `pub` - see that file's comment on the same fn.
    ///
    /// Also the single place a run's outcome gets recorded into Stats -
    /// every run-ending path (Victory/GameOver's "press 1" handlers, and
    /// Pause's Q quit-early handler) calls this one function, so none of
    /// those three callers need to know anything about Stats themselves.
    fn return_to_title(&mut self) {
        // Read what play-history needs BEFORE wiping ecs/resources below -
        // both are gone the instant World::default()/Resources::default()
        // run. A run ended via Pause's early quit still has `outcome` at
        // whatever TurnState it was mid-run (AwaitingInput/Paused/etc,
        // never Victory) - that's correctly treated as "no win recorded"
        // below, while the deepest level reached still counts, since the
        // player genuinely got that far.
        let outcome = self.resources.get::<TurnState>().map(|t| *t);
        let player_info = <(&Player, &Class)>::query()
            .iter(&self.ecs)
            .next()
            .map(|(p, c)| (p.map_level, c.0.clone()));
        // Read before the wipe below for the same reason as outcome/
        // player_info above - both are gone the instant Resources::default()
        // runs. adventure_mode is a plain State field (not a resource), so
        // it survives the wipe on its own, but is read here anyway since
        // this fn resets it to DungeonCrawl a few lines down and the
        // Stats-recording code below needs the value as it was DURING the
        // run that's ending, not the post-reset default.
        let arena_run = self
            .resources
            .get::<Option<ArenaRun>>()
            .and_then(|run| *run);
        let mode = self.adventure_mode;

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
        self.resources.insert(None::<ShoppingActive>);
        self.resources.insert(None::<ArenaRun>);
        self.resources.insert(None::<ShopMessage>);
        self.resources.insert(Keymap::load());
        self.adventure_mode = AdventureMode::DungeonCrawl;

        let mut stats = Stats::load();
        if let Some((map_level, class)) = player_info {
            if outcome == Some(TurnState::Victory) {
                stats.record_win(&class, mode);
            }
            match mode {
                // Dungeon Crawl's own "how far did I get" - Player::map_level,
                // untouched by Arena code, so only meaningful here.
                AdventureMode::DungeonCrawl => {
                    stats.record_deepest_level(&class, map_level);
                }
                // Arena's own "how far did I get" - by wave, not level
                // alone, and "reached" rather than "cleared" (see
                // record_arena_progress). wave == 0 means the run ended
                // while still browsing a level's shop, before any wave of
                // THAT level was reached - that's already reflected by
                // whatever the previous level's last-reached wave was, so
                // it's deliberately not recorded as new progress here.
                AdventureMode::BattleArena => {
                    if let Some(run) = arena_run {
                        if run.wave >= 1 {
                            stats.record_arena_progress(&class, run.level, run.wave);
                        }
                    }
                }
            }
        }
        self.resources.insert(stats);

        self.background_move_timer_ms = 0.0;
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
        spawn_boss(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            map_builder.amulet_start,
        );
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
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
        ctx.set_active_console(MAP_SCROLL_CONSOLE);
        ctx.cls();
        ctx.set_active_console(ENTITY_SCROLL_CONSOLE);
        ctx.cls();
        ctx.set_active_console(HUD_CONSOLE);
        ctx.cls();
        ctx.set_active_console(BIG_TEXT_CONSOLE);
        ctx.cls();
        ctx.set_active_console(END_SCREEN_FALLEN_CONSOLE);
        ctx.cls();
        ctx.set_active_console(GLIDE_CONSOLE);
        ctx.cls();
        ctx.set_active_console(BATTLE_PORTRAIT_WIGGLE_CONSOLE);
        ctx.cls();
        self.resources.insert(ctx.key);
        self.resources.insert(FrameTime(ctx.frame_time_ms));
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
        ctx.set_active_console(HUD_CONSOLE);
        self.resources
            .insert(HudMousePos(Point::from_tuple(ctx.mouse_pos())));
        ctx.set_active_console(0);
        let current_state = self.resources.get::<TurnState>().unwrap().clone();
        match current_state {
            TurnState::TitleScreen => {
                self.title_screen(ctx);
            }
            TurnState::AdventureSelect => {
                self.adventure_select(ctx);
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
            TurnState::Options => {
                self.options_tick(ctx);
            }
            TurnState::StatsView => {
                self.stats_view_tick(ctx);
            }
            TurnState::InBattle => {
                self.battle_tick(ctx);
            }
            TurnState::BattleVictory => {
                self.battle_victory_tick(ctx);
            }
            TurnState::ArenaTransition => {
                self.arena_transition_tick(ctx);
            }
            TurnState::ArenaWaveCleared => {
                self.arena_wave_cleared_tick();
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
        // Raised from 30 to 60: with real elapsed-time-based animation
        // (MovingAnimation/FrameTime, not frame counts - see
        // systems/animation.rs), every timed effect in this project
        // already scales correctly at any frame rate. The tile-glide
        // specifically only gets ~4-5 frames total to play out at 30fps
        // for its default MOVE_ANIM_DURATION_MS, which reads as jumpy
        // rather than smooth - doubling the frame budget is the direct
        // fix for that, with no knock-on effect on anything's actual
        // speed (battle flashes, popups, background wandering, etc. all
        // still take exactly as long in real time as before).
        .with_fps_cap(60.0)
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
        // Console 4 (MAP_SCROLL_CONSOLE): a "fancy console" - supports
        // DrawBatch::set_fancy, unlike every plain "simple console" above.
        // Same DISPLAY_WIDTH x DISPLAY_HEIGHT grid/32x32px cells and
        // dungeonfont as console 0, WITH a background (unlike every other
        // fancy console below) so it can fully replace console 0 for a
        // frame without anything showing through around the edges of a
        // tile. Deliberately registered here, right after console 3 and
        // before HUD_CONSOLE, so it stays below the HUD in z-order the
        // same way console 0 always has - see MAP_SCROLL_CONSOLE's own
        // doc comment in the prelude module above for why that ordering
        // matters. Used only by map_render, only on frames where the
        // camera itself is panning.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 5 (ENTITY_SCROLL_CONSOLE): a second new "fancy
        // console", same grid as console 4 above, transparent background
        // (the same RGBA-alpha-0 trick GLIDE_CONSOLE uses below). Used
        // only by entity_render, on the same frames as MAP_SCROLL_CONSOLE
        // - see its own doc comment in the prelude module above.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(HUD_COLS, HUD_ROWS, "terminal8x8.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "terminal8x8.png")
        // Console 8 (END_SCREEN_FALLEN_CONSOLE): a "fancy console" -
        // supports DrawBatch::set_fancy (sub-pixel position + rotation +
        // scale), unlike every "simple console" above. Same square
        // 32x32px cells as console 0/1 (dungeonfont at native size on the
        // DISPLAY_WIDTH x DISPLAY_HEIGHT grid), so a 90-degree rotation
        // doesn't stretch/squash the glyph. Used only by
        // draw_end_screen_fallen_portrait for the GameOver screen.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 9 (GLIDE_CONSOLE): a second "fancy console", same grid
        // and cell size as console 8 above. Used by entity_render to draw
        // any entity mid-tile-glide at a sub-pixel position with a
        // genuinely transparent background, instead of console 1's
        // integer-snapped grid - see the GLIDE_CONSOLE doc comment.
        .with_fancy_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        // Console 10 (BATTLE_PORTRAIT_WIGGLE_CONSOLE): a third "fancy
        // console", sharing console 3's coarse BATTLE_PORTRAIT_COLS x
        // BATTLE_PORTRAIT_ROWS grid (so cells are automatically just as
        // big, no separate scale needed). Used by draw_battle_arena to
        // give the "Attacking" portrait a small sub-pixel shake - see the
        // BATTLE_PORTRAIT_WIGGLE_CONSOLE doc comment.
        .with_fancy_console(
            BATTLE_PORTRAIT_COLS,
            BATTLE_PORTRAIT_ROWS,
            "dungeonfont.png",
        )
        .with_vsync(false)
        .build()?;

    main_loop(context, State::new())
}
