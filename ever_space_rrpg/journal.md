# Getting Started
Right now I will be taking the code that I made while working through the Hands on Rust book. I created a new project with Claude and it has a full set of instructions.

```cargo build --release --target x86_64-pc-windows-gnu```
<br />

---

<br />

# RPG Battles
All eight as separate files. Where each one goes and what to do with it:

| File              | Path in your project          | Action           |
| ----------------- | ----------------------------- | ---------------- |
| `battle.rs`       | `src/battle.rs`               | **New** — add it |
| `turn_state.rs`   | `src/turn_state.rs`           | Replace          |
| `components.rs`   | `src/components.rs`           | Replace          |
| `mod.rs`          | `src/systems/mod.rs`          | Replace          |
| `player_input.rs` | `src/systems/player_input.rs` | Replace          |
| `chasing.rs`      | `src/systems/chasing.rs`      | Replace          |
| `random_move.rs`  | `src/systems/random_move.rs`  | Replace          |
| `main.rs`         | `src/main.rs`                 | Replace          |

One more thing to do manually that isn't a file to copy in: **delete `src/systems/combat.rs`** — it's dead code now that battles replace the old `WantsToAttack` path, and GitHub Desktop will flag it as removed once you delete it locally.
<br />

---

<br />

# Better Looking Battles
Before I redesign the battle screen, let me check `systems/entity_render.rs` to confirm the exact draw call convention used elsewhere in the project (so the portrait rendering matches).Good — this project actually uses `DrawBatch` (target console + set + submit) for all entity/glyph drawing, not raw `ctx.set()`. I'll use that same proven pattern for the portraits rather than guessing at an unproven API. Here's the plan:

## Design

**Battle actions as components** (in `battle.rs`): `CanAttack`, `CanDefend`, `CanFlee` marker components, plus a `BattleAction` enum and an `available_actions(ecs, entity)` function that builds the menu from whichever marker components an entity actually has. Player gets all three now; enemies get only `CanAttack` (matches "enemies will only attack"). When you add classes later, a Mage might get `CanAttack + CanFlee` but not `CanDefend`, or a future `CanCastSpell` — no menu code changes needed, just component assignment at spawn.

**Flee** — I read your message as "let's actually add it now that the framework supports it," so I added it: always succeeds, ends the battle, enemy survives. Say if you wanted it deferred instead.

**Visuals** — there's no portrait/sprite art in `resources/` (just the two tile fonts), so "image" here means: draw each creature's own existing glyph (the same character used on the dungeon map) scaled way up. I'm adding a 4th console with a coarse 8×5 grid over the same window (vs. the dungeon's 40×25), so each cell is 160×160px instead of 32×32 — a 5x scale-up — then drawing each creature as a 2×2 block of its own glyph/color in that grid. Enemy top-right, player bottom-left, per your ask. Text (name, HP bar, message, menu) stays on the existing fine-grained console, positioned around the portraits.

**HP bars** — added a simple `[####------]` text bar alongside the numbers for a more "detailed" feel.

Let's build it.Now let's rewrite `battle.rs` with the action-component system, flee support, and the render/HP-bar helpers:Now update `spawner/mod.rs` — the player gets all three capabilities:Let me reconsider that — `ecs.push()` takes a tuple, and I don't want to risk hitting a tuple-arity limit in legion by jumping from 6 elements to 9 in one push. I'll use the same push-then-`add_component`-via-`CommandBuffer` pattern that `spawner/template.rs` already proves works, instead:Now update `spawner/template.rs` — enemies get `CanAttack` (already uses the proven `add_component` pattern, so this is a trivial safe addition):Now the `main.rs` changes: a 4th console for portraits, clearing it each tick, and the full `battle_tick` rewrite with dynamic menu, flee, HP bars, and portraits.Now add the constants and clear the new console each tick:Now the full `battle_tick` rewrite. Let me replace the whole method:Let me check whether `ctx.print_color` (non-centered) is already used anywhere, to confirm the signature before relying on it:One more gap — I referenced `draw_portrait_block` but haven't defined it yet. Let me add it as a free function, using the exact `DrawBatch::set(Point, ColorPair, glyph)` pattern already proven in `entity_render.rs`/`map_render.rs`:Let's do a full final review of the whole `main.rs` to make sure everything is coherent:All four files, all **modified** (no new/deleted files this round):

| File          | Path                      | What changed                                                                                                                                                                 |
| ------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `battle.rs`   | `src/battle.rs`           | Added `CanAttack`/`CanDefend`/`CanFlee` marker components, `BattleAction` enum, `available_actions()`, Flee support, HP bar string, and a render-lookup helper for portraits |
| `mod.rs`      | `src/spawner/mod.rs`      | Player now spawns with `CanAttack`/`CanDefend`/`CanFlee`                                                                                                                     |
| `template.rs` | `src/spawner/template.rs` | Enemies now spawn with `CanAttack`                                                                                                                                           |
| `main.rs`     | `src/main.rs`             | New 4th console for portraits, rewritten `battle_tick` with dynamic menu + HP bars + portraits                                                                               |

**What to expect:** enemy glyph rendered big in the top-right, your glyph big in the bottom-left, name + `[####------]` HP bar next to each, and a menu that now reads `1) Attack   2) Defend   3) Flee` — built from your components at runtime rather than hardcoded, so a future class with different capabilities just needs different `CanXxx` components at spawn, no menu code changes.

**One thing to flag rather than bury:** I used `ctx.print_color(x, y, fg, bg, text)` for the name/HP-bar labels — the non-centered sibling of `ctx.print_color_centered`, which is already proven working in your build. I'm confident it exists (it's a standard, stable bracket-lib method), but it's genuinely the one line in this batch I haven't seen proven in *your* codebase specifically, unlike everything else here which mirrors patterns already compiling in your project. If `cargo build` complains about that line specifically, tell me the error and I'll swap it for the `DrawBatch`-based `print_color` your `hud.rs` already proves works — trivial fix either way.

Also: since exact pixel layout is something I can't visually preview from here, the portrait/text positions are my best numeric estimate (I worked through the coordinate math by hand) — expect it might need a round of "nudge this two rows down" type feedback once you see it running.

## Fixing Pattern Issues
Right now the images are just patterned so I took the time to change the code to better reflect the desired changes
```rust
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
    // than the dungeon view's 32px tiles (256x200px per cell here).
    pub const BATTLE_PORTRAIT_COLS: i32 = 5;
    pub const BATTLE_PORTRAIT_ROWS: i32 = 4;
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
fn draw_portrait(batch: &mut DrawBatch, col: i32, row: i32, render: Render) {
    batch.set(Point::new(col, row), render.color, render.glyph);
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
        resources.insert(map_builder.map);
        resources.insert(Camera::new(map_builder.player_start));
        resources.insert(TurnState::AwaitingInput);
        resources.insert(map_builder.theme);
        resources.insert(None::<Battle>);
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
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
        self.resources.insert(None::<Battle>);
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
        self.resources.insert(map_builder.map);
        self.resources.insert(Camera::new(map_builder.player_start));
        self.resources.insert(TurnState::AwaitingInput);
        self.resources.insert(map_builder.theme);
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

        let (enemy_hp, enemy_max) = entity_health(&self.ecs, battle.enemy);
        let (player_hp, player_max) = entity_health(&self.ecs, battle.player);

        // --- Portraits: each creature's own glyph, drawn once on the coarse
        // BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS console, so it renders
        // far larger than its normal dungeon-map size. Enemy sits top-right,
        // player sits bottom-left.
        let mut portraits = DrawBatch::new();
        portraits.target(3);
        if let Some(render) = entity_render_component(&self.ecs, battle.enemy) {
            draw_portrait(&mut portraits, 4, 0, render);
        }
        if let Some(render) = entity_render_component(&self.ecs, battle.player) {
            draw_portrait(&mut portraits, 0, 3, render);
        }
        portraits.submit(0).expect("Batch error");

        // --- Text: name + HP bar anchored next to each portrait, and a
        // message/menu panel centered in the gap between them.
        ctx.set_active_console(2);

        ctx.print_color(128, 26, YELLOW, BLACK, &battle.enemy_name);
        ctx.print_color(
            128,
            27,
            YELLOW,
            BLACK,
            &format!(
                "{} {}/{}",
                hp_bar_string(enemy_hp, enemy_max, 16),
                enemy_hp.max(0),
                enemy_max
            ),
        );

        ctx.print_color(1, 73, WHITE, BLACK, "You");
        ctx.print_color(
            1,
            74,
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
                let actions = available_actions(&self.ecs, battle.player);
                let menu_text: String = actions
                    .iter()
                    .enumerate()
                    .map(|(i, action)| format!("{}) {}", i + 1, action.label()))
                    .collect::<Vec<_>>()
                    .join("   ");
                ctx.print_color_centered(48, GREEN, BLACK, &menu_text);

                if let Some(key) = ctx.key {
                    if let Some(chosen) = number_key_index(key).and_then(|i| actions.get(i)) {
                        match chosen {
                            BattleAction::Attack => {
                                let dmg = entity_damage(&self.ecs, battle.player)
                                    + carried_weapon_damage(&self.ecs, battle.player);
                                apply_damage(&mut self.ecs, battle.enemy, dmg);
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
                        }
                        battle.turn = BattleTurn::PlayerActionResult;
                    }
                }
            }
            BattleTurn::PlayerActionResult => {
                ctx.print_color_centered(48, WHITE, BLACK, &battle.message);
                ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");
                if ctx.key.is_some() {
                    if battle.fled {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    let (enemy_hp_now, _) = entity_health(&self.ecs, battle.enemy);
                    if enemy_hp_now < 1 {
                        let mut cb = CommandBuffer::new(&mut self.ecs);
                        cb.remove(battle.enemy);
                        cb.flush(&mut self.ecs);
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::AwaitingInput);
                        return;
                    }

                    // Enemy's turn. Enemies only know Attack for now (see
                    // available_actions / CanAttack), so this always picks
                    // that - future enemy AI variety hooks in here.
                    let mut dmg = entity_damage(&self.ecs, battle.enemy);
                    if battle.player_defending && dmg > 0 {
                        dmg = (dmg / 2).max(1);
                    }
                    apply_damage(&mut self.ecs, battle.player, dmg);
                    battle.message = if battle.player_defending {
                        format!(
                            "The {} attacks - you block some of it! ({} damage)",
                            battle.enemy_name, dmg
                        )
                    } else {
                        format!("The {} attacks you for {} damage!", battle.enemy_name, dmg)
                    };
                    battle.player_defending = false;
                    battle.turn = BattleTurn::EnemyActionResult;
                }
            }
            BattleTurn::EnemyActionResult => {
                ctx.print_color_centered(48, WHITE, BLACK, &battle.message);
                ctx.print_color_centered(51, YELLOW, BLACK, "Press any key to continue.");
                if ctx.key.is_some() {
                    let (player_hp_now, _) = entity_health(&self.ecs, battle.player);
                    if player_hp_now < 1 {
                        self.resources.insert(None::<Battle>);
                        self.resources.insert(TurnState::GameOver);
                        return;
                    }

                    battle.turn = BattleTurn::PlayerMenu;
                    battle.message.clear();
                }
            }
        }

        self.resources.insert(Some(battle));
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
        self.resources.insert(ctx.key);
        ctx.set_active_console(0);
        self.resources.insert(Point::from_tuple(ctx.mouse_pos()));
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
        .with_title("Dungeon Crawler")
        .with_fps_cap(30.0)
        .with_dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT)
        .with_tile_dimensions(32, 32)
        .with_resource_path("resources/")
        .with_font("dungeonfont.png", 32, 32)
        .with_font("terminal8x8.png", 8, 8)
        .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
        .with_simple_console_no_bg(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "terminal8x8.png")
        .with_simple_console_no_bg(BATTLE_PORTRAIT_COLS, BATTLE_PORTRAIT_ROWS, "dungeonfont.png")
        .with_vsync(false)
        .build()?;

    main_loop(context, State::new())
}
```

## Crashes
This backtrace is the smoking gun — and it points to something important: the fix that's already "documented as done" in your project instructions was never actually taking effect.This backtrace is actually really useful, because it reveals something bigger than just this one crash: **the `WINIT_UNIX_BACKEND=x11` fix your project instructions describe as "already handled" was never actually working.**

Look at the very first warning in your output:

```
warning: unused manifest key: env
help: env is a valid .cargo/config.toml key
```

Cargo is telling you exactly what's wrong. Your `Cargo.toml` has:

```toml
[env]
WINIT_UNIX_BACKEND = "x11"
```

But `[env]` is **not a valid section in `Cargo.toml`** — it only works in a separate file, `.cargo/config.toml`. Cargo silently ignores it in the manifest (hence the warning), which means `WINIT_UNIX_BACKEND` has never actually been set by this mechanism, in this project, ever. There's no `.cargo/config.toml` file anywhere in the project — I checked.

That fully explains the crash you just hit: the backtrace goes through `winit::platform_impl::platform::wayland::window::Window::new` → `sctk_adwaita` → `crossfont` → `freetype`'s `bitmap.rs` panic — that's *exactly* the Wayland title-bar rendering crash your instructions doc already describes, still happening because X11 was never actually being forced. It also plausibly explains the original "random crashes" from earlier — without the env var reliably applied, which windowing path WSLg takes can vary, giving you different failure modes (this Wayland/freetype panic one run, the Mesa/Zink EGL failure another) that all trace back to the same root cause.

**The real fix:** move `[env]` into an actual `.cargo/config.toml`.Now clean up `Cargo.toml` — remove the dead `[env]` block so it doesn't keep confusing future you (or future me) into thinking this is already handled:**Two files:**

| File          | Path                 | Action                                                                                            |
| ------------- | -------------------- | ------------------------------------------------------------------------------------------------- |
| `config.toml` | `.cargo/config.toml` | **New** — create the `.cargo` folder in your project root (sibling to `src/`) and drop this in it |
| `Cargo.toml`  | `Cargo.toml`         | Replace — removed the dead `[env]` block that was never actually doing anything                   |

After that, run a plain `cargo run` (no `LIBGL_ALWAYS_SOFTWARE`, no `--release`) — that env var override should no longer be needed once X11 is actually being forced correctly, and it's worth ruling out first since it may have been contributing to the "opens but no window" behavior you just saw. Let me know if it crashes again and if so, paste the output the same way — but I'd bet this was the actual root cause of the "random" crashes all along, not the graphics driver.

Once you've confirmed it's stable, this is worth fixing in your project instructions doc too — the current "Known Environment Quirks" entry says this fix is "now set permanently via `.cargo/config.toml`," which was inaccurate (it was in `Cargo.toml` this whole time). Want me to write the corrected version of that section for you to paste in?

## Background for Battle Grid
Good — only `DungeonTheme` and `ForestTheme` implement it (`EmptyArchitect` is unrelated, it's a dead `MapArchitect`, not a theme). Let's add the palettes:Now let's add the arena background drawing to `battle_tick`. Let's look at its current start:Now add the `vignette` helper next to `draw_portrait`:Good, `bracket_lib::prelude::*` covers `RGB`, `ColorPair`, `DrawBatch`, `Point` — all consistent with what's already used elsewhere in the file. Let's do a final full read-through of the changed section to confirm it's coherent end to end:This all reads correctly and mirrors patterns already proven elsewhere in the file (the resource-borrow-then-drop pattern matches the earlier fix, `DrawBatch::target/set/submit` matches `map_render.rs` exactly). Let's ship it.All three **modified**:

| File        | Path                        | What changed                                                                                                                                                                                                    |
| ----------- | --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `mod.rs`    | `src/map_builder/mod.rs`    | `MapTheme` trait gains `floor_color()` and `wall_color()`                                                                                                                                                       |
| `themes.rs` | `src/map_builder/themes.rs` | `DungeonTheme` gets a cool slate-gray/warm-stone palette, `ForestTheme` gets a mossy-green/bark-brown palette                                                                                                   |
| `main.rs`   | `src/main.rs`               | New arena background drawn on console 0 during battle: current theme's floor tiles fill the space, a wall-tile border frames it, both tinted and pulled toward black at the edges via a distance-based vignette |

Since `MapTheme` is a trait with only those two implementors, this was a safe, contained change — no other code touches `tile_to_render` in a way that would break.

The background now automatically differs between a Dungeon-themed level and a Forest-themed level without any battle-specific logic knowing the difference — it just asks whatever theme is currently active. When you add a third theme down the line, it gets a battle background for free as soon as it implements those two color methods.

Give it a build and let me know how the vignette strength/border thickness reads — those are the two easiest knobs to tune (the `0.6` fade factor in `vignette()`, and the single-cell border check in the arena loop) if it looks too subtle or too strong.

### Almost There, Battle Arena
Good — I'll leave the Forest trees as they are then (the stepped canopy shape, tinted from the theme's floor color) since that satisfies either interpretation. Let's make the battle scenery theme-aware.Now implement it per theme:Now let's update `battle_tick` in `main.rs` to branch on this — trees for Forest, thick side walls for Dungeon:Let's do a final review to make sure everything's coherent:Consistent across all three files, and `BattleScenery` is exported through `map_builder`'s `pub use themes::*` / the trait living in `mod.rs`, so it reaches `main.rs` via the existing `pub use crate::map_builder::*;` in the prelude — same path everything else already uses.All three **modified**:

| File        | What changed                                                                                    |
| ----------- | ----------------------------------------------------------------------------------------------- |
| `mod.rs`    | New `BattleScenery` enum (`ScatteredTrees` / `RoomWalls`) + `MapTheme::battle_scenery()` method |
| `themes.rs` | `DungeonTheme` → `RoomWalls`, `ForestTheme` → `ScatteredTrees`                                  |
| `main.rs`   | Battle arena background now branches on the theme's scenery choice                              |

Dungeon battles get thick stone-colored walls running the full height down both sides (4 tiles deep, solid fill, full brightness — not dimmed by the vignette, since they're structural rather than lighting) instead of the tree silhouettes. Forest keeps exactly what it had.

The design is meant to scale cleanly: when you add a third theme later, it just picks one of the two existing `BattleScenery` variants (or we add a new one) and gets an appropriate arena for free — no battle-screen code changes needed, same pattern as `floor_color`/`wall_color`.

One thing worth watching for when you build it: the side walls are 4 tiles thick on a 40-wide grid, which is a meaningful chunk of the screen (20% total width) — I checked it doesn't overlap the portraits or their labels, but if it feels like it's crowding the play space too much once you see it, `SIDE_WALL_THICKNESS` in that `RoomWalls` branch is the one number to shrink.

## Adding a Speed Stat
This is a meaningful architecture change to the battle loop — right now it's hardcoded "player always acts first, then enemy." To support speed-based initiative, I need the battle round to be able to flip that order dynamically. Let me check `components.rs` first to place the new component consistently with the existing ones.Now give the player a Speed, matching the existing `CanAttack`/`CanDefend`/`CanFlee` pattern:Now add `speed` to the `Template` struct and wire it up for spawned enemies:Now update `template.ron` — speed values inverse to size/damage, per your ask:Now the harder part — reworking `battle.rs`'s turn structure to support either combatant going first:Now let me rewrite this with `Combatant`, the reworked `BattleTurn`, `Battle` gaining `first_actor`/`awaiting_order_decision`, plus `entity_speed` and `resolve_enemy_attack` helpers:Now the `battle_tick` rewrite in `main.rs`. Let's view the current version in full:Now I have the full picture. Let me rewrite the whole match block plus insert the round-initiative logic right after extracting `battle`:Now the main match block — this needs the biggest rework:Let me do a full read-through of the rewritten `battle_tick` to verify everything is syntactically and logically sound end-to-end:This is correct and reads cleanly. Let's verify the whole match block compiles logically one more time by viewing it in full:This all checks out correctly — I traced through both orderings (player-fast and enemy-fast) by hand and the target-checking logic is consistent. Let's ship the full set.All six **modified**:

| File                  | What changed                                                                                                                                                                                      |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `components.rs`       | New `Speed` component                                                                                                                                                                             |
| `battle.rs`           | New `Combatant` enum, reworked `BattleTurn` (symmetric — either side can go first), `Battle` gains `first_actor`/`awaiting_order_decision`, new `entity_speed`/`resolve_enemy_attack` helpers     |
| `spawner/mod.rs`      | Player spawns with `Speed(6)`                                                                                                                                                                     |
| `spawner/template.rs` | `Template` gets an optional `speed` field, applied to spawned enemies (defaults to 5 if a template omits it)                                                                                      |
| `template.ron`        | Speed values added: Goblin 9, Orc 7, Ogre 4, Ettin 2 — fastest-to-slowest as size/damage goes up                                                                                                  |
| `main.rs`             | `battle_tick` now decides initiative from Speed at the start of every round; the faster combatant acts first (enemy auto-attacks with no menu shown if they win it), then the slower one responds |

**How it plays out:** at 6, your player usually loses initiative to Goblins (9) and Orcs (7) — you'll see "Too fast to react!" and take a hit before you get a menu — but wins it against Ogres (4) and Ettins (2), so you get the first swing on the big slow ones. Ties go to the player. Defend still works exactly the same regardless of who goes first — it blocks whatever the next incoming hit is, whether that's later this round or the opening hit of the next one.

One thing to flag before you go looking for glyph ideas: **an Ettin already exists** in `template.ron` (glyph `E`, 10 HP, 3 damage, only appears on level 2) — it's just never spawned yet if you haven't reached that depth. So either you already knew that and want a *different* new enemy, or this was already what you had in mind and nothing else is needed there. What did you want to add — another Ettin-tier heavy hitter, something in between Orc and Ogre, or something else entirely? I'd rather get the name/stats from you than guess at a glyph blind, since I can't preview what's actually in `dungeonfont.png`.

## Battle Win Attacks
Good — that simplifies things a lot. Let me finalize the remaining details with sensible defaults (stating them so you can correct anything), then build. First let me check how item pickup currently transitions components, so the new "auto-granted" loot matches that pattern exactly.Good, `Point` is removed and `Carried` added on pickup — I'll construct battle loot items already in that "picked up" state (no `Point` at all), matching the same end result. Let's build this out.Now `template.ron` — 4 new items with `levels: []` so they never appear as floor spawns, only as battle loot:Now the core of the feature — `battle.rs`. Let me view the current state and rework it:Now let me rewrite this file with the new item system integrated:Now `spawner/template.rs` — wire up the new `provides` types and add the loot-granting function:Let me check whether `Templates` is actually exported from the `spawner` module — I need to call `grant_random_battle_loot` from `main.rs`.Good catch on my part to check — `Templates` isn't exposed outside `spawner/mod.rs`. I'll add a wrapper function, matching the exact pattern `spawn_level` already uses:Good, confirmed — `grant_random_battle_loot` will be reachable from `main.rs` via the prelude. Now the big one: `battle_tick` in `main.rs`.Let me check whether the existing HUD actually lists carried items by name, since I'm planning to grant loot silently (matching the existing instant-return-to-dungeon pattern) rather than adding a new "you found X!" screen:Now let's rewrite the pre-match block to add the Garrote tick, and update the `PlayerMenu` render to show `battle.message` (needed so a Garrote tick is visible even when you go straight to your menu):Now let's view and rewrite the full `match battle.turn` block — this needs the item menu entries, the message display in `PlayerMenu`, and loot-granting on the other two death paths:Let me check which color constants are already proven in this file before using `ORANGE`, which I haven't confirmed exists:That's just my own edit showing up — `ORANGE` isn't actually proven in code the user has successfully built. Let me swap it for `YELLOW`, which is already confirmed working elsewhere in this exact file:Now let's view the rest of the match block (`FirstResult` and `SecondResult`) to add loot-granting on the two remaining death paths:Now let's add loot-granting to both remaining enemy-death exit points:Good, all consistent. Let's do a final full read-through of the entire `battle_tick` method to verify it's coherent end-to-end:This all reads correctly. Let's ship the complete set of changes.All six **modified**. Here's the summary and every default I chose without a direct answer from you — flag anything you want changed:

| File                  | What changed                                                                                                                                                                                                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `components.rs`       | 4 new marker components: `ProvidesDeathblow`, `ProvidesQuickAttack`, `ProvidesCounterAttack`, `ProvidesGarrote`                                                                                                                                                                             |
| `battle.rs`           | `BattleAction` gains the 4 item variants; `available_actions` now returns `(action, Option<count>)` pairs; new `carried_*` lookup helpers; `resolve_enemy_attack` now rolls Counter Attack; new `tick_garrote`; two tunable constants (`GARROTE_DAMAGE = 2`, `COUNTER_CHANCE_PERCENT = 65`) |
| `spawner/mod.rs`      | New `grant_random_battle_loot` wrapper                                                                                                                                                                                                                                                      |
| `spawner/template.rs` | `provides` handling extended for the 4 new types; `apply_provides` factored out so floor-spawns and battle-loot share the same logic; new `Templates::grant_random_battle_loot`                                                                                                             |
| `template.ron`        | 4 new item templates, `levels: []` so they're never floor-spawned — only reachable via battle loot                                                                                                                                                                                          |
| `main.rs`             | Garrote ticks once per round before initiative; items appear in the battle menu with remaining counts and are each consumed on use; loot is granted (silently, into inventory — shows up in your existing "Items carried" HUD list) at all three points an enemy can die                    |

**Defaults I picked (you only answered the Quick Attack question):**
- Loot goes **straight into inventory**, no ground pickup.
- **40% chance** per victory, uniformly random among the 4 items, `BATTLE_LOOT_DROP_CHANCE_PERCENT` — one constant to tune.
- Items **stack** — hold as many of each as you find, each single-use.
- Menu is **flattened** into one list (`4) Deathblow x2`, etc.) — up to 7 options total when you're carrying everything.
- Counter Attack: **you still take the enemy's normal damage**, and separately deal 3x back if the 65% roll hits.
- Garrote: **consumes your action** that round, ticks start the *following* round, and re-using it while active **refreshes to a fresh 3 turns** rather than stacking.
- Quick Attack: implemented as the simple version we agreed on — both hits land as one combined message, then the round proceeds normally (enemy retaliates or round ends) rather than showing two separate "press any key" screens.

**Worth testing specifically once you build it:** the PlayerMenu can now show up to 7 options in one line, plus a status line above it for Garrote ticks — I can't preview how that reads at the actual font size, so that's the first thing I'd look at.

### Classes for Attacks
Confirmed — let's build it.Now `battle.rs` — add `entity_class`, and update the `carried_*` helpers and `available_actions` to check class:Now `spawner/template.rs` — add the `class` field, attach it to spawned entities, and filter loot by class:Now `template.ron` — tag the 4 battle items as Barbarian:Confirmed safe — the existing `template.ron` already relies on missing `Option<T>` fields defaulting to `None` (e.g. `Goblin` has no `provides`), so the other 8 templates don't need touching. Now `spawner/mod.rs`:Now update the 4 call sites in `main.rs`'s `battle_tick` to pass the player's class:Let me add a `player_class` lookup right where `actions` is computed, then update all 4 call sites:Now update the 4 call sites:Let's verify there are no remaining stale call sites and do a final consistency check:Everything is consistent — 4 helper functions each now take the class parameter, all call sites updated. Let's ship the complete set.All six **modified**:

| File                  | What changed                                                                                                                                                                                                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `components.rs`       | New `Class(pub String)` component                                                                                                                                                                             |
| `battle.rs`           | New `entity_class`/`item_usable_by_class` helpers; the 4 `carried_*` functions now take a class and filter to only usable items; `available_actions` looks up the entity's class once and passes it through   |
| `spawner/mod.rs`      | Player spawns with `Class("Barbarian")`; `grant_random_battle_loot` now looks up the player's class first and bails (returns `None`) if they somehow have none                                                |
| `spawner/template.rs` | `Template` gets an optional `class` field; `apply_class` helper tags spawned/granted entities with it; loot rolling now filters candidates to `template.class == player's class` instead of a fixed name list |
| `template.ron`        | The 4 battle items tagged `class: Some("Barbarian")`; nothing else needed changes since missing `Option` fields already default to `None`                                                                     |
| `main.rs`             | `player_class` computed once per `PlayerMenu` render, threaded into the 4 item-selection branches                                                                                                             |

**How this scales to a Mage later:** add a template with `class: Some("Mage")`, give the Mage player entity `Class("Mage")` instead of `"Barbarian"` — that's it. The menu, the consumption logic, and the loot roll all already read from the template/entity data rather than any hardcoded list, so no code changes needed for a second class, just content in `template.ron` plus wherever you decide class gets assigned to the player (currently hardcoded at spawn — you'll want a real character-selection step before this matters for real, but the plumbing's in place).

One thing worth testing once you build it: since loot is now filtered to class, and the *only* class that exists is Barbarian, every loot roll should always succeed in finding a candidate — but it's worth confirming nothing regressed for a hypothetical player with no `Class` at all (shouldn't happen given `spawn_player` always sets one, but `grant_random_battle_loot`'s early return via `?` on a missing class is the safety net there).

### End Battle Screen, Item UI
This all makes sense — two clear, separable pieces:

1. **HUD split**: a `BattleItem` marker tag on the 4 battle-attack items, so the left "Items carried" list can exclude them and a new right-side panel can show just them, grouped with counts.
2. **`BattleVictory` state**: a new `TurnState` that shows a dismissible screen ("You defeated the Goblin! You found: Deathblow!" or "No loot this time.") instead of silently dropping back into the dungeon — routed through on all three places an enemy can currently die in battle.

Let me confirm the current file states before editing, then build both.Now `components.rs` — the new tag:Now `spawner/template.rs` — tag the 4 battle items with `BattleItem`:Now `hud.rs` — split the item list:Now `battle.rs` — add the `BattleVictory` resource struct:Now `main.rs` — resource init, dispatch arm, the new `battle_victory_tick` method, and updating all 3 death-exit points:Now the dispatch arm:Now let's view the 3 death-exit points precisely and rework each, then add the `battle_victory_tick` method right before `game_over`:Now let's add the `battle_victory_tick` method, right before `game_over`:All three exit points consistently updated, and the dispatch arm is wired up. Let's ship the complete set.All six **modified**:

| File                  | What changed                                                                                                                                                                           |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `turn_state.rs`       | New `TurnState::BattleVictory`                                                                                                                                                         |
| `components.rs`       | New `BattleItem` marker component                                                                                                                                                      |
| `battle.rs`           | New `BattleVictory` resource struct (`enemy_name` + `loot: Option<String>`)                                                                                                            |
| `spawner/template.rs` | The 4 battle attacks now also get tagged `BattleItem` when spawned/granted                                                                                                             |
| `systems/hud.rs`      | Left list now excludes `BattleItem`s; new right-side "Battle Attacks" panel groups them by name with counts (e.g. `Deathblow x2`)                                                      |
| `main.rs`             | Resource init for `BattleVictory`, dispatch arm, new `battle_victory_tick` screen, and all 3 enemy-death exit points now route through it instead of silently returning to the dungeon |

**How it plays**: win a battle, and instead of instantly landing back in the dungeon, you get a dismissible screen — "You defeated the Goblin! You found: Deathblow!" or "No loot this time." — then a keypress drops you into `AwaitingInput` as before. While exploring, your regular items still list top-left as always, and now a "Battle Attacks" panel lists top-right with counts, live-updating as you use or gain them.

One design note worth flagging: I grouped the right-side panel by *name* with a count rather than listing every individual item entity — this matches how the battle menu already shows `Deathblow x2`, so the two displays stay visually consistent. If you'd rather see them itemized some other way, that's a quick change in the new block in `hud.rs`.

## Item Bug
Right now we are dealing with an issue where the item we have equipped is on the left side and all the items that we have removed from the left side HUD are still being help in a position so when you try to use an item the buttons associated is not correct on screen.
<br />

Let's tackle these one at a time, starting with the numbering bug since it's an actual bug. Let me look at the item-use key handling:Let me also check `tooltips.rs`, since "all the text for the map" likely includes hover tooltips too:Let's tackle this in three passes: first the numbering bug (shared source-of-truth for the item list), then the weapon relocation, then the 1.5x HUD scale. Starting with the shared helper:Now fix `player_input.rs` — the actual bug (the query was including `BattleItem` entities, which shifts every subsequent number's mapping):Now let's move weapons to the right side and switch the left list to the shared helper. I'll do this together with the 1.5× text-size change since both touch `hud.rs` heavily — let's set up the new console first.Now let's find the `BTermBuilder` chain and console-clearing logic in `main()`/`tick()`:Now clear console 4 each tick:Now the full `hud.rs` rewrite — target console 4, use `HUD_COLS`, use the shared `usable_carried_items` helper, and add the weapons section on the right:Now `tooltips.rs` — this one needs care, since its `mouse_pos * 4` shortcut was specific to the old console2's exact 4× relationship to console0. I'll replace it with a proper ratio-based conversion that works for any console size:Let's do a final consistency check across all touched files:Everything is consistent — both consumers use the shared helper, both `hud.rs` and `tooltips.rs` target the new console 4, and `HUD_COLS`/`HUD_ROWS` are used uniformly. Let's ship the complete set.All five **modified**. Here's what fixed each of your three points:

**1. The numbering bug** — root cause was that `use_item` (the thing that actually fires when you press a number key) queried *every* `Carried + Item` entity, while the HUD's displayed list had already been filtered to exclude battle items. Two different lists, two different orderings, same number key. Fixed by pulling both into one function — `usable_carried_items()` in `components.rs` — that excludes both `BattleItem` and `Weapon` entities, and having *both* `hud.rs` (what you see) and `player_input.rs` (what pressing a number does) read from it. They literally cannot drift apart now since they're the same call.

**2. Weapons moved off the left list** — same helper handles this for free, since it already excludes `Weapon`. They now get their own grouped "Weapons" panel on the right, stacked below "Battle Attacks," styled the same way (`Rusty Sword x1`).

**3. 1.5× bigger HUD text** — added a 5th console (`HUD_COLS=107 × HUD_ROWS=67`, ~12px cells vs the old 8px) and moved `hud.rs` and `tooltips.rs` onto it, leaving console 2 untouched so the battle screen, game-over, and victory screens are unaffected — you'd said "for the map," so I read that as exploration-view text specifically, not battle UI. One thing worth knowing: I couldn't hit an exact clean 1.5× *and* keep integer cell counts (1280×800 doesn't factor that way), so it's landed on ~1.494× — visually indistinguishable from 1.5×, but flagging the inexactness. The mouse-tooltip position math also had to change from a hardcoded `*4` shortcut (which only worked because the old console was exactly 4× finer than the dungeon grid) to a proper ratio-based conversion, since that relationship no longer holds with the new console size.

## Counter-Attack Bug
Good catch — the bug is exactly what you're describing. `resolve_enemy_attack` can now kill the enemy as a side effect (via Counter Attack), but the code paths right after it only ever check the *expected* target's health (the player's), never the enemy's. Let me find and fix both spots where this can happen.Now I have exact context for both fixes. First, `FirstResult`'s `Combatant::Enemy` branch — this processes the result of the enemy's *opening* attack (when they're faster), and needs to also check if Counter Attack killed them:Now the `SecondResult` block — this is likely where you're actually hitting it (player picks Counter Attack, enemy retaliates via `resolve_enemy_attack`, counter kills them mid-call, but `target_died` for `Combatant::Player` only ever checks the player's HP):Now let me rewrite this whole block to check both combatants' health unconditionally, rather than only checking whichever one the `first_actor` framing "expects" to have been hit:Let's do a final check to make sure no other spot has this same asymmetric-check pattern:Both call sites (line 376 and 580) are now followed by their corresponding continue-handlers checking *both* combatants' health (lines 587+599, and 638+639), instead of only the one side the old logic assumed was the target. Let's ship it.**`src/main.rs`** — modified. Two spots fixed:

1. **`FirstResult`'s `Combatant::Enemy` branch** (handles the enemy's opening attack when they're faster) — now also checks enemy HP before routing to the player's menu, in case a previously-armed Counter Attack killed them during their own opening swing.
2. **`SecondResult`** — this was the more likely place you were actually hitting it. Replaced the old logic (which derived "the" target from `first_actor` and only ever checked *that* side) with an unconditional check of both player and enemy HP. A counter can kill the enemy as a side effect of *any* enemy attack, regardless of which combatant "should" have been the one taking damage that exchange — so now both get checked every time, and whichever one is actually dead ends the battle immediately instead of waiting for the next normal attack to notice.

Root cause was the code reasoning about "who this attack's target is" instead of just checking "is anyone dead now" — the second is strictly correct and covers this case (and any similar future one) automatically.

## Battle Item HUD
Good, that's the design. Let's build it.Now `spawner/template.rs` — add the `description` field and apply it:Now `template.ron` — add descriptions to the 4 battle items:Now the core piece — `hud.rs`. Let's view it and add hover-detection:Let's verify the resource pattern matches what's already proven in `tooltips.rs`, and do a final consistency sweep:One cleanup — `tooltips.rs` still has its own inline copy of the conversion formula I extracted into `mouse_to_hud`. Let me make it use the shared function for consistency:Let's ship the complete set.All five **modified**:

| File                  | What changed                                                                                                                                                                                                          |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `components.rs`       | New `Description(String)` component; new `mouse_to_hud()` shared helper                                                                                                                                               |
| `spawner/template.rs` | `description` field on `Template`; `apply_description` helper wired into both spawn paths                                                                                                                             |
| `template.ron`        | Descriptions added to the 4 battle items                                                                                                                                                                              |
| `systems/hud.rs`      | Now takes `mouse_pos` as a resource; Battle Attacks entries track a representative entity per group; hover-detection against the mouse's HUD-console position; tooltip printed to the left of the panel when hovering |
| `systems/tooltips.rs` | Refactored to use the shared `mouse_to_hud()` instead of its own copy of the formula                                                                                                                                  |

**How the hover-detection works:** each Battle Attacks row remembers which entity it represents. Every frame, the mouse position gets converted into the HUD console's coordinate space (same conversion tooltips.rs already used), and if it lands on a row *and* is generally over the right-side panel (rightmost ~30 columns, so it doesn't fire when you're hovering the left item list on the same row number), that entity's `Description` gets printed starting at column 25.

**Things I genuinely can't verify without seeing it render, flagged honestly:**
- Column 25 for the tooltip text was a judgment call to clear both the left item list and the right panel — if your longest carried item name pushes past there, or a description is long enough to run into the panel's own text on the right, that's the first thing to nudge.
- The hover hit-test is row-based with a loose X threshold rather than pixel-precise against the actual right-aligned text width — deliberately forgiving (small text targets are hard to hit exactly), but if it ends up feeling too generous or not generous enough, `HUD_COLS - 30` is the one number to adjust.

### No Tooltip Bug
That's a real bug, and I can see why: I converted a *coarse* mouse position into the finer HUD grid, which loses precision the wrong way. Let me trace through why.Confirmed — `mouse_pos` is captured while console 0 (32px cells) is active, so it's already quantized to a coarse ~40×25 grid before my conversion ever runs. Scaling that up to console 4's 107×67 grid doesn't recover precision — it just spreads those same ~40×25 possible values across a wider range, and most of the tightly-packed rows where the item list actually lives (rows 3–10) fall between the achievable values and are never hit. The real fix is to capture a second, dedicated mouse position while console 4 is actually active, so it's precise in that grid from the start.Now capture it in `tick()`:Now update `hud.rs` to use this precise position instead of the lossy conversion:Everything lines up correctly — `hud.rs` now gets a precisely-captured position, `tooltips.rs` correctly keeps using the coarse console0-aligned one for its dungeon-tile hover check (which is the right choice there, since entities live in that same coarse grid). Let's ship it.All three **modified**:

| File             | What changed                                                                                                                          |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `components.rs`  | New `HudMousePos(Point)` resource — a mouse position captured while console 4 is actually active, precise in that grid from the start |
| `main.rs`        | `tick()` now switches to console 4 and captures a second `ctx.mouse_pos()` reading there, before restoring console 0                  |
| `systems/hud.rs` | Uses `HudMousePos` directly for the row hover-test instead of the lossy scaled-up conversion                                          |

**Root cause, plainly:** `ctx.mouse_pos()` returns coordinates relative to whichever console is active *at the moment you call it* — that's a bracket-terminal behavior, not something obvious from the outside. The original `mouse_pos` resource was captured with console 0 active (32px cells, ~40×25 grid), so it was already limited to roughly 40×25 distinct positions before I ever touched it. My earlier fix scaled those values up into the 107×67 HUD grid, but scaling up a value that's already been rounded down doesn't get the precision back — it just spreads ~40×25 achievable points across a much bigger grid, and the tightly-packed rows where your item list actually sits fell in the gaps between them. The real fix was capturing the mouse position fresh, with the *correct* console active, rather than trying to convert after the fact.

--- 

# Battle Feedback

## Battle Portrait Hit Feedback

Started as "just a jiggle" for attacks in the battle screen — turned into a good lesson on how the battle portraits actually render, and why a positional shake doesn't fit that setup.

### Why a jiggle didn't work

Portraits aren't sprite art — they're the same tiny glyph used on the dungeon map, drawn on a deliberately coarse 5x5-cell console (`console 3`, `BATTLE_PORTRAIT_COLS`/`BATTLE_PORTRAIT_ROWS`) that shares the same 1280x800 window as everything else. Dividing that window into only 5 columns makes each cell huge (256x160px), and bracket-terminal stretches the glyph to fill it — that's the entire trick behind the "big portrait" look, no scaling code involved.

That coarseness is exactly what broke the jiggle:

- **A "1 cell" shake is enormous.** On a 5-cell-wide grid, one cell is ~1/5 of the screen — nowhere near subtle, confirmed by measuring it directly on screen.
- **Tiling doesn't scale an image, it repeats it.** Tried switching to a fine 40x25 grid (32px cells, matching the dungeon view) and drawing each portrait as a tiled block of cells to get finer shake resolution — produced a repeated grid of tiny sprites instead of one big one, since tiling only reconstructs a single image if the source art is itself split into matching fragments. Ours is one repeated icon, so a pattern is the only possible result. Reverted.
- **Sub-pixel movement (bracket-lib's "fancy console") was the book's own suggested technique** for smooth positioning — genuinely the right tool for the underlying dungeon-tile movement work, but wrong here for a different reason: our portrait would be a static image sliding over a static background, which reveals a moving box edge as soon as it's not perfectly aligned — no amount of background color-matching fixes a box that's in motion.

### What we shipped instead: a color flash

No repositioning at all — just a brief foreground color swap on the portrait that already works, confined to the console we know is safe (console 3, plain `no_bg`, no box risk since background was never rendered there in the first place).

- `Battle` gained `enemy_flash`/`player_flash: Option<(FlashKind, f32)>`, replacing the old shake-timer fields. `FlashKind` is `Attacking` (white) or `Hit` (red).
- Every damage-dealing path sets both sides at once: whoever swung gets `Attacking`, whoever got hit gets `Hit`. Covers Attack, Deathblow, Quick Attack, Garrote's tick, the enemy's own attack, and a landed Counter Attack.
- `flash_tint()` (`main.rs`) swaps in the flash color for the portrait's foreground while its timer is running, then falls back to the normal sprite color once it expires — ticked down each frame in `battle_tick` using `ctx.frame_time_ms`, same pattern as everything else time-based in this file.

| File        | What changed                                                                                                                                                                                                            |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `battle.rs` | New `FlashKind` enum; `Battle::enemy_flash`/`player_flash` replace the old shake-timer fields; all damage-applying functions (`resolve_enemy_attack`, `tick_garrote`) set the appropriate flash                         |
| `main.rs`   | New `flash_tint()` helper (replaces `shake_offset`); `draw_battle_arena` takes flash state instead of shake amounts; per-frame flash countdown in `battle_tick`; the three player-attack menu branches set both flashes |

**Known limitation, not yet addressed:** the flash is a hard on/off, not a fade — pops to white/red instantly, then snaps back to normal exactly at 150ms. If it reads as too abrupt once we've looked at it more, the next step is fading the tint proportionally to remaining time instead of a flat swap.

# Fortress Sword Placement
Good catch — let's look at that more carefully before I fix anything.Confirmed — the 'M' points get pushed into the exact same `monster_spawns` list that feeds the general weighted lottery in `spawn_entities`, which picks *any* template (enemy or item) at each point. So a fortress guard slot has no more guarantee of actually being a monster than any other random point on the level — that's the bug. Let me fix that properly while adding the sword.

**Plan:**
1. Give guaranteed fortress spawns their own dedicated path — no more mixing into the general lottery.
2. Add a `'S'` marker to the fortress layout for the sword, guarded in the center by the existing 3 `'M'`s.
3. Add a `fortress_only` flag on templates so swords stop appearing in the ambient pool entirely, and pick which sword tier appears based on the dungeon level (reusing the `levels` field templates already have).

Let's build it.Now update all four architect files to initialize the new fields:Now the fortress prefab itself — add the `'S'` marker and route `'M'`/`'S'` into the new dedicated fields instead of the general pool:Now `spawner/template.rs` — add the `fortress_only` flag, exclude it from the ambient pool, and add the two dedicated guaranteed-spawn functions:Now add the two new guaranteed-spawn methods, right after `spawn_entities`:Now `spawner/mod.rs` — add thin wrapper functions matching the `spawn_level` pattern:Now `resources/template.ron` — mark the three swords `fortress_only: true`:Now let's find and update the three `spawn_level` call sites in `main.rs`:Now updating all three call sites:Let me verify the fortress layout dimensions are still exactly correct after editing:Confirmed correct — 11 rows of exactly 12 characters, sword sitting in the exact center between the three guards. Now let's do a final consistency sweep across every changed file:Everything wires up consistently across all 4 architects, the prefab, the spawner, and all 3 `main.rs` call sites. Let's pull all the changed files for delivery.All 10 files above. **Note on filenames:** two of these had to be renamed since your project has same-named files in different folders — save them back to their real paths:

| Downloaded as        | Save to                                     |
| -------------------- | ------------------------------------------- |
| `map_builder_mod.rs` | `src/map_builder/mod.rs` (**replace**)      |
| `spawner_mod.rs`     | `src/spawner/mod.rs` (**replace**)          |
| `rooms.rs`           | `src/map_builder/rooms.rs` (**replace**)    |
| `automata.rs`        | `src/map_builder/automata.rs` (**replace**) |
| `drunkard.rs`        | `src/map_builder/drunkard.rs` (**replace**) |
| `empty.rs`           | `src/map_builder/empty.rs` (**replace**)    |
| `prefab.rs`          | `src/map_builder/prefab.rs` (**replace**)   |
| `template.rs`        | `src/spawner/template.rs` (**replace**)     |
| `template.ron`       | `resources/template.ron` (**replace**)      |
| `main.rs`            | `src/main.rs` (**replace**)                 |

**Summary of the change:**
- Fixed the actual bug you spotted: fortress guard markers (`'M'`) no longer feed into the general weighted spawn lottery (where they could just as easily become an item). They now go through a new `spawn_fortress_enemies` path that only ever picks from Enemy-type templates.
- Added a `'S'` marker to the existing fortress layout, placed dead-center between the three guards. A new `spawn_fortress_sword` function places a guaranteed sword there.
- Swords (`Rusty`/`Shiny`/`Huge Sword`) got a new `fortress_only: true` flag in `template.ron` and are excluded from the ambient pool in `spawn_entities` — they genuinely can't spawn anywhere except a fortress now.
- Which sword tier can appear still reuses each sword's existing `levels` field (`Huge Sword` stays `[1, 2]`, so it won't show up on level 0) — no new level-scaling mechanism needed, the data was already there.

One thing worth knowing: fortress placement can still fail (up to 10 attempts, per the existing logic) if no valid spot is found — per your "let's get this working now" steer, I left that as-is rather than adding a placement guarantee, so it's possible (if uncommon) for a level to generate with no sword available at all. Easy to revisit later if that turns out to matter in practice.

# Defense Stats
I was able to add in a defense stat by adding a new component and then going to the battle.rs and adding a check to the damage that looks for a defense stat and if it is on the entity it will remove that amount of damage from the attack.


# New Classes
Now let me see the full main.rs, particularly the middle section covering `battle_tick` in its current (post-journal) form, plus the rest.Good, that's every site — three files to change (`components.rs`, `battle.rs`, `spawner/template.rs`), plus `main.rs` and `resources/template.ron`. Here's the design before I write it:

**Replace the "one enum variant + one marker component + one lookup function per technique" pattern with a single data-driven `TechniqueEffect` enum.** Instead of `ProvidesDeathblow`/`ProvidesQuickAttack`/`ProvidesCounterAttack`/`ProvidesGarrote` as four separate marker components with four near-identical `carried_*` functions and a hardcoded match arm per action in `main.rs`, there's one `Technique(TechniqueEffect)` component where `TechniqueEffect` is an enum describing *what the item does* (`DamageMultiplier(2)`, `MultiHit(2)`, `Counter{chance,mult}`, `DamageOverTime{damage,turns}`, plus a new `Heal{amount}` for later). `template.ron` picks the variant per item — no Rust code changes needed to add a new technique that reuses an existing effect shape.

This means: adding a Mage's Fireball that also multiplies damage is a **content-only** change (`technique: Some(DamageMultiplier(3))` in `template.ron`, tagged `class: Some("Mage")`). A genuinely new mechanic (e.g. a shield) needs one new enum variant handled once in `battle.rs`, not a new component + a new match arm scattered across `main.rs`.

Let me build this out.Now the `battle.rs` rewrite — this is where most of the hardcoding lived.Now replace the four `carried_*` lookup functions with one generic grouping function, plus small helpers `technique_effect` and `entity_name`:Now update the `Battle` struct's `countering`/`garrote_turns_remaining` fields to be data-driven, and remove the hardcoded constants:Now update `resolve_enemy_attack` to use `CounterState`, replace `tick_garrote` with a generic `tick_dot`, and add `apply_player_technique` + `heal_entity`:This looks correct. Now the `Templates` struct in `spawner/template.rs`:Now simplify `apply_provides` and add `apply_technique`, wiring it into both spawn paths:Now wire `apply_technique` into both call sites (`spawn_entity` and `grant_random_battle_loot`):Now `resources/template.ron` — swap the four Barbarian items' `provides:` for `technique:`:Now the `main.rs` `battle_tick` — replace the giant per-action match with the simplified generic version, and rename `tick_garrote` → `tick_dot`:Now the big one — the `PlayerMenu` match block:Now let's clean up the dead `src/systems/combat.rs` file, and double-check for any remaining stale references across the whole project:Clean — the only remaining `player_class` references are legitimate generic parameter names in the loot-filtering code (not hardcoded to Barbarian). Let me do a final full read-through of every touched file to check consistency before delivering.Now let's view the full modified `battle.rs` middle section (the `Battle` struct) and `main.rs` battle_tick end-to-end once more to make sure everything is coherent:Good — logic flow unchanged, just renamed. Let's view the rest of `battle_tick` to confirm the `PlayerMenu` edit landed cleanly and nothing downstream references the old fields:This is clean and correct. Everything downstream (`FirstResult`/`SecondResult`) only reads `battle.message`/`battle.fled`/health, which are unaffected by this refactor. Let's confirm nothing else in the file references removed items, and do a final full-project grep sanity check:Everything is consistent and `combat.rs` is gone with no dangling module reference. Now let's package the four changed files plus the deletion note for delivery.Yes, this makes sense — that's a solid four-part session. I've done part 1 (the refactor) now, since it's the foundation everything else builds on. Here's what changed and what's next.

## What I did: class refactor

**File delivery — save these to their real paths:**

| Downloaded as         | Save to                   | Action                                                  |
| --------------------- | ------------------------- | ------------------------------------------------------- |
| `battle.rs`           | `src/battle.rs`           | **Replace**                                             |
| `components.rs`       | `src/components.rs`       | **Replace**                                             |
| `spawner_template.rs` | `src/spawner/template.rs` | **Replace** (renamed to avoid collision with `main.rs`) |
| `main.rs`             | `src/main.rs`             | **Replace**                                             |
| `template.ron`        | `resources/template.ron`  | **Replace**                                             |

**Also delete:** `src/systems/combat.rs` — confirmed genuinely dead (not even declared in `systems/mod.rs`, so it wasn't compiling), left over from a session that told you to delete it but it never got removed.

**The core change:** the four Barbarian-specific pieces (`ProvidesDeathblow`/`QuickAttack`/`CounterAttack`/`Garrote` marker components → four near-identical `carried_*` lookup functions → four `BattleAction` enum variants → four hardcoded match arms in `main.rs`) collapse into **one data-driven system**:

- One component, `Technique(TechniqueEffect)`, where `TechniqueEffect` is an enum describing the *shape* of an effect: `DamageMultiplier(n)`, `MultiHit(n)`, `Counter{chance,mult}`, `DamageOverTime{damage,turns}`, plus a new `Heal{amount}` that nothing uses yet but is ready for a Mage spell.
- `template.ron` picks the variant per item (`technique: Some(DamageMultiplier(2))` for Deathblow, etc.) — the mechanical difference between techniques is now content, not Rust types.
- `main.rs`'s `battle_tick` has exactly one match arm for all techniques (`BattleAction::Technique(item) => apply_player_technique(...)`) instead of one per item.
- `Battle.countering`/`garrote_turns_remaining` became `Option<CounterState>`/`Option<DotState>`, carrying their own chance/multiplier/damage/turns instead of hardcoded constants.

**What this buys you for the Mage stage:** a Mage spell that reuses an existing shape (e.g. a damage-multiplying Fireball) is a `template.ron` entry only — zero Rust changes. A genuinely new mechanic (shield, mana cost, AoE) needs one new `TechniqueEffect` variant and one match arm in `apply_player_technique` — not a new component, enum variant, lookup function, *and* `main.rs` block like before.

**One wording note to flag:** Counter Attack's "ready" message is now generated from the item's name (`"You ready a {name.to_lowercase()}..."` → "You ready a counter attack..."), a slightly different string than the old hardcoded "You ready a counter-attack..." (no hyphen). Cosmetic, but you'll notice it in testing.

Ready to move on to the Mage class whenever you've applied and built this.

## Error
```rust
   Compiling ever_space_rrpg v0.1.0 (/home/vinny/Projects/roguelike_rpg/ever_space_rrpg)
error[E0308]: mismatched types
   --> src/battle.rs:164:58
    |
164 |         .filter(|(e, _, _, _)| item_usable_by_class(ecs, *e, wielder_class))
    |                                --------------------      ^^ expected `Entity`, found `&Entity`
    |                                |
    |                                arguments to this function are incorrect
    |
note: function defined here
   --> src/battle.rs:142:4
    |
142 | fn item_usable_by_class(ecs: &World, item: Entity, wielder_class: &str) -> bool {
    |    ^^^^^^^^^^^^^^^^^^^^              ------------
help: consider dereferencing the borrow
    |
164 |         .filter(|(e, _, _, _)| item_usable_by_class(ecs, **e, wielder_class))
    |                                                          +

error[E0308]: mismatched types
   --> src/battle.rs:167:54
    |
167 |                 Some((_, entities)) => entities.push(e),
    |                                                 ---- ^ expected `Entity`, found `&Entity`
    |                                                 |
    |                                                 arguments to this method are incorrect
    |
note: method defined here
   --> /home/vinny/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs:991:12
    |
991 |     pub fn push(&mut self, value: T) {
    |            ^^^^
help: consider dereferencing the borrow
    |
167 |                 Some((_, entities)) => entities.push(*e),
    |                                                      +

error[E0308]: mismatched types
   --> src/battle.rs:168:59
    |
168 |                 None => groups.push((name.0.clone(), vec![e])),
    |                                                           ^ expected `Entity`, found `&Entity`
    |
help: consider dereferencing the borrow
    |
168 |                 None => groups.push((name.0.clone(), vec![*e])),
    |                                                           +

For more information about this error, try `rustc --explain E0308`.
error: could not compile `ever_space_rrpg` (bin "ever_space_rrpg") due to 3 previous errors
```

## Title Screen and Character Select
Now `spawn_player` needs to take the chosen class instead of hardcoding `"Barbarian"`:Now the bigger part — restructuring `main.rs`. Let's view the current `State::new()` / `reset_game_state()`:Now update `game_over`/`victory` to route back to the title screen instead of restarting directly, and wire the new states into the dispatcher:Now the `tick()` dispatcher:Good — only one call site, correctly updated, and the comment is just documentation referencing the old name for context. Let's view the full new block once more end-to-end to double check borrow/logic correctness:This all lines up correctly. Let's package and deliver the three changed files.Makes sense. Here's the flow now: **Title Screen → Class Select → game**, and both Game Over and Victory route back to the **Title Screen** instead of restarting directly.

**Files — three modified:**

| Downloaded as    | Save to                                                                                    |
| ---------------- | ------------------------------------------------------------------------------------------ |
| `main.rs`        | `src/main.rs` (**replace**)                                                                |
| `turn_state.rs`  | `src/turn_state.rs` (**replace**)                                                          |
| `spawner_mod.rs` | `src/spawner/mod.rs` (**replace** — renamed to avoid collision with `spawner/template.rs`) |

**What changed:**

- **`TurnState`** gains `TitleScreen` and `ClassSelect`.
- **`State::new()`** no longer builds a game world — it just sets `TurnState::TitleScreen`. The old world-building code moved into a new **`start_game(&mut self, class: &str)`**, which is the one place a run actually begins (replaces `reset_game_state`, which always hardcoded `"Barbarian"`).
- **`spawn_player`** now takes a `class: &str` parameter instead of hardcoding it — this is the actual plumbing that lets class selection matter.
- **`title_screen`**: shows the game name, any key advances to class select.
- **`class_select`**: lists classes with number keys — `1) Barbarian` (full techniques), `2) Mage` (explicitly labeled as a placeholder on-screen — it's selectable and playable right now, but only has the universal Attack/Defend/Flee, no spells yet, since we haven't built its techniques). Picking one calls `start_game(class)`.
- **`return_to_title`**: tears down the ECS/resources and goes back to `TitleScreen`. `game_over` and `victory` now call this instead of restarting immediately — their prompts read "Press 1 to return to the title screen."

One thing worth testing specifically: I put the title text at row 15/18/30 and the class list at rows 10–21 on console 2, which is a 160×100 grid (same one `game_over`/`victory` already use successfully) — so it should fit fine, but I can't render this myself, so let me know if anything's crowded or off-screen and I'll adjust the row numbers.

Next up: giving the Mage actual spell techniques and probably differentiating its base stats (lower HP/damage, higher speed, or whatever direction you want) from the Barbarian — let me know when you're ready for that.

## Pause Screen
Now `player_input.rs` — Escape triggers pause, checked before any movement/battle logic so it never fires mid-battle (battle uses its own key handling in `battle_tick`, not this system at all):Now add a dedicated map-only render schedule in `systems/mod.rs`:Now wire it into `main.rs` — add the schedule field, a `paused_tick` method, and the dispatch arm:Now add the `paused_tick` method, right after `class_select`:Now the dispatcher:Everything is wired consistently. Let's do a final structural check of `main.rs` and `systems/mod.rs` for brace/method sanity:This all lines up correctly. Delivering the four changed files:**Four files, all modified:**

| Downloaded as     | Save to                                                           |
| ----------------- | ----------------------------------------------------------------- |
| `main.rs`         | `src/main.rs` (**replace**)                                       |
| `turn_state.rs`   | `src/turn_state.rs` (**replace**)                                 |
| `systems_mod.rs`  | `src/systems/mod.rs` (**replace** — renamed to avoid a collision) |
| `player_input.rs` | `src/systems/player_input.rs` (**replace**)                       |

**How it works:**

- **Escape pauses.** Checked first in `player_input.rs`, before any movement/attack logic, and returns immediately — sets `TurnState::Paused`. This system only ever runs while `TurnState::AwaitingInput` is active (dungeon exploration waiting on a keypress), so pausing mid-battle isn't reachable at all — `battle_tick` has its own completely separate key handling and never touches this code path. No extra guard needed for "can't pause in battle."
- **Map stays visible, no black screen.** A new `build_pause_scheduler()` runs *only* `map_render_system` — not `entity_render`, `hud`, `fov`, or anything else. Since every console gets cleared each tick regardless of state, the pause tick has to actively repaint the map (console 0) itself, but deliberately skips the entity pass (console 1), so monsters/items/the player glyph disappear and the map terrain stays exactly as it was.
- **Pause menu:** text overlay on console 2 — `Esc` resumes (back to `AwaitingInput`), `Q` quits to the title screen (via the existing `return_to_title`, tearing down the run).

One thing worth testing: I didn't add a pause hint anywhere in the regular HUD (e.g. "Esc: Pause"), so it's currently a hidden keybind — let me know if you want that surfaced somewhere.

## Better Character Select and Title Screen
Now clear the new console each tick:Now add the background-rendering schedule in `systems/mod.rs`:Now update the `State` struct and its constructors/screens in `main.rs`. First the struct and `new()`:Now add `spawn_title_background`, update `return_to_title`, and rewrite `title_screen`/`class_select`:Let me fix a bug in that loop — I referenced `row` inconsistently between the two consoles' different row scales. Let me redo it cleanly:Now let's add the `CLASS_ROSTER` data table that drives both the class-select screen and `start_game` dispatch:Now let's view the rest of `class_select` to confirm the loop closes correctly and the key-handling block is right:This is all correctly structured. Now let's double-check `spawn_player` is unaffected (still needs `class: &str` param — already done) and verify no leftover references to old class strings, then check `game_over`/`victory`/dispatcher are unaffected:All method boundaries look correct. Let's do a final full-project sanity grep and check `main()`'s console builder chain once more:I realize I introduced a compile risk: I don't have bracket-lib's source available to confirm `set_active_console`'s exact parameter type, and typing `BIG_TEXT_CONSOLE` as `usize` could mismatch it. Since every other console reference in this file already uses a plain untyped integer literal (letting inference handle it), let me match that pattern instead of risking a type error:Now replace all the `BIG_TEXT_CONSOLE` call-site references with the plain literal `5`:Good — all executable references are now the plain literal `5`; the only remaining hits are in comments, which is fine. Let's verify the tick-clear line and check the whole file compiles logically once more:All consistent — the tick-clear at line 1026, and the three uses in `title_screen`/`class_select`. Let's do one last full sanity pass and package the files:Balanced. Let's package everything for delivery:Good — Barbarian stays, `@` glyph and all. Here's what's built:

**Two files, both modified:**

| Downloaded as    | Save to                            |
| ---------------- | ---------------------------------- |
| `main.rs`        | `src/main.rs` (**replace**)        |
| `systems_mod.rs` | `src/systems/mod.rs` (**replace**) |

**Roster is now 5 classes**, all driven by one `CLASS_ROSTER` table in `main.rs` (adding a class later is just one more entry there):

| Key | Class     | Icon | Status                                |
| --- | --------- | ---- | ------------------------------------- |
| `1` | Barbarian | `@`  | Fully implemented (unchanged)         |
| `R` | Rogue     | `r`  | Placeholder — Attack/Defend/Flee only |
| `A` | Amazon    | `a`  | Placeholder                           |
| `B` | Archer    | `b`  | Placeholder                           |
| `M` | Mage      | `m`  | Placeholder                           |

**Bigger text:** added a 6th console reusing the dungeon view's own 40×25 grid on the small-text font instead of the dungeon font — same physical window, so cells land at 32×32px, 4x console 2's normal 8px text. Both title and class-select headlines use it; descriptions stay on the normal fine-text console since they're longer.

**The background map:** a new `spawn_title_background()` generates a real random level (map + monsters, same generator as a real run), marks every tile `revealed`, and spawns an invisible "anchor" entity (Player + FieldOfView, no Render component so it never draws a `@`) whose visible-tiles set is pre-filled with the *entire* map — so everything renders at full brightness, monsters included, with no real player or turn logic running. It's built once at startup and again every time you land back on the title screen (`return_to_title`), but **not** rebuilt between the title screen and class select — they share the exact same map, exactly as you asked. `start_game` wipes it outright (`ecs = World::default()`) the moment a real run begins.

**Class-select icons:** reused the existing `draw_portrait` helper (the same one that blows up a glyph for battle portraits) on console 3, one letter per class in a column. This is a placeholder that'll swap cleanly for real sprites later — once you upload the sheet, it's a matter of pointing at a different font/tile-index instead of `to_cp437(letter)`.

**Honest flag on layout:** I had to hand-compute pixel alignment across three different consoles (big text at 32px cells, description text at 8px cells, icons at 256×160px cells) with no way to render and check it myself. The math should be *close*, but I'd bet money the spacing needs a nudge once you actually see it — the easiest numbers to tweak are `headline_row` in `class_select` (currently `i * 5 + 3`) and the icon's `col`/`row` in the `draw_portrait` call.

One thing I decided without asking (small): the title-screen monsters are static, not wandering — you didn't get to that question. Easy to add if you want them animated later.

## Mage Class
Now `components.rs` — add `Invisible`, `ProvidesInvisibility`, and the `FlatDamage` technique variant:Now `battle.rs` — add the `FlatDamage` match arm to `apply_player_technique`:Now `spawner/template.rs` — wire in `ProvidesInvisibility` and rename the sword-specific machinery to be weapon-generic:Also fix the stale `spawn_prefab_sword` reference in the `prefab_only` doc comment:Now `spawner/mod.rs` — rename the wrapper, add the per-class base stats table, and use it in `spawn_player`:Now update `main.rs` call sites (`spawn_prefab_sword` → `spawn_prefab_weapon`, `prefab_sword_spawn` → `prefab_weapon_spawn`) and add `Invisible` end-turn decrement wiring:Now the map_builder side — rename the struct field and update every architect that initializes it, plus `prefab.rs`'s marker handling:Now update the doc comments in `prefab.rs` and `map_builder/mod.rs` that still say "sword":Now `use_items.rs` — wire in the Invisible Cloak effect:Now `end_turn.rs` — tick down `Invisible` once per completed player turn:Now `resources/template.ron` — rename Garrote→Rend, add Fireball/Burn techniques, the Invisible Cloak item, and 3 Staff tiers:Let me fix two glyph collisions I introduced — Wooden Staff's `|` collides with the Amulet of Yala, and Arcane Staff's `!` collides with the Healing Potion:Let's do a final sweep, then update the Mage's description on the class-select screen since it's no longer a placeholder:Now update the Mage's class-select description — it's no longer a bare placeholder:Now let's tidy the two remaining "Garrote" mentions in `battle.rs` for accuracy:Now let's do a comprehensive final sanity sweep across the whole project:Now let's package everything for delivery — 10 files touched in total:That's a big batch — 14 files, all **modified** (none new/deleted). Here's the map:

| Downloaded as         | Save to                       |
| --------------------- | ----------------------------- |
| `template.ron`        | `resources/template.ron`      |
| `components.rs`       | `src/components.rs`           |
| `battle.rs`           | `src/battle.rs`               |
| `spawner_mod.rs`      | `src/spawner/mod.rs`          |
| `spawner_template.rs` | `src/spawner/template.rs`     |
| `player_input.rs`     | `src/systems/player_input.rs` |
| `use_items.rs`        | `src/systems/use_items.rs`    |
| `end_turn.rs`         | `src/systems/end_turn.rs`     |
| `main.rs`             | `src/main.rs`                 |
| `map_builder_mod.rs`  | `src/map_builder/mod.rs`      |
| `prefab.rs`           | `src/map_builder/prefab.rs`   |
| `empty.rs`            | `src/map_builder/empty.rs`    |
| `drunkard.rs`         | `src/map_builder/drunkard.rs` |
| `rooms.rs`            | `src/map_builder/rooms.rs`    |
| `automata.rs`         | `src/map_builder/automata.rs` |

**What changed, by request:**

- **Rend replaces Garrote** — pure rename in `template.ron`. No code changes needed since the tick/apply messages already generate themselves from the item's own `Name`.
- **Fireball** needed a new effect shape, since none of the existing ones express "flat base + staff damage." Added `TechniqueEffect::FlatDamage(i32)` (new enum variant + one match arm in `apply_player_technique`) — `2` base, plus whatever's in `carried_weapon_damage`, same additive pattern the Barbarian's sword already uses.
- **Burn** needed zero new code — it's the existing `DamageOverTime` effect with smaller numbers (1 damage/3 turns, my default; trivial to retune in `template.ron` alone).
- **Base stats** are no longer one-size-fits-all: `spawner/mod.rs` now has a `class_base_stats()` table. Mage gets Health 10, Defense −1 (so it takes *more* damage — `apply_damage` subtracts Defense from incoming damage, and a negative value adds to it). Damage/Speed weren't specified for Mage, so I defaulted those to match Barbarian (1 and 6) — flag if you want them different.
- **3 Staff tiers** (Wooden/Silver/Arcane, 1/2/3 damage) are `prefab_only` items exactly like the Swords, sharing the *same* single guaranteed fortress-treasure slot — a level's guaranteed weapon is randomly a Sword or a Staff, not both, since `spawn_prefab_sword`'s eligible-item pool was already name-agnostic (filtered only by `prefab_only` + `levels`). I renamed it to `spawn_prefab_weapon` throughout (map_builder, spawner, main.rs) since it's no longer sword-specific — that rename touched the map architect files even though their actual logic didn't change, just the field name they initialize.
- **Invisible Cloak**: new `Invisible { moves_remaining }` status component + `ProvidesInvisibility` item component, consumed like any other item via `use_items.rs`. It ticks down once per completed player turn in `end_turn.rs`, and `player_input.rs` now checks it before triggering a battle — if you're Invisible and walk into an enemy, it's blocked exactly like a wall (no `WantsToMove` queued, no battle), consuming the turn the same way a real wall-bump already does.



