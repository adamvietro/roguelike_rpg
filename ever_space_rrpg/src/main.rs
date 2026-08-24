#![warn(clippy::pedantic)]

mod battle;
mod camera;
mod components;
mod map;
mod map_builder;
mod render_helpers;
mod screens;
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
    pub use crate::render_helpers::*;
    pub use crate::spawner::*;
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

    /// Tears down the current run (if any) and returns to the title
    /// screen - called when the player dismisses the GameOver or Victory
    /// screen, instead of immediately starting a new run with whatever
    /// class they last had. ClassSelect (via start_game) is now the only
    /// place a run actually begins. spawn_title_background is defined in
    /// screens/title.rs (a descendant module), which is why it needed to
    /// be marked `pub` - see that file's comment on the same fn.
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
