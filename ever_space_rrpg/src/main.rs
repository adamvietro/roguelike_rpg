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

struct State {
    ecs: World,
    resources: Resources,
    input_systems: Schedule,
    player_systems: Schedule,
    monster_systems: Schedule,
}

impl State {
    fn new() -> Self {
        let mut ecs = World::default();
        let mut resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut ecs, map_builder.player_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut ecs, &mut rng, 0, &map_builder.monster_spawns);
        spawn_prefab_enemies(&mut ecs, &mut rng, 0, &map_builder.prefab_enemy_spawns);
        spawn_prefab_sword(&mut ecs, &mut rng, 0, map_builder.prefab_sword_spawn);
        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);
        resources.insert(None::<Battle>);
        resources.insert(None::<BattleVictory>);
        Self {
            ecs,
            resources,
            input_systems: build_input_scheduler(),
            player_systems: build_player_scheduler(),
            monster_systems: build_monster_scheduler(),
        }
    }

    fn reset_game_state(&mut self) {
        self.ecs = World::default();
        self.resources = Resources::default();
        let mut rng = RandomNumberGenerator::new();
        let mut map_builder = MapBuilder::new(&mut rng);
        spawn_player(&mut self.ecs, map_builder.player_start);
        let exit_idx = map_builder.map.point2d_to_index(map_builder.amulet_start);
        map_builder.map.tiles[exit_idx] = TileType::Exit;
        spawn_level(&mut self.ecs, &mut rng, 0, &map_builder.monster_spawns);
        spawn_prefab_enemies(&mut self.ecs, &mut rng, 0, &map_builder.prefab_enemy_spawns);
        spawn_prefab_sword(&mut self.ecs, &mut rng, 0, map_builder.prefab_sword_spawn);
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
        self.resources.insert(None::<BattleVictory>);
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
        spawn_prefab_sword(
            &mut self.ecs,
            &mut rng,
            map_level as usize,
            map_builder.prefab_sword_spawn,
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

        // --- Initiative: decided once at the start of each round, from
        // Speed. Garrote (if active) ticks first, before initiative is
        // even decided - it's a lingering wound, not an action. If the
        // enemy is faster, they attack immediately here - no menu shown -
        // before the player ever gets a choice this round.
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
            battle.first_actor = if player_speed >= enemy_speed {
                Combatant::Player
            } else {
                Combatant::Enemy
            };
            battle.awaiting_order_decision = false;

            if battle.first_actor == Combatant::Enemy {
                let attack_message = resolve_enemy_attack(&mut self.ecs, &mut battle);
                battle.message = match dot_message {
                    Some(d) => format!("{} Too fast to react! {}", d, attack_message),
                    None => format!("Too fast to react! {}", attack_message),
                };
                battle.turn = BattleTurn::FirstResult;
            } else {
                // Player is first_actor - PlayerMenu renders this same
                // tick, so surface the damage-over-time tick there instead.
                battle.message = dot_message.unwrap_or_default();
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

        match battle.turn {
            BattleTurn::PlayerMenu => {
                if !battle.message.is_empty() {
                    ctx.print_color_centered(45, YELLOW, BLACK, &battle.message);
                }

                let actions = available_actions(&self.ecs, battle.player);
                let menu_text: String = actions
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| match entry.count {
                        Some(n) => format!("{}) {} x{}", i + 1, entry.label, n),
                        None => format!("{}) {}", i + 1, entry.label),
                    })
                    .collect::<Vec<_>>()
                    .join("   ");
                ctx.print_color_centered(48, GREEN, BLACK, &menu_text);

                if let Some(key) = ctx.key {
                    let chosen = number_key_index(key)
                        .and_then(|i| actions.get(i))
                        .map(|entry| entry.action);
                    if let Some(chosen) = chosen {
                        // Every technique's mechanical effect is resolved
                        // in one place (battle::apply_player_technique)
                        // rather than a match arm per item here - adding a
                        // new class's technique needs no main.rs change.
                        match chosen {
                            BattleAction::Attack => {
                                let dmg = player_attack_damage(&self.ecs, battle.player);
                                apply_damage(&mut self.ecs, battle.enemy, dmg);
                                battle.player_flash =
                                    Some((FlashKind::Attacking, PORTRAIT_FLASH_DURATION_MS));
                                battle.enemy_flash =
                                    Some((FlashKind::Hit, PORTRAIT_FLASH_DURATION_MS));
                                battle.message = format!(
                                    "You hit the {} for {} damage!",
                                    battle.enemy_name, dmg
                                );
                            }
                            BattleAction::Defend => {
                                battle.player_defending = true;
                                battle.message = "You brace yourself to defend.".to_string();
                            }
                            BattleAction::Flee => {
                                battle.fled = true;
                                battle.message =
                                    format!("You flee from the {}!", battle.enemy_name);
                            }
                            BattleAction::Technique(item) => {
                                battle.message =
                                    apply_player_technique(&mut self.ecs, &mut battle, item);
                            }
                        }
                        // The player is first_actor at the start of a round
                        // they act in unprompted; if the enemy already
                        // opened the round (first_actor == Enemy), this
                        // menu is the player's second action instead.
                        battle.turn = if battle.first_actor == Combatant::Player {
                            BattleTurn::FirstResult
                        } else {
                            BattleTurn::SecondResult
                        };
                    }
                }
            }
            BattleTurn::FirstResult => {
                ctx.print_color_centered(48, WHITE, BLACK, &battle.message);
                ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");
                if ctx.key.is_some() {
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
                            battle.message = resolve_enemy_attack(&mut self.ecs, &mut battle);
                            battle.turn = BattleTurn::SecondResult;
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
                            battle.message.clear();
                        }
                    }
                }
            }
            BattleTurn::SecondResult => {
                ctx.print_color_centered(48, WHITE, BLACK, &battle.message);
                ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");
                if ctx.key.is_some() {
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
                    battle.message.clear();
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
        ctx.set_active_console(2);
        ctx.print_color_centered(2, RED, BLACK, "Your quest has ended.");
        ctx.print_color_centered(
            4,
            WHITE,
            BLACK,
            "Slain by a monster, your hero's journey has come to a premature end.",
        );
        ctx.print_color_centered(
            5,
            WHITE,
            BLACK,
            "The Amulet of Yala remains unclaimed, and your home town is not saved.",
        );
        ctx.print_color_centered(
            8,
            YELLOW,
            BLACK,
            "Don't worry, you can always try again with a new hero.",
        );
        ctx.print_color_centered(9, GREEN, BLACK, "Press 1 to play again.");

        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
        }
    }

    fn victory(&mut self, ctx: &mut BTerm) {
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
        ctx.print_color_centered(7, GREEN, BLACK, "Press 1 to play again.");
        if let Some(VirtualKeyCode::Key1) = ctx.key {
            self.reset_game_state();
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
        .with_vsync(false)
        .build()?;

    main_loop(context, State::new())
}
