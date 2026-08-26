# Getting Started
Right now I will be taking the code that I made while working through the Hands on Rust book. I created a new project with Claude and it has a full set of instructions.



```bash
cargo clean --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```
Then copy the exe to a windows folder and then make sure to have the latest resources as a folder withing the same folder

```cargo build --release```
Then copy the file to an other folder /game_files as well as the latest version of the resources
``` bash
cp ever_space_rrpg/target//release/ever_space_rrpg game_files/
cp ever_space_rrpg/resources game_files/ -r
```

You can run the file with 
```bash
cd /gamefiles
./ever_space_rrpg
```


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

## Starting Items and Ice Armor
Two solid additions. Let me build both.

**Ice Armor** needs a new mechanic — none of the existing techniques model "temporary extra Defense for N incoming hits." **Starting kits** need a way to guarantee specific items land in a class's inventory at spawn, bypassing the random floor/loot pools entirely. Let's do the design work first, then implement.Now let's add the `Shield` mechanic to `battle.rs`:Now update `resolve_enemy_attack` to apply and tick down the shield, and add the `Shield` variant to `TechniqueEffect`/`apply_player_technique`:Now `components.rs` — add the `Shield` variant:Now the match arm in `apply_player_technique`:Now let's add "Ice Armor" to `template.ron`:No collision. Now let's build the starting-kits system. First, `spawn_player` needs to return the `Entity` so callers can grant items to it:Now create the new `spawner/starting_kits.rs` module:Now add `spawn_named_item` to `Templates` in `template.rs` — a guaranteed, non-random grant by name, reusing the same entity-construction helpers as battle loot:This is correctly placed and structured. Now wire it into `spawner/mod.rs`:Now the `resources/starting_kits.ron` data file:Now update `main.rs`'s `start_game` to capture the player entity and grant starting items:Only one call site, correctly updated. Now let's do a full sanity sweep across everything touched:All consistent. Let's package everything for delivery:**8 files — 2 new, 6 modified:**

| Downloaded as         | Save to                        | Action  |
| --------------------- | ------------------------------ | ------- |
| `starting_kits.ron`   | `resources/starting_kits.ron`  | **New** |
| `starting_kits.rs`    | `src/spawner/starting_kits.rs` | **New** |
| `spawner_mod.rs`      | `src/spawner/mod.rs`           | Replace |
| `spawner_template.rs` | `src/spawner/template.rs`      | Replace |
| `battle.rs`           | `src/battle.rs`                | Replace |
| `components.rs`       | `src/components.rs`            | Replace |
| `main.rs`             | `src/main.rs`                  | Replace |
| `template.ron`        | `resources/template.ron`       | Replace |

**Ice Armor:** needed a genuinely new mechanic — none of the existing effects modeled "temporary bonus Defense." Added `TechniqueEffect::Shield { defense_bonus, attacks }`. It lives on `Battle` (like Defend/Counter), not on the persistent `Defense` component, so it never leaks outside the fight it's cast in — a fresh battle always starts with no shield. It reduces incoming damage by `defense_bonus` on top of your normal Defense, and ticks down once per enemy attack actually thrown at you (not per round) until it expires. Numbers are a first guess — **+2 Defense for 10 hits** — pure data in `template.ron`, trivial to retune.

**Starting kits:** new file, `resources/starting_kits.ron`, separate from the main item table on purpose — it's a different shape of data (class → item list) and keeping it apart means it can't accidentally corrupt `Templates` parsing. A new `Templates::spawn_named_item` grants any named item straight into inventory with zero RNG — it deliberately bypasses `prefab_only`/`levels: []` restrictions, so a kit can hand out things like a starting Staff or a starting Fireball that would otherwise only come from a guarded treasure room or battle-loot luck.

Mage's kit right now: **Healing Potion, Wooden Staff, Fireball** — a real weapon, a real offensive technique, and a safety net, instead of walking into the first Goblin fight barefisted at −1 Defense. `spawn_player` now returns the player `Entity` so `start_game` can hand it straight to `grant_starting_items`. Barbarian has no kit yet since it's already proven playable without one — adding one later, or one for Rogue/Amazon/Archer, is just another `StartingKit` entry.

## Battle Screen Update Items For Classes
Let's look at the screenshot together — that muscular sprite is the actual pixel-art tile the dungeon font draws for whatever glyph code the player is using; the small pixelated `m` underneath is a *different* font rendering the literal letter (that's likely from a leftover render call, I'll check). All three issues make sense — let me dig into each.This confirms it exactly — `apply_damage` silently clamps damage by the target's Defense internally, but every message is built from the *pre-clamp* number that was passed in. Fixing this at the source:Now update every call site to shadow the local variable with the real returned value:Now the three `apply_player_technique` arms:Now the last call site, in `main.rs`'s basic Attack action:Now let's confirm every call site was caught and check brace/logic balance:Every call site now uses the real applied damage. Now let's tackle the class-restricted weapon spawning:Now the wrapper in `spawner/mod.rs`:Now tag the Swords/Staffs with their class in `template.ron`:Now update the two call sites in `main.rs` to pass the player's class:`player_entity` is available. Let's use it with `entity_class`:Now let's tackle the abilities box — moving the menu to the bottom-right with a border. Let's view the current `PlayerMenu` block:Now let's add a `draw_hollow_box` helper using only proven primitives (`DrawBatch::set` + `to_cp437`, both already used extensively throughout this file):Now let's rewrite the `PlayerMenu` block to draw the box in the bottom-right and list actions vertically inside it:This all reads correctly. Now let's do one last full-project sanity sweep, then package everything for delivery.Everything is consistent — both call sites now pass the class, both weapon families are properly tagged. Let's package the final deliverables

**5 files, all modified:**

| Downloaded as         | Save to                   |
| --------------------- | ------------------------- |
| `battle.rs`           | `src/battle.rs`           |
| `main.rs`             | `src/main.rs`             |
| `spawner_mod.rs`      | `src/spawner/mod.rs`      |
| `spawner_template.rs` | `src/spawner/template.rs` |
| `template.ron`        | `resources/template.ron`  |

**1. Damage messages now show real numbers.** Root cause: `apply_damage` silently reduced damage by the target's Defense internally, but every message was built from the number *passed in*, not what actually happened. With Mage's Defense at −1 (or any future positive-Defense enemy), the two would diverge. `apply_damage` now returns the actual damage dealt, and all seven call sites (basic Attack, every technique, the enemy's attack, Counter, and damage-over-time ticks) were updated to report that real number.

**2. Actions moved to a bordered box, bottom-right.** Console 2 is 160×100, so the box sits at columns 96–157, its height growing with however many actions that class currently has (baseline 3 for Attack/Defend/Flee, more with techniques) — well clear of the name/HP text and the message line above it. I built the border by hand with `DrawBatch::set` (the same primitive every other visual in this file already uses) rather than reaching for an unproven higher-level box-drawing call, since I can't compile-test this project myself.

**3. Mage can no longer find a Sword.** `spawn_prefab_weapon` now filters its candidate pool to the current player's class (or unrestricted), and the 3 Swords/3 Staffs got tagged `class: Some("Barbarian")`/`class: Some("Mage")` in `template.ron`. Worth knowing: Rogue/Amazon/Archer have no weapon tier defined yet, so a fortress treasure room for one of them will now find *nothing* — silent, not a crash, but you'll notice an empty guarded room until they get their own weapon type.

### Move Improvements
You're right — I hadn't finished the implementation yet, so there was nothing to send. Let me finish the remaining pieces now and then deliver everything together.Now let's view and rewrite `resolve_enemy_attack` to check the persistent `IceArmored` component instead of `battle.shield`:Now let's remove the `Shield` match arm from `apply_player_technique`:Now let's add `class_technique_names` support. First let's find `available_actions` and `BattleMenuEntry` to rework them for the full-roster/greyed-out display:Now let's rework `BattleAction`, `BattleMenuEntry`, and `available_actions` for the full-roster/greyed-out design:Now let's add `class_technique_names` — a way for `battle.rs` to see a class's full technique roster, not just what's currently owned. This needs a small addition in `spawner/template.rs` and `spawner/mod.rs`:Now let's do the full consolidation — `provides` → `effect`, and add `technique_names_for_class`:Now update the three call sites and add `technique_names_for_class`:All 3 call sites updated. Now add `technique_names_for_class`:Now the wrapper in `spawner/mod.rs`:Now let's update `use_items.rs` to work with the consolidated `Effect`/`ProvidesEffect`:Now let's update `template.ron` — change `provides:` to `effect:` everywhere, and move Ice Armor out of the technique system:Now let's update `main.rs` — move the box to the more moderate HUD console (12px cells, not the huge 32px one) and grey out locked entries:Good, `HUD_ROWS` is in scope. Let's do a full sanity sweep now across every touched file:All braces balanced, no stale code references (the one hit is just an explanatory comment), and no leftover `provides:` fields in the RON file. Let's also verify `class_technique_names` and `Effect`/`ProvidesEffect` wiring end-to-end:Everything is correctly wired end-to-end. Now let's package the full set of changed files for delivery:Here they are — 7 files, all modified:

| Downloaded as         | Save to                    |
| --------------------- | -------------------------- |
| `main.rs`             | `src/main.rs`              |
| `battle.rs`           | `src/battle.rs`            |
| `components.rs`       | `src/components.rs`        |
| `spawner_mod.rs`      | `src/spawner/mod.rs`       |
| `spawner_template.rs` | `src/spawner/template.rs`  |
| `use_items.rs`        | `src/systems/use_items.rs` |
| `template.ron`        | `resources/template.ron`   |

**What changed:**

1. **Box is smaller.** Moved off the 32px "big text" console onto the HUD console instead (~12px cells — the same "1.5x" size the dungeon HUD already uses). Its left edge is still pinned to clear the player portrait exactly, just at the new scale.

2. **Full roster, greyed out.** The battle menu now always lists every technique a class *could* have, not just what you're currently carrying. Unowned ones show as `4) Ice Armor (locked)` in dark grey and can't be selected — but wait, Ice Armor's not there anymore, see below. This needed a new `class_technique_names` lookup that reads the full `template.ron` roster for a class, separate from what's actually in your inventory.

3. **Ice Armor moved out of battle entirely.** This was the bigger piece: since Ice Armor needed two parameters (`defense_bonus` and `attacks`) and the old `provides` field only supported one, I consolidated the whole out-of-combat item-effect system into one typed `ProvidesEffect` enum + `Effect` component — the same pattern `TechniqueEffect`/`Technique` already used for battle items. Healing Potion, Dungeon Map, and Invisible Cloak all moved onto this new system too (mechanically identical, just cleaner data). Ice Armor is now a persistent `IceArmored` status (mirroring `Invisible`) applied the moment you use it from your item list — it survives across turns and even across separate battles until its 10 hits are used up, unlike before where it reset between fights. It's still Mage-only, still won from battle loot, same as Invisible Cloak.

One thing worth testing: since Ice Armor left the technique system, Mage's in-battle roster is now just Attack/Defend/Fireball/Burn/Flee — smaller box automatically. Let me know if the grey/locked look reads clearly once you see it running.


# Better Battles

Now let's look at the actual menu box drawing code (the `BattleTurn::PlayerMenu` block around line 761-800):Found it. The box is anchored to the very bottom of the HUD console (`box_y = HUD_ROWS - box_height`) regardless of where the player portrait actually sits — it was never actually positioned relative to the portrait, just clear of its right edge horizontally. Let me work out the right numbers.The player portrait (row 3 of the 5-row portrait console) occupies roughly HUD rows 40–54. Right now the box sits at `HUD_ROWS - box_height` (bottom-anchored, e.g. row ~58 for a typical box), which puts it well below the portrait instead of beside it.

I'll top-align the box with the portrait's row instead — this is a small, contained change in `main.rs`, no other files touched.Now let me sanity-check the rest of the file to confirm nothing else references `box_y`/`BOX_X` assuming the old bottom-anchored position:All uses derive from `box_y`, so they'll move together automatically — no other spot to fix.**1 file — modified: `src/main.rs`**

**What changed:** The actions box was bottom-anchored (`box_y = HUD_ROWS - box_height`), which put it near the bottom of the screen no matter where the portrait was. It's now top-aligned with the player portrait's row: since the portrait console (5 rows) and HUD console (67 rows) both span the same 1280×800 window, the portrait's row 3 works out to HUD row ~40, so the box now starts there instead. Horizontal position (`BOX_X = 44`, already clearing the portrait's right edge) is untouched. Added a `.min(HUD_ROWS - box_height)` clamp so a longer action list (more techniques than fit below row 40) can't push the box off the bottom of the screen.

Save this over `src/main.rs` and rebuild. Let me know how it looks once you see it running — "right next to" could also mean vertically centered on the portrait rather than top-aligned, so easy to nudge if row 40 isn't quite it.

## Persistent Abilities
Now let's do a sanity sweep to make sure nothing else broke — check brace balance and confirm `actions` isn't redeclared anywhere else in scope.Good — the paren imbalance pre-exists in the original file too (it's just prose in comments), so my edit didn't introduce it. Let's view the final edited section end-to-end to eyeball correctness:This reads correctly. One thing worth flagging: this box is now also visible during `FirstResult`/`SecondResult`, where `if let Some(key) = ctx.key` in those arms still advances the result screen on *any* key — including 1–9, which now visibly correspond to actions in the box. That was already true before (any key advances), just less noticeable since the box wasn't on screen then. Not a bug, but worth knowing since it might look like "pressing 2 selected Defend" when it just dismissed the result message.**1 file — modified: `src/main.rs`**

**What changed:** Moved the actions box drawing (border + labels) out of the `BattleTurn::PlayerMenu` match arm and up to run unconditionally every `battle_tick` frame, right after the name/HP text. It now renders during `PlayerMenu`, `FirstResult`, and `SecondResult` alike, instead of vanishing while a result message is on screen. The `actions` list is computed once at that same spot, since both the box's labels and `PlayerMenu`'s key-selection logic need it — no duplicate computation. Only the message text and input-handling logic stayed inside the `PlayerMenu` arm.

One thing to watch for when you test it: any key already dismisses the `FirstResult`/`SecondResult` screens, and now the action box (with its 1–9 labels) is visible while that's true. It's not new behavior, just newly visible — pressing "2" during a result screen just advances it, it doesn't secretly select Defend. Flag it if that reads as confusing in practice and we can grey the box out or hide the numbers during result screens.

## More Information
Now let's make the Battle struct edits — swapping `message` for a bounded log, and adding damage-popup state:Now update `Battle::new` and add the helper methods:Now wire in the damage-popup calls at every `apply_damage` site inside `battle.rs`, and make `entity_ice_armor` public (main.rs will need it to display the Ice Armor status):Now the three technique arms in `apply_player_technique` (DamageMultiplier, FlatDamage, MultiHit) that hit the enemy:Now let's verify no other reference to `battle.message` remains inside `battle.rs`, and check the full file's brace/logic sanity:Good — `battle.rs` is clean. Now onto `main.rs`. Let's update every `battle.message = ...` site to `push_log`, add popup tick-down, remove the old single-line prints, and add the log/status/damage-popup rendering.Now let's update the rest — view the current state of the box/status/log rendering area plus the three match arms:Now let's remove the old single-message prints and fix the three `battle.turn` match arms to `push_log` instead of assigning `.message`:Now the `FirstResult` and `SecondResult` arms — remove the message print but keep the "press any key" prompt, and swap the remaining assignments/clears:Now let's view the rest of the `FirstResult` arm and the `SecondResult` arm:Now let's confirm there are no remaining `.message` references anywhere and check overall consistency:Clean. Now let's check that `RED` and `CYAN` colors are actually available/used elsewhere (bracket-lib's `named_colors` or similar prelude constants) so I'm not introducing an undefined identifier:This all lines up cleanly with no console overlap. Let's finalize and package both files.**2 files, both modified: `src/battle.rs`, `src/main.rs`**

**Item use in battle:** left as-is, per your call — no `BattleAction::Item` added. That gap stays a "leave the fight" flow, not a mid-battle option.

**What changed:**

1. **Active effects, with time left.** `Battle.enemy_dot` (Rend/Burn), `Battle.player_defending`, `entity_ice_armor` (persistent Ice Armor), and `Battle.countering` all existed as real state but had no display before. Now shown directly under each combatant's HP bar — enemy gets a red `rend (2 turns left)`-style line; the player gets a combined cyan line like `Defending | Ice Armor (3 left)` when more than one is active at once.

2. **Floating damage numbers.** Added `DamagePopup { amount, remaining_ms }` to `Battle` for both sides, armed via two new `Battle` methods (`show_enemy_damage`/`show_player_damage`) right after every `apply_damage` call across both files — basic Attack, Counter, damage-over-time ticks, and all three enemy-damaging techniques. Ticks down each frame the same way the existing portrait flash timers do, and disappears after 700ms. One quirk worth knowing: `MultiHit` only pops the *last* hit's number (matches how its message already only reported the last hit — not a new limitation, just now visible).

3. **Battle log, message text removed.** `Battle.message: String` is gone, replaced by `Battle.log: Vec<String>` (capped at 4 lines via `push_log`, oldest drops off). It's always on screen now — no more vanishing the instant the next action resolves — and every previous `battle.message = ...` call site was converted to `push_log(...)`. The old single centered line at PlayerMenu/FirstResult/SecondResult is gone; the "Press any key to continue." prompts stayed put.

One thing to watch for once you see it running: the enemy status line, log lines, and damage popups all sit on different consoles occupying different coordinate grids that get stacked on the same physical window — I worked out the pixel math by hand to keep them clear of the actions box and each other, but I can't compile-test this project myself, so it's worth a close look the first time it renders.

## Dungeon Screen
Good — `Name` is already declared `read_component` in both `hud.rs` and `player_input.rs`, so no permission changes needed there. Let's implement the sorted hotkey ordering in `components.rs`:Now let's implement the low-health red tint and invisible/stealth grey tint in the dungeon-view render system:Let's do a final sanity check across all three touched files:Both call sites already declare `Name` access. Let's package all three files.**2 files — modified: `src/components.rs`, `src/systems/entity_render.rs`**

**Red on low health / grey on stealth:** added to `entity_render.rs`, the dungeon-view render system. It now checks, for the player entity only (enemies and items are untouched): `Invisible` present → grey; otherwise HP ≤ 30% of max → red. Invisible wins if both are somehow true at once, since a stealthed player flashing red would defeat the point of hiding. Since this checks the `Invisible` *component* rather than anything class-specific, it'll automatically apply to Rogue too whenever that class gets its own stealth ability later — no extra work needed then. One assumption I couldn't verify by compiling: I used bracket-lib's `GRAY` constant (untested in this codebase so far, though `DARK_GRAY` and other extended names like `CYAN` are already proven to work here) — flag it if the build complains about that specific name.

**Hotkeys:** rather than hardcoding "slot 1 is always the potion," I changed `usable_carried_items` (the single function both the HUD listing and the number-key handler already share) to sort by item name: Healing Potion first, Dungeon Map second, everything else (Invisible Cloak, Ice Armor, and any future class item) after — in pickup order. So **when carried**, potion is always key 1 and map is always key 2, and your out-of-combat class abilities land on 3/4/5... automatically, with zero per-class code needed as new ones get added.

One nuance worth flagging: this doesn't reserve empty slots. If you're not carrying a potion, key 1 becomes whatever's next (e.g. the map), rather than key 1 doing nothing and the map staying on 2. If you'd rather have truly fixed, always-reserved slots (like the battle screen's greyed-out "locked" actions), that's a bigger change to the dungeon HUD display — let me know if that's what you actually want and I'll build it out.

### Still Not Showing and Using the Right Hotkeys
This helped with the color of the player but not the hotkey issue. This next one will be the solution.

Good catch — that's exactly the gap I flagged before. Right now slots compact (no map → the next item slides up into 2). Let's make the slots truly fixed: potion is *always* key 1's identity, map is *always* key 2's identity, and if either isn't carried, that key just does nothing instead of anything else taking its place.Now update `player_input.rs`'s `use_item` to index off the fixed slots:Now update `hud.rs` — the displayed key number must reflect the fixed slot index, not the print order, while still skipping blank rows for empty slots:Let's do a final sanity sweep and view the finished blocks:Both consumers correctly switched over. Let's package the files.**3 files, all modified: `src/components.rs`, `src/systems/hud.rs`, `src/systems/player_input.rs`**

Added `usable_item_slots` — key 1 is now *always* the potion identity and key 2 is *always* the map identity, each `None` when not carried rather than the next item quietly taking its place. Everything else fills 3, 4, 5... in pickup order.

- **`player_input.rs`**: `use_item` now checks `usable_item_slots(...).get(n)` and only fires if that specific slot is `Some(...)` — pressing 2 with no map carried now does nothing, instead of activating whatever used to slide into that spot.
- **`hud.rs`**: the displayed number for each line is now the item's true fixed slot index (`i + 1`), not just a running count of what's been printed so far — so if you're carrying a map and an Ice Armor but no potion, you'll correctly see `2 : Dungeon Map` and `3 : Ice Armor x1`, with no line 1 shown at all (rather than the map incorrectly showing as `1`).

## Rogue
Now let's check the bump-into-enemy / battle-start logic in `player_input.rs` and where `Battle::new` gets called:Now let's check `template.ron` structure for existing techniques (to mirror the format for Garrote/Dodge) and the `Templates` struct / `technique_effect` lookup in `spawner/template.rs`:Good — `technique`/`effect` are already generic `Option<TechniqueEffect>`/`Option<ProvidesEffect>` driven purely by data, so a new variant just needs a Rust enum arm + one `battle.rs`/`use_items.rs` match arm, no template.rs changes needed. Let's check `technique_effect`/`apply_effect`/`apply_technique` functions for where new enum variants get wired:This is a big feature — let me build it out piece by piece across the files it touches. Starting with new components/effects:Now let's find `Invisible`/`IceArmored` component definitions to add `Stealthed` and `Evasion` alongside them:Now let's find where `Damage`/`Speed`/`Defense` stat components are defined to add `Evasion` alongside them:Now let's check braces balance on `components.rs` before moving on:Now let's move to `battle.rs` — add `entity_evasion`, `DodgeState`, `Battle.dodge_bonus`, `Battle.sneak_attack`, the evasion-check logic in `resolve_enemy_attack`, and the `Evade` technique arm:Now let's add the `DodgeState` struct near `CounterState`/`DotState`, and initialize the new fields in `Battle::new`:Now let's add the `DodgeState` struct near `CounterState`/`DotState`:Now let's add `entity_evasion`, right next to `entity_speed`:Now let's update `resolve_enemy_attack` to check evasion first, before the Defend/Ice-Armor damage pipeline:Let's verify there's no naming conflict with the existing `rng` declared later for the Counter roll:Good — the inner `let mut rng = ...` at line 605 is in its own nested block scope, so it shadows harmlessly; no conflict. Now let's add the `Evade` technique arm in `apply_player_technique`:Let's check `battle.rs` braces balance now:Now let's update `use_items.rs` to apply the `Stealth` effect (parallel to `Invisibility`):Now `end_turn.rs` — mirror the `Invisible` tick-down block for `Stealthed`:Now let's update `entity_render.rs`'s grey tint check to include `Stealthed`:Now the key piece — `player_input.rs`'s bump-into-enemy logic needs to check for `Stealthed` and arm the ambush:Let's check `player_input.rs` braces balance:Now let's implement the forced-first-turn and 3x-damage bonus in `main.rs`'s `battle_tick`. First, the initiative decision:Now let's find the `BattleAction::Attack` handler to add the 3x multiplier, and clear the flag after the first action resolves regardless of which action was chosen:Now let's find where the action's `match chosen` block ends so we can clear `sneak_attack` unconditionally right after it (one-shot, regardless of which action was picked):I ran out of tool budget mid-edit, so let me be upfront about exactly where things stand before wrapping up.

**What's done and verified (braces balanced, logic checked):**
- `components.rs`: added `Evasion` component, `Stealthed` component, `ProvidesEffect::Stealth`, `TechniqueEffect::Evade`
- `battle.rs`: added `entity_evasion`, `DodgeState`, `Battle.sneak_attack` + `Battle.dodge_bonus` fields, `Battle::as_sneak_attack()`, the evasion-check rewrite of `resolve_enemy_attack` (full dodge skips Defend/Ice-Armor/Counter entirely), and the `Evade` technique arm
- `use_items.rs`: `Stealth` effect now grants `Stealthed` (mirrors `Invisibility`)
- `end_turn.rs`: `Stealthed` now ticks down the same way `Invisible` does
- `entity_render.rs`: grey tint now triggers on `Invisible` **or** `Stealthed`
- `player_input.rs`: bumping an enemy while `Stealthed` now starts the battle as an ambush (`sneak_attack = true`) and consumes the status, instead of blocking the battle like `Invisible` does
- `main.rs`: initiative now forces `first_actor = Player` when `sneak_attack` is set, and the `Attack` action triples damage when it fires

**Still needed — I did not get to these:**
1. **Clear `battle.sneak_attack = false`** after the first action resolves (right after the `match chosen { ... }` block, before `battle.enter_result(...)`). Without this it's a dangling `true` that never gets consumed — I was mid-edit on this exact spot when I ran out of calls.
2. **`spawner/mod.rs`**: add `evasion: i32` to `ClassBaseStats`, give Barbarian its own arm (health 15), set Rogue's real stats (health 10, speed 10, defense 0, damage 1, evasion 10), and add `commands.add_component(player, Evasion(stats.evasion));` in `spawn_player`.
3. **`resources/template.ron`**: add three new entries — `Garrote` (`DamageOverTime(damage: 2, turns: 3)`, class Rogue), `Dodge` (`Evade(chance_percent: 70, turns: 3)`, class Rogue), `Stealth` (`Stealth(20)`, class Rogue) — following the exact pattern of Rend/Fireball/Ice Armor already in that file.
4. **`main.rs` CLASS_ROSTER**: update Rogue's description from the "(Placeholder...)" text to something real, matching Barbarian/Mage's style.
5. A final full sanity sweep (brace balance + grep check) across every touched file once the above is added, and packaging all ~9 changed files for delivery — nothing has been copied to outputs or presented yet this round.

Nothing here has been sent to you as files yet — I'd rather flag the incomplete state clearly than hand you a half-wired build. Want me to continue from where I left off?

### Continued
Now let's tackle `spawner/mod.rs` — add `Evasion` to `ClassBaseStats`, give Barbarian its own arm, and set Rogue's real stats:Now let's add the three new Rogue items to `template.ron`, following the exact pattern of the existing techniques/effects:Now let's insert the three new Rogue items right after the Invisible Cloak entry, before the Enemy entries begin:Let's verify the RON syntax is valid (quote style, no stray commas) by checking the raw file:Good — properly escaped, consistent with the file's existing CRLF/quote convention (str_replace correctly matched and inserted using the same style). Let's verify parenthesis/bracket balance across the whole RON file as a sanity check:Good — that imbalance pre-existed in the original file (RON doesn't even use `{}` for this struct style, so it's just a stray character in a comment somewhere, not a real syntax issue). Not something I introduced. Now let's update the Rogue description on the class-select screen:Let's verify this longer description still fits the layout math established earlier (Rogue is class index 1, with less vertical room than the last-class case I originally stress-tested):Fits comfortably (ends at row 21, next class's headline doesn't start until ~row 29 in the same units). Now let's do a full sanity sweep across every file touched in this feature:All balanced. Let's do a targeted end-to-end wiring check for the new pieces to make sure nothing was left dangling:Everything is wired end-to-end — no orphaned fields, every new component/effect is read *and* written somewhere. Let's also confirm `TechniqueEffect::Evade` and `ProvidesEffect::Stealth` are matched everywhere their sibling variants are (no missing match arms that would fail to compile):All 5 `ProvidesEffect` variants and all 7 `TechniqueEffect` variants have matching arms — `match` would fail to compile otherwise since neither enum has a `#[non_exhaustive]` or wildcard arm, so this confirms nothing's missing. Let's package all the changed files.**9 files — 1 modified data file, 8 modified source files:**

| Downloaded as      | Save to                        |
| ------------------ | ------------------------------ |
| `main.rs`          | `src/main.rs`                  |
| `battle.rs`        | `src/battle.rs`                |
| `components.rs`    | `src/components.rs`            |
| `spawner_mod.rs`   | `src/spawner/mod.rs`           |
| `use_items.rs`     | `src/systems/use_items.rs`     |
| `end_turn.rs`      | `src/systems/end_turn.rs`      |
| `entity_render.rs` | `src/systems/entity_render.rs` |
| `player_input.rs`  | `src/systems/player_input.rs`  |
| `template.ron`     | `resources/template.ron`       |

**Barbarian:** Health 15 (up from 10), now its own explicit stat entry rather than sharing the generic fallback.

**Rogue, fully implemented:**
- **Stats:** Health 10, Speed 10, Defense 0, Damage 1, and a new `Evasion` component/stat (10%) — a new persistent per-entity value, currently 0 for every other class.
- **Garrote:** identical `DamageOverTime` shape to Rend/Burn — 2 damage/turn for 3 turns. No new mechanic needed.
- **Dodge:** new `TechniqueEffect::Evade` — +70% evasion for the next 3 enemy attacks, stacking additively on base Evasion (80% total while active), confirmed per your answer.
- **Stealth:** new `Stealthed` component + `ProvidesEffect::Stealth`, deliberately **not** reusing `Invisible` — Invisible blocks a bump-into-enemy from starting a fight at all; Stealth lets the fight start as an **ambush** instead. Bumping an enemy while Stealthed now forces the player to act first regardless of Speed, and triples the damage of whichever action they pick first (scoped specifically to a basic **Attack** — I didn't extend the 3x multiplier to Techniques, since that wasn't explicitly confirmed; flag it if you want that too). Stealth is consumed the instant the ambush triggers. It also gets the same grey dungeon-view tint as Invisible now, since `entity_render.rs` checks for either.

A few implementation notes worth knowing:
- A full dodge (from Evasion or Dodge) skips the *entire* damage pipeline for that hit — no Defend or Ice Armor gets consumed, no Counter triggers, since nothing actually landed. Logged as the existing "Dodge attack." line.
- Dodge's 3-turn duration ticks down once per enemy attack faced, win or lose the roll — matching how Rend/Burn's duration already works.
- I updated Rogue's class-select description off the old "(Placeholder...)" text to describe the real kit, and re-verified it still fits on screen using the same row-math check I ran for the other classes earlier in this project.

I traced every new field/component/enum variant to confirm nothing's orphaned, and cross-checked that both `ProvidesEffect` and `TechniqueEffect` have a match arm for every variant (Rust won't compile an inexhaustive match on either, so that's a real guarantee, not just a spot-check) — but I can't run `cargo build` myself, so a real compile is still the first thing worth doing before playtesting.

### Continued
Daggers not being spawned, No Stealth for the player working yet, Evasion is not working atm

Let me check a few things before assuming anything — starting with whether there's an actual bug, since "nothing seems to work" could point to something more fundamental than four separate issues.Good, no compile-breaking typo. Let's check the actual enemy-chasing system, since that's a concrete, verifiable gap I know I never touched:Found real bugs on two of these — let's check the starting-kit system before fixing everything, since Rogue currently has no way to reliably get any of its items to even test with:Now let's add the Dagger weapon tier right after the Staff tiers:Now let's fix `chasing.rs` — enemies never check `Stealthed` at all, only `Invisible`:Now let's add a Rogue starting kit so Stealth/Dodge/Garrote/a weapon are actually reachable to test, mirroring Mage's exactly:Now let's re-scrutinize the sneak-attack and evasion code paths one more time for an actual bug, rather than assuming it's just item scarcity:Let's also confirm the Rogue weapon-tag pattern matches what `spawn_prefab_weapon`/`grant_random_battle_loot` expect, and do a final grep sweep:All in place. Let's package everything for this round.**3 files — 1 modified source file, 2 modified data files:**

| Downloaded as       | Save to                       |
| ------------------- | ----------------------------- |
| `chasing.rs`        | `src/systems/chasing.rs`      |
| `template.ron`      | `resources/template.ron`      |
| `starting_kits.ron` | `resources/starting_kits.ron` |

**Daggers — confirmed, real gap on my part.** I built out Garrote/Dodge/Stealth last time but genuinely forgot the weapon tier — Barbarian has Swords, Mage has Staffs, and I never gave Rogue an equivalent. Added `Rusty Dagger`/`Silver Dagger`/`Venomous Dagger` (glyphs `4`/`5`/`6`, damage 1/2/3), sharing the same guaranteed-fortress-treasure slot as Swords/Staffs — no code changes needed there since that system already generalizes across weapon types by class tag.

**Enemies ignoring Stealth — also a real gap.** `chasing.rs` already had a full early-out for `Invisible` ("no enemy can see you, so none of them move toward you") but I never extended that check to `Stealthed` when I built it. Fixed — enemies now can't detect or chase the player under either status. This doesn't touch the player-initiated ambush path in `player_input.rs` at all, so walking into an enemy while Stealthed still triggers the ambush as designed.

**Tripled damage / evasion "not working" — I re-audited both code paths carefully and didn't find a bug in the logic itself.** What I did find: Garrote, Dodge, and Stealth were all **battle-loot-only** (a 40% chance per victory, split three ways) with zero other way to obtain them — no floor spawns, no starting kit. It's entirely possible you were testing with base 10% Evasion alone (subtle over a few fights) and never actually got a Stealth or Dodge item to test the bigger, more obvious effects. I gave Rogue a starting kit (mirroring Mage's, which exists for the identical reason) — `Healing Potion`, `Rusty Dagger`, `Garrote`, `Dodge`, `Stealth` — so all of this is guaranteed reachable from turn one instead of down to loot luck.

Worth retesting now that these are actually reachable — if the tripled damage or evasion still doesn't show up once you can reliably get a Stealth/Dodge item in hand, that points to a real bug and I'll dig further.

## Moving Enemies on the Start Screen
Now let's implement — tag enemies with `MovingRandomly` for ambient wandering, and make the invisible anchor entity actually `Invisible` so no wandering enemy can accidentally trigger a battle against it:Now let's update the schedule to actually run the movement pipeline:Before finalizing, let me check whether the `Option<Battle>` resource `random_move` now depends on is actually guaranteed to exist every time this schedule runs — since `return_to_title` wipes all resources:Found a real landmine — `Option<Battle>` is never inserted until a real run starts (`start_game`), but `random_move_system` now requires that resource. Both `State::new()` and `return_to_title()` would panic the moment this schedule runs, since neither of them inserts it. Let's fix that:Now let's find and fix `return_to_title`:Good — plain marker component, `Copy`, works fine with `add_component`. Let's do one final end-to-end grep sweep and package the changes.Both fixes at lines 282 and 421 confirmed, and the new schedule (lines 81/83/85) is wired correctly. Let's package the two changed files.**2 files — both modified: `src/main.rs`, `src/systems/mod.rs`**

**What changed:**

1. **`systems/mod.rs`**: `build_title_background_scheduler` now runs `random_move_system` → `movement_system` → `animation::tick_animations_system` before the existing map/entity render. Deliberately used `random_move`, not `chasing` — chasing paths every enemy toward a specific target, which here would mean converging on the invisible anchor entity.

2. **`main.rs`, `spawn_title_background`**: real dungeon enemies only carry `ChasingPlayer` (they path toward the real player), not `MovingRandomly` — so without tagging them, the new schedule would've had nothing to actually move. Now tags every spawned background enemy with `MovingRandomly` so they wander.

**A real bug I caught before it shipped:** `random_move_system` requires the `Option<Battle>` resource, and I traced through — neither `State::new()` nor `return_to_title()` ever inserted it; it was only ever set once a real run starts via `start_game`. That was harmless before, since nothing in the background schedule touched it, but adding `random_move_system` would have made the game **panic on startup** the instant it reached the title screen. Fixed by inserting `None::<Battle>` in both places.

**One more defensive measure**, though I found while double-checking that it's actually not load-bearing: I made the invisible anchor entity permanently `Invisible`, so if a wandering enemy ever steps onto its tile, `random_move` treats it like bumping a wall rather than trying to start a battle (which would panic given the anchor has none of the Health/Damage/Speed/etc. components battle code expects). Turns out the anchor's missing `Health` component already excludes it from that collision query entirely, so this couldn't actually have fired — but it's cheap, correct, and documents the intent clearly in case that changes later.

One thing I can't verify without compiling: whether the visual result feels like the right amount of movement — `random_move` picks a fully random adjacent tile every turn with no bias toward open space, so enemies may wander into walls/each other somewhat aimlessly rather than looking purposeful. Worth a look once it's running.

### Continued (Moving too Fast)
The root cause is clear: this schedule runs at full frame rate (30fps) with no turn-based pacing at all, unlike real gameplay which only advances monster movement once per player input. Let's split rendering from movement and gate movement behind a timer.First, let's revert `build_title_background_scheduler` back to render-only, and add a separate movement schedule:Now let's add the second schedule field, a timer resource, and gate the movement schedule in `main.rs`:Now let's update `return_to_title`'s comment for accuracy, then gate the movement schedule in `title_screen`/`class_select`:Now let's find `title_screen`/`class_select` and add the timer-gated movement call. We need `ctx.frame_time_ms` to accumulate the timer:Now let's find and update `class_select`'s call site:Let's check for any remaining stale comment references and confirm brace balance:Good — no stale references, both files balanced. Let's do a final cross-check that both schedules are constructed and stored correctly:Everything's wired correctly — field declared, initialized in `new()`, reset in `return_to_title`, and consumed by the shared `tick_background` helper used by both `title_screen` and `class_select`. Let's package both files.**2 files — both modified: `src/main.rs`, `src/systems/mod.rs`**

**Root cause:** the background schedule was running the full movement pipeline every single rendered frame — 30 times a second at this project's FPS cap — with no pacing at all. Real gameplay movement only ever advances once per player keypress via the `TurnState` machine; this decorative world has no equivalent gate, so enemies were stepping a full tile 30x/second instead of once per turn.

**Fix:** split the one schedule into two — `background_systems` (map/entity render, still every frame, so there's no flicker) and a new `background_movement_systems` (random_move → movement → animation), which now only runs once every `BACKGROUND_MOVE_INTERVAL_MS` (400ms, easy to retune — that's the single number to change if 400ms still feels off in either direction). A small `background_move_timer_ms` field accumulates real elapsed time each frame and resets after each movement pass; a shared `tick_background` helper handles both the timer and the always-render call, used by both `title_screen` and `class_select`.

Take a look and let me know if 400ms feels right, too slow, or still too fast.

# Victory and Death Screens
Now let's implement. First, add the tint helper next to `vignette`:Now add the `draw_end_screen_background` method. Let's find where `draw_battle_arena` starts to place this new method near it:Now update `game_over` and `victory` to call these:Now the victory screen:GOLD isn't actually proven elsewhere in the codebase (that hit was just my own edit) — matching the project's own past caution about unproven color constants (e.g. the earlier ORANGE→YELLOW swap), I'll use YELLOW instead, which is used successfully throughout this file already:Now let's sanity-check everything compiles. Let's try `cargo check` (network is disabled, but dependencies may already be vendored/cached):No Rust toolchain in this sandbox, so let's do careful manual verification instead — brace balance and a grep sweep for the new code:Good — this exact query pattern is already proven in `hud.rs` and `player_input.rs`. Let's finalize by copying the file to outputs and presenting it.**1 file — modified: `src/main.rs`**

Both `game_over` and `victory` were plain text on a black console 0/1 (nothing else draws to those consoles in those states). Rather than building a separate fake world (like the title screen's decorative background needed to, since there's no real run before you start one), I reused what's already there: **neither death nor victory tears down the current run's `ecs`/`resources`** — that only happens in `return_to_title`, once you dismiss the screen. So the actual map, theme, and even your player entity (with its real glyph/class) are all still sitting there, unused.

**What I added:**

1. **`tint_color()`** — a small helper that multiplies an RGB color by a tint, so I can recolor the theme's existing floor/wall palette instead of hardcoding a whole second palette.
2. **`draw_end_screen_background()`** — same floor/wall/vignette technique `draw_battle_arena` already proves works, minus the battle-specific scenery, tinted red (defeat) or warm gold (victory). Draws the *actual* dungeon you died/won in, dimmed or glowing, instead of black.
3. **`draw_end_screen_portrait()`** — your own glyph, drawn big on the battle-portrait console (same trick as battle portraits), recolored as a solid dark-grey silhouette for defeat or yellow for victory.
4. Both screens now call these before their existing text — none of the existing text/keybind logic changed.

**One thing to flag honestly:** I originally reached for a `GOLD` color constant for the victory portrait, then caught that it wasn't actually proven anywhere in your codebase (only my own new line) — swapped it for `YELLOW`, which is used successfully throughout this file already. Since I can't compile this myself, that's the kind of thing worth a close look on first build, along with the portrait's exact row/column placement (centered on the portrait console, row/col math worked by hand).

## Errors
```rust
error[E0308]: mismatched types
    --> src/main.rs:1397:39
     |
1397 |         self.draw_end_screen_portrait(DARK_GRAY);
     |              ------------------------ ^^^^^^^^^ expected `RGB`, found `(u8, u8, u8)`
     |              |
     |              arguments to this method are incorrect
     |
     = note: expected struct `bracket_lib::prelude::RGB`
                 found tuple `(u8, u8, u8)`
note: method defined here
    --> src/main.rs:734:8
     |
 734 |     fn draw_end_screen_portrait(&mut self, tint: RGB) {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^            ---------
help: call `Into::into` on this expression to convert `(u8, u8, u8)` into `bracket_lib::prelude::RGB`
     |
1397 |         self.draw_end_screen_portrait(DARK_GRAY.into());
     |                                                +++++++

error[E0308]: mismatched types
    --> src/main.rs:1431:39
     |
1431 |         self.draw_end_screen_portrait(YELLOW);
     |              ------------------------ ^^^^^^ expected `RGB`, found `(u8, u8, u8)`
     |              |
     |              arguments to this method are incorrect
     |
     = note: expected struct `bracket_lib::prelude::RGB`
                 found tuple `(u8, u8, u8)`
note: method defined here
    --> src/main.rs:734:8
     |
 734 |     fn draw_end_screen_portrait(&mut self, tint: RGB) {
     |        ^^^^^^^^^^^^^^^^^^^^^^^^            ---------
help: call `Into::into` on this expression to convert `(u8, u8, u8)` into `bracket_lib::prelude::RGB`
     |
1431 |         self.draw_end_screen_portrait(YELLOW.into());
     |                                             +++++++

For more information about this error, try `rustc --explain E0308`.
error: could not compile `ever_space_rrpg` (bin "ever_space_rrpg") due to 2 previous errors
```

Good, exactly what the compiler suggested — the named color constants (`DARK_GRAY`, `YELLOW`, etc.) are `(u8, u8, u8)` tuples, not `RGB` directly; they need `.into()` to convert. Fixing both call sites:That fixes it — the compiler's suggested `.into()` is correct: bracket-lib's named color constants like `DARK_GRAY` and `YELLOW` are actually plain `(u8, u8, u8)` tuples, and `RGB` implements `From<(u8, u8, u8)>`, so `.into()` converts them cleanly. Nothing else in the two new functions is affected — they already work in plain `RGB` throughout.**1 file — modified: `src/main.rs`**

Same file as before, with the two `.into()` calls added. Rebuild and it should compile clean now.

## Make the Icon Look Dead
Got it — you want the fallen hero's glyph actually rotated to lie on its side, tinted red. That needs a different rendering path than the plain simple console `draw_portrait` uses — bracket-lib's simple consoles can't rotate a glyph, only bracket-lib's "fancy console" feature (`DrawBatch::set_fancy`) can. Let me check the console setup and wire in a dedicated fancy console for this.Good — confirmed nothing in this project has ever used bracket-lib's "fancy console" rotation feature (there's only a commented-out, never-activated stub for sub-pixel movement that even mentions abandoning that path). This will genuinely be new ground for this codebase, so I'll flag that clearly. Let's build it.Now let's add the fallen-portrait function using a new fancy console, right after the existing portrait helper:Now add the console index constant and update the console-layout comment block near the top:Now update the console-clearing block in `tick()` and the `BTermBuilder` chain in `main()`:Now the `main()` builder chain to register console 6 as a fancy console:Now let's update `game_over` to use the fallen, red-tinted portrait, and add bigger text on the title-screen's big-text console:Now let's do a final sanity sweep — brace balance, and check for any missed references:Everything's wired consistently. Let's ship it.**1 file — modified: `src/main.rs`**

**What changed:**

1. **Fallen, red, rotated hero.** `draw_portrait`/`draw_end_screen_portrait` can only stamp a glyph upright onto a fixed grid cell — there's no rotation on a plain "simple console." Real rotation needs bracket-lib's **"fancy console"** feature (`DrawBatch::set_fancy`), which nothing in this project has used before (I checked — there's only a commented-out, never-activated stub for sub-pixel movement that even mentions the team backing away from fancy consoles for a different reason). So this adds a new **console 6**, registered via `.with_fancy_console(...)`, using the same square 32×32px cells as the main dungeon view so a 90° turn doesn't stretch the glyph. `draw_end_screen_fallen_portrait` draws your hero's own glyph there, rotated flat and tinted red, centered on screen.

2. **Bigger death-screen header.** "Your quest has ended." now renders on the same big-text console the title screen's logo already uses (32px cells vs. the old 8px), with the paragraph text repositioned below it using the same pixel-height math `class_select` already uses to avoid overlap. The fallen portrait sits center-screen; the two closing lines ("Don't worry...", "Press 1...") were pushed down to clear it.

**Flagging honestly:** `DrawBatch::set_fancy`'s exact signature — specifically whether rotation is a plain `f32` in radians or a wrapper type like `Radians`/`Degrees` — is my best recollection, not something proven anywhere in your codebase like the rest of this file's patterns. This is the one real "might not compile as-written" risk here. If it errors on that specific call, paste the message back and I'll fix just that line — the console setup, tinting, and layout around it should be sound regardless.

Victory screen is untouched for now, per "I haven't seen the victory screen yet" — take a look at both and let me know what you want adjusted (including whether Victory should get the same big-text treatment).

### Errors
```rust
That's a real, specific type mismatch, not a guess-and-hope situation — the error itself points to the fix: `set_fancy` wants something that converts `Into<Radians>`, and `Degrees` is the type that does that (not a bare `f32`). Swapping to `Degrees`:**1 file — modified: `src/main.rs`**

Your compiler error was actually the useful kind — it confirmed `set_fancy` wants `Into<Radians>`, and pointed at `Degrees` as the type that provides it. Swapped the rotation argument from a raw `f32` to `Degrees::new(90.0)`.

**One small remaining guess, flagged plainly:** I'm assuming `Degrees` has a `::new()` constructor, matching every other geometry type this file already uses (`Point::new`, `PointF::new`, `RGB::from_f32`). If it's actually a plain tuple struct instead, the fix is a one-word swap to `Degrees(90.0)` — paste the error if it complains and I'll make that change immediately.
```

Good, useful screenshot — two real things to fix:

1. **The rotated icon rendered tiny.** I placed it correctly (dead center), but I left `set_fancy`'s scale parameter at `1.0` — that's exactly the knob that should blow it up, similar to how the battle portraits get "big" from their coarse grid. Bumping the scale up.
2. **Body text needs to be bigger too**, not just the header — right now it's still on the small 8px console, which now looks tiny next to the big header and needs to move down anyway to clear the now-bigger icon.Add a scale constant near the other console-layout constants:Now let's move the body text onto the HUD console (already used for the "1.5×" bigger dungeon-view text elsewhere) and recompute row positions to clear both the header and the now-larger icon:**1 file — modified: `src/main.rs`**

**Two fixes:**

1. **Icon size.** I had `set_fancy`'s `scale` parameter locked at `1.0` — native glyph size, which is tiny against a 1280×800 window. That's the actual knob for size on a fancy console (there's no coarse-grid trick available here like the battle portraits use, since fancy-console positions aren't snapped to a grid). Added a new `END_SCREEN_FALLEN_SCALE` constant (currently `6.0`) and wired it into both the x and y scale — uniform, so the rotation stays a clean square instead of stretching.

2. **Body text.** Moved the four paragraph lines off the small 8px console onto console 4 — the same "~1.5× bigger" HUD console already used for the dungeon HUD text — and recalculated every row position in actual pixels rather than row counts, since the header (32px cells), body (~12px cells), and the now-much-bigger icon all live on differently-scaled consoles that need to clear each other's real screen space, not just look right by row-number coincidence.

`END_SCREEN_FALLEN_SCALE = 6.0` is the one number worth tuning by eye once you rebuild — bump it up or down if the icon still reads too small/large.

### Weird Black Background
That black box is real — the fancy console renders an opaque background quad from whatever color we pass, and we passed plain `BLACK`. There's no proven "transparent background" API for a fancy console in this codebase to fall back on safely, so instead of guessing at a new, unverified API surface, I'll make the box's background color exactly match the arena color that's already drawn behind it at that spot — same vignette math, computed at the icon's exact position, using only APIs already proven in this file.Now update the `game_over` call site to pass both tints:Good, all consistent — one call site, correctly updated with both tints.**1 file — modified: `src/main.rs`**

**Root cause:** a fancy console always renders an opaque background quad — unlike every other overlay console in this file, which are registered "`_no_bg`" and let console 0 show through. I'd been passing plain `BLACK` for that background, which is exactly the solid box you saw.

Rather than guess at a second unverified API (whether there's some transparent-background trick for fancy consoles — nothing in this codebase proves one exists), I made `draw_end_screen_fallen_portrait` compute the **exact same vignette color** `draw_end_screen_background` already painted at that spot on console 0, and use that as the quad's background. Both functions now take the same `background_tint`, so the icon's backing box is mathematically guaranteed to match the arena color right behind it — no transparency needed, and no new API surface risked, since this only reuses `vignette`/`tint_color`/`RGB`, all already proven elsewhere in this file.

It won't be a *perfect* seam if the vignette shifts noticeably between the icon's center and its edges (the box is ~192px wide, so there's some falloff across it), but it should read as blended rather than a hard black rectangle. Let me know how it looks once you rebuild.

### Still Weird Background (Try Transparent)
Understood — let's actually try it. Given the earlier evidence that `ColorPair::new` accepted plain `RGB` without complaint, and this project's later features (flash tinting, the abandoned fade-tint idea) all assume this engine supports real alpha blending under the hood, my best bet is that `ColorPair` is built on `RGBA` (with `RGB` auto-converting to opaque alpha=1), which would make a genuinely transparent background possible by passing `RGBA` with alpha=0 explicitly.Now update the `game_over` call site — no longer needs the shared `background_tint` for the portrait call:Both helpers are still used by `draw_end_screen_background`/`draw_battle_arena`, so nothing's orphaned. Let's ship it.**1 file — modified: `src/main.rs`**

**What changed:** `draw_end_screen_fallen_portrait` now passes `RGBA::from_f32(0.0, 0.0, 0.0, 0.0)` (fully transparent) as the background, instead of trying to color-match the arena. Both `fg`/`bg` are now explicitly typed as `RGBA` rather than `RGB`, betting that `ColorPair` is actually built on `RGBA` internally (with plain `RGB` silently converting to opaque alpha=1 — which is consistent with every other `ColorPair::new(RGB, RGB)` call in this file compiling fine without ever needing alpha before).

Also simplified: dropped the now-unused `background_tint` parameter and the vignette-sampling logic entirely, since there's no longer a background color to match.

**This is a real guess, not a proven pattern** — nothing in this codebase has needed an alpha channel until now. If it fails to compile, the error will tell us exactly what type `set_fancy`/`ColorPair` actually expect, and I can fix it precisely from that. If it *does* compile but doesn't render transparently (i.e., alpha isn't actually respected by this rendering path), that'll only be visible once you run it — worth flagging that "compiles" and "looks right" are two separate things to check here.


# Debug Class
Debug
hp - 100hp
evasion - 50
damage - 5
speed - 10
defense - 0

abilities
Victory - Instantly finishes the game
Defeat - Instantly Loses the game
Next Level - Instantly move to the next level

Starting Items
Victory - x1
Defeat - X1
Next Level - x2

Good, simple to extend. Now let's check the class glyphs already in use, `class_base_stats`, `Templates`/`spawner`/`template.ron` structure, and `use_items.rs`'s resource access to `TurnState`:Now let's check `starting_kits.ron` and `starting_kits.rs` for exact format:Good, this fully applies effect/technique/class/description. Now let's check `use_items.rs`'s system macro signature to add the `TurnState` resource cleanly, matching the pattern already used in `end_turn.rs`:Everything I need is confirmed. Let's implement.

### 1. `components.rs` — add three new `ProvidesEffect` variants### 2. `use_items.rs` — wire the three effects to `TurnState`### 3. `spawner/mod.rs` — add Debug's base stats### 4. `resources/template.ron` — add the three Debug items### 5. `resources/starting_kits.ron` — grant them (Next Level ×2 via repetition)### 6. `main.rs` — hidden `\` shortcut in `class_select`Confirmed — fully generic, no per-variant code needed there. Now a final sanity sweep across all touched files:Everything is consistently wired. Let's copy all six changed files to outputs and present them.Yes, this all makes sense — and I designed it around one key realization: **"Victory," "Defeat," and "Next Level" aren't battle techniques, they're out-of-combat items**, using the same system as Healing Potion/Dungeon Map/Ice Armor rather than the battle-menu Technique system. That's the right fit since you'd use them while exploring, not mid-fight.

**6 files:**

| Downloaded as       | Save to                       | Action  |
| ------------------- | ----------------------------- | ------- |
| `main.rs`           | `src/main.rs`                 | Replace |
| `components.rs`     | `src/components.rs`           | Replace |
| `use_items.rs`      | `src/systems/use_items.rs`    | Replace |
| `spawner_mod.rs`    | `src/spawner/mod.rs`          | Replace |
| `template.ron`      | `resources/template.ron`      | Replace |
| `starting_kits.ron` | `resources/starting_kits.ron` | Replace |

**How it works:**

- **Stats:** Health 100, Damage 5, Speed 10, Evasion 50, Defense 0 (not specified, so left at the default). Glyph `D` — checked against every existing player/monster/item glyph, no collisions.
- **The three items** are new `ProvidesEffect` variants (`DebugWin`/`DebugLose`/`DebugNextLevel`). Using one just sets `TurnState` directly to `Victory`/`GameOver`/`NextLevel` — the exact same states the real win/lose/level-exit conditions already set in `end_turn.rs`, so `main.rs`'s existing dispatcher and screens handle them with zero new code. I confirmed `end_turn.rs`'s own state-transition logic has a catch-all that won't clobber this if it runs later in the same tick.
- **Starting kit:** Victory ×1, Defeat ×1, Next Level ×2 — done by just listing `"Next Level"` twice in the kit's item list, since items already stack (no count field needed).
- **Hidden access:** `Debug` isn't in `CLASS_ROSTER` at all, so it's invisible on the class-select screen. Pressing `\` there calls `start_game("Debug")` directly, bypassing the visible roster entirely.

**One thing worth double-checking on your end:** I used `VirtualKeyCode::Backslash`, which is standard in winit's keycode enum but hasn't been used anywhere in this codebase before — if the build complains about that specific identifier, tell me the error and I'll adjust it.

One thing I didn't add: a Defense stat for Debug (left at 0, same as most classes) since you didn't list one — flag it if you want it tanky on that front too.

## Can't Select
Good — that narrows it down a lot. Punctuation keys like backslash are exactly the kind of thing that can behave inconsistently through WSLg/X11 (this project's already hit real X11/WSL keyboard quirks before). Letter keys, on the other hand, are already proven working in this exact codebase — `G` for pickup, `Q` for pause-quit — so let's switch to a letter instead of gambling on punctuation again.**1 file — modified: `src/main.rs`**

Swapped the shortcut from `\` (`VirtualKeyCode::Backslash`) to **`D`** (`VirtualKeyCode::D`). Since it builds fine but the key just never registers, this smells like the same category of issue as the project's documented WSLg/X11 windowing quirks — punctuation keys are more prone to that than plain letters, and letter keys (`G`, `Q`) already work reliably elsewhere in this exact codebase. `D` for "Debug" is free (only `Key1`–`Key5` are handled on this screen), so no collision.

If `D` still doesn't register either, that would point away from a keycode-naming issue entirely and toward something upstream eating the keypress before it reaches `class_select` — worth knowing either way.

# Victory Screen
D-for-Debug worked, and there's the Victory screen — confirmed it's the plain small-text layout we knew about. Let's rebuild it to match the same treatment as Game Over: big header, bigger body text, portrait/icon centered, and the "Press 1" prompt pushed to the bottom.Now let's update `draw_end_screen_portrait` to take a column (so the hero can sit beside the amulet instead of dead-center), and add a matching amulet-drawing method:Now let's rewrite `victory()` with bigger text, both icons, and the prompt at the bottom:Good — one call site, correctly updated to the new two-arg signature. Final balance check and ship:**1 file — modified: `src/main.rs`**

**What changed:**

1. **Bigger text** — same treatment as Game Over: "You have won!" moved to the big-text console (32px cells), body paragraphs moved to the HUD console (~12px, bigger than the old 8px fine text), all row positions recalculated in real pixels so nothing overlaps across the differently-scaled consoles.
2. **"Press 1..." moved to the bottom** — row 60 out of 67 on the HUD console, well clear of the icons above it, near the bottom of the screen instead of right under the paragraph text.
3. **Amulet shown alongside the hero** — `draw_end_screen_portrait` now takes a `col` parameter instead of always centering, so the hero sits to the left and a new `draw_end_screen_amulet` draws the Amulet's own glyph (`|`, same one `spawn_amulet_of_yala` uses) to the right, both tinted gold. I hardcoded the Amulet's glyph rather than looking up the live entity — by the time this screen shows, the run's basically over, and there's no guarantee that entity's still sitting in the ECS, so there's nothing to gain from depending on it.

Both icons use the same non-rotated, non-transparent drawing method the original victory portrait already used (the coarse 5×5 grid trick, not the fancy-console rotation from Game Over) — no new API risk here, since a standing hero and a held-up amulet don't need to be "on their side."

# Refactor main.rs
No real pushback on the idea itself — `main.rs` is **1,693 lines** now and doing at least five distinct jobs: app bootstrap/console setup, the `State` lifecycle (new run, level transitions, title background), several full-screen UI states (title, class select, pause, battle, victory, game over), a ~480-line `battle_tick` that's become its own subsystem, and a pile of free-standing draw helpers (`vignette`, `draw_tree`, `flash_tint`, etc.) that have nothing to do with game state at all. That's a lot to hold in one file when you just want to tweak, say, the victory screen.

Two technical notes, not objections, just things worth knowing going in:

1. **This is safe mechanically.** `State`'s fields are all private, but Rust's privacy rule is "visible to the defining module and all its descendants" — since `State` is defined at the crate root (`main.rs`), any new submodule we create can still see and use those private fields directly, no need to mark anything `pub`. Splitting `impl State { ... }` across multiple files is completely normal Rust.

2. **`battle_tick` is the risky part.** It's the single biggest, most tangled function in the file — lots of local state, nested matches, borrows of `self.ecs`/`self.resources`. Moving code between files doesn't change what it *does*, but I can't compile-test here, so for a function that size I'd rather move it as its own careful pass and have you build/confirm before continuing, rather than one giant everything-at-once rewrite that's hard to debug if something's off.

Proposed module split, for your sign-off before I start moving code:

- **`src/render_helpers.rs`** — the free functions with no `State` dependency: `draw_portrait`, `wrap_text`, `draw_hollow_box`, `flash_tint`, `draw_tree`, `vignette`, `tint_color`.
- **`src/screens/title.rs`** — `title_screen`, `class_select`, `spawn_title_background`, `tick_background`, `CLASS_ROSTER`/`ClassRosterEntry`.
- **`src/screens/pause.rs`** — `paused_tick`.
- **`src/screens/battle.rs`** — `battle_tick`, `battle_victory_tick`, `draw_battle_arena` (the big one — done last, its own pass).
- **`src/screens/end.rs`** — `game_over`, `victory`, `draw_end_screen_background`/`portrait`/`amulet`/`fallen_portrait`.
- **`main.rs`** stays with: `State` struct definition, `new`/`start_game`/`advance_level`/`return_to_title` (core lifecycle), the `GameState::tick` dispatcher, and `main()`.

Want me to go with this breakdown, or adjust the boundaries first?

## Solution
Now let's view the entire `battle_tick` body (368-852) and `battle_victory_tick` (852-892):Given the size, let's do this mechanically with `sed` to guarantee an exact, byte-for-byte extraction rather than retyping 500+ lines by hand.Good, that's exactly the closing brace of `battle_victory_tick`. Now let's build the new file with a proper header, then splice in the extracted body:Now mark `battle_tick` and `battle_victory_tick` as `pub` (they're called from `main.rs`'s `tick()` dispatcher, an ancestor module — so they need `pub` just like the other screen entry points):Now remove this block from `main.rs` and register the module:Delete lines 248 (doc comment start) through 892 (`battle_victory_tick`'s closing `}`), leaving the `impl State`'s own closer at 893:Same off-by-one closing-brace issue as before — `start_game`'s closing `}` was kept but `impl State`'s own closer is now missing. Fix:That confirms it — `advance_level` is the last method, correctly closed, followed by `impl State`'s closer, then `impl GameState`. Structure is exactly what I expect: `State::new`, `start_game`, `return_to_title`, `advance_level` remain in `main.rs`. Now let's register `mod battle;` and check overall balance:Every file is brace-balanced. Now let's do a full-project sanity sweep — check that nothing references a moved item incorrectly, and view the final `main.rs` end-to-end:Let's verify every cross-module `self.` method call resolves correctly — check each new screens file for method calls (not just field access) and confirm they're visible from that location:No duplicates, every function appears exactly once. Now let's verify no other file in the project referenced any of the moved free functions with an unqualified path that would now break, and check the `screens/mod.rs` final state:Found two stale comments in `src/battle.rs` referencing "flash_tint in main.rs" — now inaccurate since it moved to `render_helpers.rs`. Let's fix those too:Also fix the `spawner/mod.rs` comments referencing "CLASS_ROSTER in main.rs" (now `screens/title.rs`):Now let's also check for any other stale "in main.rs" references throughout the project that might now be inaccurate:Also caught a **pre-existing** stale comment unrelated to this refactor — `spawner/mod.rs` still says "hidden Backslash shortcut" even though we swapped that to the `D` key a few turns ago. Fixing all three:Now a final complete sweep — brace balance on every touched file, and one more grep for any remaining stale cross-references:All balanced. Now let's package the full delivery — 10 files, mostly new plus a few touched-up.Refactor's done. **10 files — 6 new, 4 modified:**

| Downloaded as       | Save to                       | Action  |
| ------------------- | ----------------------------- | ------- |
| `main.rs`           | `src/main.rs`                 | Replace |
| `render_helpers.rs` | `src/render_helpers.rs`       | **New** |
| `screens_mod.rs`    | `src/screens/mod.rs`          | **New** |
| `screens_title.rs`  | `src/screens/title.rs`        | **New** |
| `screens_pause.rs`  | `src/screens/pause.rs`        | **New** |
| `screens_battle.rs` | `src/screens/battle.rs`       | **New** |
| `screens_end.rs`    | `src/screens/end.rs`          | **New** |
| `battle.rs`         | `src/battle.rs`               | Replace |
| `spawner_mod.rs`    | `src/spawner/mod.rs`          | Replace |
| `player_input.rs`   | `src/systems/player_input.rs` | Replace |

**`main.rs` went from 1,693 lines to 346.** It now holds only: the prelude, `State`'s struct definition, its core lifecycle (`new`/`start_game`/`return_to_title`/`advance_level`), the `GameState::tick` dispatcher, and `main()`. Everything else moved into a `screens/` folder (mirroring `systems/`/`map_builder/`) plus a flat `render_helpers.rs` for the free drawing/text functions.

**Two visibility corrections along the way**, worth knowing about since they contradict what I told you last time:
- I initially said `State`'s private fields would just work from new submodules — that part's true (Rust: private items are visible to *descendant* modules).
- But the *screen methods themselves* (`title_screen`, `battle_tick`, `game_over`, etc.) needed to be marked `pub`, since `main.rs`'s `tick()` dispatcher is calling *into* those submodules — and privacy doesn't flow the other direction. I caught this while actually writing the code, not before, so I'm flagging it plainly rather than pretending I had it right from the start.

**Three bonus fixes found while moving things around**, all cosmetic/doc-only, no behavior change:
1. A doc comment for `draw_battle_arena` had gotten glued onto `draw_end_screen_background` by mistake in an earlier edit — split back apart correctly.
2. Two comments in `battle.rs` and one in `player_input.rs` still said "main.rs" for things that just moved — updated to point at `screens/battle.rs`.
3. A **pre-existing, unrelated** stale comment in `spawner/mod.rs` still said "hidden Backslash shortcut" from before we swapped that to the `D` key — fixed.

I verified brace balance on every touched file and traced every cross-file `self.method()` call by hand to confirm it resolves under Rust's actual privacy rules. I can't compile this myself, so a real build is still the first thing worth doing — but structurally, this should be a clean, behavior-identical reorganization.

### Errors
```rust
cargo run --release
   Compiling ever_space_rrpg v0.1.0 (/home/vinny/Projects/roguelike_rpg/ever_space_rrpg)
warning: `draw_hollow_box` is ambiguous
   --> src/screens/battle.rs:305:9
    |
305 |         draw_hollow_box(
    |         ^^^^^^^^^^^^^^^ ambiguous name
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #114095 <https://github.com/rust-lang/rust/issues/114095>
    = note: ambiguous because of multiple glob imports of a name in the same module
note: `draw_hollow_box` could refer to the function imported here
   --> src/main.rs:15:13
    |
 15 |     pub use bracket_lib::prelude::*;
    |             ^^^^^^^^^^^^^^^^^^^^^^^
    = help: consider adding an explicit import of `draw_hollow_box` to disambiguate
note: `draw_hollow_box` could also refer to the function imported here
   --> src/main.rs:67:13
    |
 67 |     pub use crate::render_helpers::*;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^
    = help: consider adding an explicit import of `draw_hollow_box` to disambiguate
    = note: `#[warn(ambiguous_glob_imports)]` (part of `#[warn(future_incompatible)]`) on by default

warning: `draw_hollow_box` is ambiguous
   --> src/screens/battle.rs:383:9
    |
383 |         draw_hollow_box(
    |         ^^^^^^^^^^^^^^^ ambiguous name
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #114095 <https://github.com/rust-lang/rust/issues/114095>
    = note: ambiguous because of multiple glob imports of a name in the same module
note: `draw_hollow_box` could refer to the function imported here
   --> src/main.rs:15:13
    |
 15 |     pub use bracket_lib::prelude::*;
    |             ^^^^^^^^^^^^^^^^^^^^^^^
    = help: consider adding an explicit import of `draw_hollow_box` to disambiguate
note: `draw_hollow_box` could also refer to the function imported here
   --> src/main.rs:67:13
    |
 67 |     pub use crate::render_helpers::*;
    |             ^^^^^^^^^^^^^^^^^^^^^^^^
    = help: consider adding an explicit import of `draw_hollow_box` to disambiguate

error[E0277]: the trait bound `object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>: bracket_lib::prelude::Console` is not satisfied
   --> src/screens/battle.rs:306:13
    |
306 |             &mut log_batch,
    |             ^^^^^^^^^^^^^^ the trait `bracket_lib::prelude::Console` is not implemented for `object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>`
    |
    = help: the following other types implement trait `bracket_lib::prelude::Console`:
              FlexiConsole
              SimpleConsole
              SparseConsole
              SpriteConsole
              VirtualConsole
    = note: required for the cast from `&mut object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>` to `&mut dyn bracket_lib::prelude::Console`

error[E0061]: this function takes 7 arguments but 6 arguments were supplied
   --> src/screens/battle.rs:305:9
    |
305 |           draw_hollow_box(
    |  _________^^^^^^^^^^^^^^^-
306 | |             &mut log_batch,
307 | |             MSG_BOX_X,
308 | |             MSG_BOX_Y,
...   |
311 | |             ColorPair::new(WHITE, BLACK),
    | |             ---------------------------- expected `RGBA`, found `ColorPair`
312 | |         );
    | |_________- argument #7 of type `bracket_lib::prelude::RGBA` is missing
    |
note: function defined here
   --> /home/vinny/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bracket-terminal-0.8.7/src/consoles/text/gui_helpers.rs:41:8
    |
 41 | pub fn draw_hollow_box(
    |        ^^^^^^^^^^^^^^^
help: provide the argument
    |
305 ~         draw_hollow_box(
306 +             &mut log_batch,
307 +             MSG_BOX_X,
308 +             MSG_BOX_Y,
309 +             MSG_BOX_WIDTH,
310 +             MSG_BOX_HEIGHT,
311 +             /* bracket_lib::prelude::RGBA */,
312 +             /* bracket_lib::prelude::RGBA */,
313 ~         );
    |

error[E0277]: the trait bound `object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>: bracket_lib::prelude::Console` is not satisfied
   --> src/screens/battle.rs:384:13
    |
384 |             &mut menu_batch,
    |             ^^^^^^^^^^^^^^^ the trait `bracket_lib::prelude::Console` is not implemented for `object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>`
    |
    = help: the following other types implement trait `bracket_lib::prelude::Console`:
              FlexiConsole
              SimpleConsole
              SparseConsole
              SpriteConsole
              VirtualConsole
    = note: required for the cast from `&mut object_pool::Reusable<'_, bracket_lib::prelude::DrawBatch>` to `&mut dyn bracket_lib::prelude::Console`

error[E0061]: this function takes 7 arguments but 6 arguments were supplied
   --> src/screens/battle.rs:383:9
    |
383 |           draw_hollow_box(
    |  _________^^^^^^^^^^^^^^^-
384 | |             &mut menu_batch,
385 | |             BOX_X,
386 | |             box_y,
...   |
389 | |             ColorPair::new(GREEN, BLACK),
    | |             ---------------------------- expected `RGBA`, found `ColorPair`
390 | |         );
    | |_________- argument #7 of type `bracket_lib::prelude::RGBA` is missing
    |
note: function defined here
   --> /home/vinny/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bracket-terminal-0.8.7/src/consoles/text/gui_helpers.rs:41:8
    |
 41 | pub fn draw_hollow_box(
    |        ^^^^^^^^^^^^^^^
help: provide the argument
    |
383 ~         draw_hollow_box(
384 +             &mut menu_batch,
385 +             BOX_X,
386 +             box_y,
387 +             BOX_WIDTH,
388 +             box_height,
389 +             /* bracket_lib::prelude::RGBA */,
390 +             /* bracket_lib::prelude::RGBA */,
391 ~         );
    |

Some errors have detailed explanations: E0061, E0277.
For more information about an error, try `rustc --explain E0061`.
warning: `ever_space_rrpg` (bin "ever_space_rrpg") generated 2 warnings
error: could not compile `ever_space_rrpg` (bin "ever_space_rrpg") due to 4 previous errors; 4 warnings emitted
```

Found it — this is a real, straightforward bug from the refactor, not a guess. `bracket_lib::prelude` **already has its own `draw_hollow_box`** function (different signature — takes `&mut dyn Console` and 7 args including two separate `RGBA` colors, not a `DrawBatch` + `ColorPair`). Before the refactor, my `draw_hollow_box` was defined directly in `main.rs`, so Rust's "local definition wins over glob imports" rule silently favored mine. Once it moved into `render_helpers.rs` and got glob-imported into the same prelude alongside `bracket_lib::prelude::*`, both are now just glob imports — ambiguous, and the compiler picked bracket-lib's version, which explains every error here.

The real fix is to rename mine so it can never collide again:Now update both call sites in `screens/battle.rs`:Balanced. Only `draw_hollow_box` collided (the compiler would've flagged every ambiguous name in the same pass if there were others, and it didn't), so this should be the complete fix.**2 files — both modified: `src/render_helpers.rs`, `src/screens/battle.rs`**

Root cause, plainly: `bracket_lib::prelude` already ships its own `draw_hollow_box` (a completely different function — different args, different console type). My own `draw_hollow_box` used to live directly in `main.rs`, where Rust's rule that a local definition beats a same-named glob import kept it working invisibly. Moving it into `render_helpers.rs` and pulling it in via `pub use crate::render_helpers::*;` put it on equal footing with bracket-lib's version in the same glob-import scope — so the compiler could no longer silently prefer mine, and (per the errors) picked bracket-lib's instead, which explains every symptom: wrong arg count, `ColorPair` where `RGBA` was expected, `DrawBatch` not implementing `Console`.

Renamed it to `draw_ascii_box` everywhere — permanent fix, since two identically-named functions can't collide if they're not actually the same name anymore. No other function in `render_helpers.rs` triggered this (the compiler would've flagged every ambiguous name in the same pass, not just one), so this should be the complete fix.


# Amazon
Glyph - A
Weapons Icons - w, x, y

Attack - 1 
Defense - 0
Speed - 6
Evasion - 5%
Health - 12

Abilities
Spear - Outside battle attack that will target the nearest enemy and do attack + weapon dmg + 2 dmg
Battle Cry - For the next 3 turns enemies will do 1-2 less damage
Trap - Used outside combat will place a trap and if an enemy runs over it will instantly take 5 damage. Icon will be T
Poison Spear - Will do 1 dmg then 2 dmg for the next 3 turns

Starting Items
Trap x1, potion x1, Battle Cry x1

## Feedback
Good — that's simple: this project's FOV is already computed via real shadowcasting (`field_of_view_set`, blocked by walls), so a target being in `visible_tiles` already *is* proper line-of-sight. One check covers both.

Still need the two Battle Cry answers before I start — they decide whether it's Technique-system code or Effect-system code, which is a real fork in the implementation, not a style choice:

```
Battle Cry - used during your turn in a fight (like Rogue's Dodge), or used from the dungeon item list before a fight starts (like Mage's Ice Armor)?
In-battle technique
Battle Cry's damage reduction - random 1-2 each hit, or a flat number to match every other numeric effect in the project?
True random 1-2 each hit
```

That confirms it — `player_input.rs:23` already uses `.filter(component::<Enemy>())`, the exact same idiom I used in both `traps.rs` and the Throw Spear targeting code. Every pattern in this delivery is now traced back to proven prior art in this codebase, not a guess. Packaging everything.Amazon is fully built. **9 files — 1 new, 8 modified:**

| Downloaded as       | Save to                       | Action  |
| ------------------- | ----------------------------- | ------- |
| `components.rs`     | `src/components.rs`           | Replace |
| `battle.rs`         | `src/battle.rs`               | Replace |
| `use_items.rs`      | `src/systems/use_items.rs`    | Replace |
| `traps.rs`          | `src/systems/traps.rs`        | **New** |
| `systems_mod.rs`    | `src/systems/mod.rs`          | Replace |
| `spawner_mod.rs`    | `src/spawner/mod.rs`          | Replace |
| `screens_title.rs`  | `src/screens/title.rs`        | Replace |
| `template.ron`      | `resources/template.ron`      | Replace |
| `starting_kits.ron` | `resources/starting_kits.ron` | Replace |

**Summary of the kit:**
- **Stats:** 12 HP, 1 dmg, 6 speed, 0 defense, 5% evasion, glyph `A`, weapon tier Wooden/Bronze/Iron Spear (`w`/`x`/`y`, dmg 1/2/3).
- **Throw Spear** (out-of-combat) and **Trap** (out-of-combat) — both genuinely new mechanics for this project: the first attack that damages an enemy without ever starting a battle, the second a placeable, persistent hazard entity with its own dedicated system.
- **Battle Cry** and **Poison Spear** (in-battle techniques) — Battle Cry is the first *randomized* effect in the project (every other numeric effect is flat, per your call); Poison Spear composes an instant hit with the existing Rend/Burn-style damage-over-time machinery.
- **Starting kit:** Healing Potion, Trap, Battle Cry. Throw Spear and Poison Spear are loot-only, matching how every other class's specialty items work.

I traced every new pattern back to something already proven elsewhere in this codebase before using it — the `component::<Enemy>()` filter, the two-phase "collect then apply" pattern for anything needing a `Health` mutation, the `Ok(&x)` copy-out idiom — so nothing here is a first-time guess the way the `set_fancy` rotation work earlier had to be. That said, I still can't compile this myself, so a real build is the next step, especially given how much new ground (targeting, traps, randomized combat math) this kit covers compared to Rogue/Mage.

### Errors
```rust
error[E0277]: can't compare `&legion::Entity` with `legion::Entity`
   --> src/systems/use_items.rs:136:63
    |
136 | ...                   .find(|(e, _)| *e == activate.used_by)
    |                                         ^^ no implementation for `&legion::Entity == legion::Entity`
    |
    = help: the trait `PartialEq<legion::Entity>` is not implemented for `&legion::Entity`
help: consider dereferencing here
    |
136 |                                             .find(|(e, _)| **e == activate.used_by)
    |                                                            +

For more information about this error, try `rustc --explain E0277`.
error: could not compile `ever_space_rrpg` (bin "ever_space_rrpg") due to 1 previous error
```

Simple, exact fix — matches the compiler's own suggestion, and `**e` is already the established idiom elsewhere in this codebase for this exact situation (e.g. the counter-attack item-filtering code uses the same double-deref).Only the one instance — fixed, balanced.**1 file — modified: `src/systems/use_items.rs`**

Legion's query iterator was yielding an extra layer of reference than my single `*e` accounted for — `**e` is exactly the idiom this codebase already uses elsewhere for the same pattern (the counter-attack item filtering). Rebuild should get past this one.

# Bosses for Each Level
Now I have everything needed. Let's build it.

### 1. `components.rs` — add `Boss` marker### 2. `spawner/template.rs` — `boss_only` field, `spawn_entity` returns `Entity`, new `spawn_boss`Now let's view `spawn_prefab_enemies` fully once more (exact end boundary) to insert `spawn_boss` right after it, and update `spawn_entity`'s return type:Good catch needed here — I should also exclude `boss_only` templates from the **general ambient pool** and the **prefab-guard pool**, or a boss could randomly show up as an ordinary floor monster instead of only guarding the stairs. Let's fix both filters while I'm in here:Now update `spawn_prefab_enemies`'s filter and add `spawn_boss` right after it:Now let's update `spawn_entity` to return the `Entity`:Both statement-form calls (lines 89, 125, 215) are unaffected — discarding a return value as a bare statement needs no changes. Now let's add the wrapper function in `spawner/mod.rs`:### 3. `main.rs` — wire `spawn_boss` into `start_game` and `advance_level`Now `advance_level`:Now let's add three placeholder boss templates — one per level — so the placement mechanism is actually testable before we tune real stats next round. I'll flag them clearly as rough stand-ins.No collisions — every glyph appears exactly once except `?` (which is intentionally shared by all battle techniques, since they're never floor-spawned). Now let's do a full sanity sweep across every touched file:All balanced. Let's do one last cross-check — confirm `spawn_boss` is called correctly and `Boss`/`boss_only` are wired end-to-end with no orphaned references:Everything is wired end-to-end — two call sites (level 0 in `start_game`, every level in `advance_level`), the `Boss` marker is applied correctly, and `boss_only` is respected by all three spawn pools (ambient, prefab-guard, and the new boss pool itself). Let's ship it.Boss placement is fully wired up. **5 files — all modified:**

| Downloaded as         | Save to                   |
| --------------------- | ------------------------- |
| `components.rs`       | `src/components.rs`       |
| `spawner_template.rs` | `src/spawner/template.rs` |
| `spawner_mod.rs`      | `src/spawner/mod.rs`      |
| `main.rs`             | `src/main.rs`             |
| `template.ron`        | `resources/template.ron`  |

**How it works:**

- **Where they go:** every level already computes an `amulet_start` point — the furthest reachable tile from the player's spawn, already reused as either the Exit tile (levels 0/1) or the Amulet's own position (level 2). A boss now spawns *exactly there* on every level, via a new `spawn_boss` function mirroring the existing `spawn_prefab_enemies`/`spawn_prefab_weapon` pattern.
- **Why that guarantees a real fight:** this project already blocks the player from walking onto any tile an enemy occupies — it starts a battle instead. So standing a boss directly on the Exit/Amulet tile means it's genuinely unavoidable: win the fight, and only then can you step onto the stairs or claim the Amulet.
- **New `Boss` marker component** tags these entities for anything later (rendering, AI, HUD) that wants to treat them differently — doesn't change any mechanics yet.
- **New `boss_only` template flag** — I made sure this is excluded from *both* the general ambient floor-spawn pool *and* the fortress prefab-guard pool, not just added as a new pool. Otherwise a boss could've randomly turned up as an ordinary monster instead of only guarding its post.
- **Testable now:** I added three clearly-flagged **placeholder** bosses (Goblin Chieftain / Orc Warlord / Ettin Overlord, one per level, roughly scaled above that level's normal enemies) so you can actually see the placement working on a build, rather than shipping the mechanism with nothing to spawn. These are explicitly not the real stats — that's next.

One small structural note: I changed the private `spawn_entity` helper to return the `Entity` it just created (previously `()`), since `spawn_boss` needs it to attach the `Boss` tag afterward. The other three call sites just ignore the new return value, which Rust allows without any warning — nothing else needed to change.

## Bosses Will Guarantee Loot
Now update the `spawner/mod.rs` wrapper:Now update all 4 call sites in `screens/battle.rs`:Now let's fix all 4 call sites — the two single-line ones and two multi-line ones:Now the two multi-line ones:All 4 call sites updated correctly, `battle.enemy` still valid at each point (removed only afterward). Let's confirm balance across every touched file:That paren "mismatch" is worth checking honestly rather than waving off — let me verify it's pre-existing prose (like the `main.rs` false-alarms earlier) and not something I introduced:That confirms it — my edits added zero net parentheses (I only inserted `battle.enemy`/`battle.enemy,`, no parens), and there are plenty of comment lines with parenthetical asides that don't close on the same line (e.g. `(see`, `(a`, `(console 0:`). This is the same benign "prose comments contain unbalanced parens" pattern we already ran into with `main.rs` earlier in this project — not a real syntax problem, just a limitation of naive whole-file paren counting. Braces (the ones that actually matter for Rust block structure) are balanced. Let's do the final cross-check and ship.Every signature and every call site agrees. Let's package the delivery.**3 files — all modified:**

| Downloaded as         | Save to                   |
| --------------------- | ------------------------- |
| `spawner_template.rs` | `src/spawner/template.rs` |
| `spawner_mod.rs`      | `src/spawner/mod.rs`      |
| `screens_battle.rs`   | `src/screens/battle.rs`   |

**What changed:** normal enemies still roll a 40% chance to drop loot (`BATTLE_LOOT_DROP_CHANCE_PERCENT`, unchanged). `grant_random_battle_loot` now also takes the defeated enemy's `Entity`, checks it for the `Boss` marker component, and if it's a boss, skips that roll entirely — loot is guaranteed. Same candidate pool as before (any item template matching the player's class), just no chance of coming up empty.

I updated all 4 places in `screens/battle.rs` that call this — every path an enemy can die through (a normal Attack, a Technique, a DOT tick, or a Counter Attack triggered off the enemy's own attack) all call it before the enemy entity is actually removed, so `battle.enemy` is still valid to check in every case. Confirmed the signature and every call site agree on argument order and count.

# Icon Mapping
The original file is still here — let's work from that directly rather than reconstructing from fragments.Yes, this all makes sense. Let me work from the actual file directly — the uploads folder is read-only, so I'll copy it to a writable location first, then make precise edits rather than risk retyping the whole 256-row table by hand.### 1. Update the top summary table — add bosses + Trap### 2. Remove codes 0–31 from the "Important custom sprite cells" table (they were never actually custom)### 3. Fix the `s` and `o` errors, add `T` (Trap), add the new boss glyphs to this tableLet me fix the row ordering — I inserted them out of numeric sequence:Now let's update the "Complete 256-cell map" section — codes 0–31 marked Unused, and the `s`/`o`/`T`/`G`/`K`/`V` corrections applied there too:Good — line 86 onward is the "Complete 256-cell map" table (7-column format, different from the curated table above). Let's find where codes 0–31 end and 32 begins there:Now let's replace these 32 rows with a compact "Unused" version, using a Python script for a clean, guaranteed-correct bulk edit:All 32 rows converted correctly, kept row/col/pixel data, dropped the pictorial noise. Now let's fix `s`/`o`, add `T`/`G`/`K`/`V` in the Complete 256-cell map too, and update the Boss glyph status section:Found all six spots in the Complete 256-cell map (lines 157, 161, 170, 172, 197, 201). Let's fix each:Now let's update the "Boss glyph status" section to reflect the final, finished state:Let's do a final consistency check on the whole file before delivering it:Everything's consistent — `G`/`K`/`V`/`T` appear correctly in all three places they're tracked, the `s`/`o` corrections are applied everywhere, and no stray "NOT a target" or "current dagger sprite" text remains. The Dagger tier's "planned dagger replacement" notes are untouched deliberately — that's tracking *art* status, not code status (same convention the doc already used for Rogue/Mage/Amazon, who are fully built in code but still awaiting custom art too), so that phrasing is still accurate as-is.**1 file — replaces `Dungeon_Font_Glyph_to_Cell_Map.md`**

**What changed:**

1. **Bosses + Trap finalized** — `G`/`K`/`V`/`T` added to the top summary table and the exhaustive 256-cell map, each marked "custom art pending" (assigned in code, artwork not drawn yet — same convention the sheet already used elsewhere).
2. **Two errors in the sheet itself corrected** — `s` now correctly says "current sword sprite" (was wrongly "dagger"), `o` now says "current Orc sprite" (the old "NOT a target" note was wrong).
3. **`Boss glyph status` section rewritten** — all four glyphs marked finalized, explains *why* `W`/`X` got dropped (so future-you or anyone else reading this doc understands the reasoning, not just the outcome), and notes `W`/`X` are free again but `X`/`Y`/`Z` (uppercase) should stay avoided since they still hold leftover spear artwork.
4. **Codes 0–31 (the CP437 icons/arrows) marked `Unused`** — removed entirely from the curated "Important custom sprite cells" table (they were never actually customized, so they didn't belong in a table about customized cells), and in the exhaustive 256-cell map their pictorial descriptions were replaced with a plain `Unused` so the row/col/pixel data is still there for reference but the clutter's gone.

I worked directly from your original uploaded file rather than reconstructing it from the chat excerpts, so nothing in the untouched ~250 rows in between should have drifted.

# Battle Cry Attacks Left
Good, there's already an established pattern for exactly this. Let's see the full status-display block:This is a well-established pattern — one combined status line, built from a list of active player statuses. Battle Cry just needs to join that list. Let's add it:**1 file — modified: `src/screens/battle.rs`**

Turns out this didn't need new UI plumbing — the battle screen already had a "combined status line" mechanism showing things like `Defending`, `Ice Armor (N left)`, and `Countering` whenever they're active. Battle Cry just needed one line joining that same list: `"Battle Cry (N left)"`, reading `battle.war_cry`'s `attacks_remaining` directly, same pattern as Ice Armor.

One thing I noticed while in there, not fixed since it's outside today's ask: Dodge (Rogue's Evade technique) has no status line at all right now — it's tracked in state but never shown. Let me know if you want that added too while we're in this area.

# Modify Defend Action
Good, there's a clear convention (module-level `pub const ..._PERCENT: i32`). I'll follow it.Good — `rng` is already declared as `mut` and in scope at that point (used for the evasion roll above), so this reuses it cleanly rather than shadowing a second RNG right next to it, matching how the file already re-declares fresh RNGs per independent roll elsewhere (e.g. Battle Cry, a few lines below).

Let me sanity-check the rest of the file didn't need touching and that this compiles conceptually (types/borrow-wise):That all lines up — `RandomNumberGenerator` comes through the crate's `prelude` glob, same as everywhere else it's used in this file, and there's no naming collision since my edit reuses the existing `rng` rather than declaring a new one.

That's the Defend change, done:

- Added `DEFEND_SUCCESS_CHANCE_PERCENT: i32 = 30` alongside the other named constants at the top of `battle.rs`, following the exact pattern `BATTLE_LOOT_DROP_CHANCE_PERCENT` already set in `spawner/template.rs`.
- In `resolve_enemy_attack`, the guaranteed `dmg = (dmg / 2).max(1)` became a 30%-chance roll using the RNG that was already live from the evasion check just above it (deliberately not sharing that same roll — dodge and defend are different mechanics and shouldn't succeed/fail together), and on success just halves with integer division (already rounds down) and no floor, per your call.

One thing worth flagging while it's fresh: right now a failed Defend roll (the 70% case) is silent — the log line still just says `"Take N damage."` with nothing distinguishing "you defended and it didn't help" from "you never defended." If you want, I can add a log variant for that so a Defend that whiffs doesn't feel indistinguishable from not defending at all — small addition, your call whether it's worth it.

---

Now, the bigger question — what I'd do with `battle.rs` given everything that's come before. A few honest observations from reading the whole file, not just a wishlist:

**It's starting to show its age structurally.** `resolve_enemy_attack` (where I just edited) is the single place every enemy hit resolves, and it's grown one more `if let Some(x) = &mut battle.x { ... }` block per mechanic — Dodge, Defend, Ice Armor, Battle Cry, Counter — each with its own tick-down/consume logic inlined. That's exactly how the class kits got built incrementally, and it's readable today at 5 mechanics, but it's the same shape of growth that made `main.rs` need its ~1,700→350 line refactor. If Archer or a 6th class adds another defensive status, I'd suggest extracting these into a small ordered "damage pipeline" — a `Vec` of reduction steps applied in sequence with a shared trace for the log — rather than one more inline block. Not urgent, but worth deciding on *before* the next status gets added, not after three more pile up.

**Two systems live oddly split.** `battle.rs` (851 lines, pure data/logic) and `screens/battle.rs` (668 lines, rendering + tick dispatch) is a good separation in principle, but a few things (like the status-line list I saw Dodge/Battle Cry just get added to) live entirely in the screen file even though they're really "what is currently true about this battle," which feels like `Battle`-struct data, not render logic. Minor, but if we're touching statuses again for Archer, might be worth moving that list-building into `battle.rs` as a method the screen just calls.

**Concretely, on the "what to add" side**, given the project's own stated priorities:
- **Archer** is the most obvious next `battle.rs` touch — it needs real `CanAttack`/technique entries wired the same way Amazon/Mage were, which is now a well-worn path (technique enum variant → one match arm in `apply_player_technique` → template.ron entries → starting kit).
- **Enemy AI variety** — right now `resolve_enemy_attack` is explicitly "enemies only ever know Attack." Boss stats are still flagged placeholder in the journal; giving bosses an actual decision (defend sometimes, use a technique-like ability) would touch this same function and is probably the highest-impact single change to `battle.rs` specifically, since it's currently the flattest part of an otherwise deep combat system.

Let me know which direction to go once you're back — happy to start sketching the damage-pipeline refactor, Archer's kit, or boss AI, whichever you want first.

# Fluid Motion Returned
Good — I've got a clear, verified picture now. Here's what's actually true and what I think we should do.

## What I found

`MovingAnimation` already exists — start/end points, elapsed time, ticked by real frame time — but it's **never actually used to draw anything**. `entity_render` draws every entity with `draw_batch.set(*pos - offset, ...)`, straight to the integer tile grid, completely ignoring `MovingAnimation`. There's even a commented-out `animated_position()`/`lerp`/`ease_out_cubic` stub sitting right next to the struct, unused. So right now, movement in the dungeon view is an instant snap — the timing machinery for a glide exists, but nothing draws the glide itself.

That stub was shelved for the same reason the battle-portrait jiggle got shelved: sub-pixel movement needs bracket-lib's "fancy console," and a fancy console always paints an opaque background quad. A moving glyph on an opaque box, sliding over a static background, shows a visible edge the instant it's not perfectly aligned — which is exactly the wall you hit with the GameOver portrait too, before landing on `RGBA` alpha=0.

## Why this changes things

Console 6 — the fancy console built for the fallen-hero portrait — already uses the **same grid dimensions and the same 32×32px cells as console 0/1** (the dungeon view), specifically so a rotated glyph doesn't stretch. That also means it's positioned in the same coordinate space as the dungeon tiles, and fancy-console positions aren't snapped to the grid the way `draw_portrait`'s are — which is precisely the sub-pixel freedom the shelved `animated_position` stub needed. Combine that with the now-proven transparent background, and the actual blocker from before is gone.

## What I'd propose

- Wire the commented-out `lerp`/`ease_out_cubic`/`animated_position` code back in (it already computes the eased fractional position — this part barely changes).
- Add a small fancy-console pass, alongside `entity_render`, that: for any entity currently carrying a `MovingAnimation`, draws it at its eased fractional position on a fancy console with a transparent background — and skip drawing that entity on the normal console 1 for that frame so it isn't double-drawn.
- Reuse console 6, or register a new one — console 6 is currently only active during GameOver, never during gameplay, so timing never overlaps, but a dedicated console is cleaner and less likely to bite us later if GameOver's usage ever changes. I'd lean toward a new console rather than overloading 6's meaning.

**Real uncertainty I want to flag up front, not discover mid-edit**: everything above is inferred from how console 6 behaves for a single static icon drawn once. I haven't confirmed `set_fancy` behaves the same way when called many times per frame for multiple simultaneously-moving entities (enemies chasing the player, say) — that's untested territory, both for the API and for performance.

Want me to go ahead and build this?

## Outcome
Good — `MovingAnimation` is already correctly attached with real `start`/`end` on every move. Now let me check the systems schedule wiring, the exact `Point`/`PointF` types available, and the `main.rs` builder/tick blocks in full detail.Now let's look at the `tick()` clearing block and the `BTermBuilder` chain in full detail before editing.Good — no collisions, safe to activate. Now let's make the edits.Now the console constant and doc-comment updates in `main.rs`:Now the `tick()` clear block:Now the `BTermBuilder` chain in `main()`:Now the core of it — `entity_render.rs`:Now let's do a full sanity sweep — check for anything that might reference `MOVE_ANIM_DURATION_MS` semantics differently, verify no orphaned references, and confirm nothing else draws entities and would need the same treatment (title-screen background, which also uses `entity_render`).Good — since `entity_render` is the single shared system used across every schedule (input, player, monster, and the title-screen background), this change automatically covers the decorative title-screen wandering enemies too, with no extra wiring needed there.Good, `camera.left_x`/`top_y` are already used as `i32` in the existing offset math, so `offset.x as f32` is consistent. Let me do one last check — that `gliding_position` is reachable from `entity_render.rs` (both go through `crate::prelude::*`, and `components::*` is re-exported there).That resolves cleanly. Let's copy the changed files out.**Three files, all modified:**

| File                                                              | What changed                                                                                                                                                                                         |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/systems/entity_render.rs` (→ `src/systems/entity_render.rs`) | Any entity mid-`MovingAnimation` now draws via `set_fancy` on a new fancy console with a transparent background, at its eased sub-pixel position, instead of snapping to console 1's integer grid    |
| `src/components.rs`                                               | Uncommented `lerp`/`ease_out_cubic`; replaced the old commented-out `animated_position` stub with `gliding_position`, which returns eased fractional coordinates (or `None` once a glide's finished) |
| `src/main.rs`                                                     | New `GLIDE_CONSOLE` constant (console 7), a second fancy console registered in the `BTermBuilder` chain, and one more `cls()` in `tick()`'s per-frame clear block                                    |

**How it works:** `movement.rs` already attached `MovingAnimation { start, end, elapsed_ms }` correctly on every move — that part was already right, it just wasn't being read by anything. Now `entity_render` checks each entity for an in-flight glide; if present, it draws the eased position on the new transparent fancy console (console 7) instead of the plain one, and skips drawing it on console 1 for that frame, so there's no double-draw. Since `entity_render` is the one shared system used by every schedule, this also automatically applies to enemies chasing the player and the title screen's decorative wandering background — no separate wiring needed there.

**One deliberate caution, flagged honestly:** rather than reading `.x`/`.y` back off a `PointF` (never proven anywhere in this codebase — only construction of one has ever been witnessed, in the fallen-portrait code), `gliding_position` returns a plain `(f32, f32)` tuple, and `entity_render` builds the final `PointF` itself after subtracting the camera offset. Slightly more verbose, but it avoids leaning on an unconfirmed part of the API on top of the genuinely new one (`set_fancy` called every frame, potentially for several entities at once, which — as noted last time — has also never been proven here).

Build it and see what happens — first real test of both of those unknowns at once.

### Errors
```rust
   Compiling ever_space_rrpg v0.1.0 (/home/vinny/Projects/roguelike_rpg/ever_space_rrpg)
error[E0308]: `match` arms have incompatible types
  --> src/systems/entity_render.rs:60:25
   |
42 |               match gliding_position(ecs, *entity) {
   |               ------------------------------------ `match` arms have incompatible types
...
51 | /                     glide_batch.set_fancy(
52 | |                         draw_pos,
53 | |                         0,
54 | |                         Degrees::new(0.0),
...  |
57 | |                         render.glyph,
58 | |                     );
   | |                      -
   | |                      |
   | |______________________help: consider removing this semicolon
   |                        this is found to be of type `()`
59 |                   }
60 |                   None => draw_batch.set(*pos - offset, color, render.glyph),
   |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `()`, found `&mut DrawBatch`

For more information about this error, try `rustc --explain E0308`.
error: could not compile `ever_space_rrpg` (bin "ever_space_rrpg") due to 1 previous error
```

Good catch — `DrawBatch::set` returns `&mut DrawBatch` (it's a chainable builder), unlike `set_fancy` which the compiler inferred as `()` because of the trailing semicolon in the other arm. Easy fix: make the `None` arm a block that also discards its return value, so both arms agree on type `()`.That's the whole fix — both arms now end in a statement (trailing `;`), so both evaluate to `()` and the `match` type-checks. Nothing else in the file changes.**Modified:** `src/systems/entity_render.rs` — replaces the previous version. Only the `None` arm changed, wrapped in `{ ...; }` so it discards `DrawBatch::set`'s return value instead of implicitly returning it, matching the `Some` arm's `()` type. Try the build again.

## Single Cell Offset
That confirms it cleanly — a constant, direction-independent, non-cumulative one-cell vertical offset is exactly what you'd get from `set_fancy` anchoring the glyph's Y position differently than `set` does (bottom-of-cell vs. top-of-cell), not from anything in the movement/lerp/camera math. Simple, targeted fix: compensate by exactly one cell on the Y axis when building the fancy-console draw position.That's the whole fix. Let's ship it.**Modified:** `src/systems/entity_render.rs` — replaces the previous version. Two small additions, no other logic touched:

- A new `GLIDE_CONSOLE_Y_ANCHOR_OFFSET: f32 = 1.0` constant, with a comment explaining it's an empirical correction confirmed by your testing, not something derived from bracket-lib docs we don't have.
- `draw_pos`'s Y calculation now adds that offset: `fy - offset.y as f32 + GLIDE_CONSOLE_Y_ANCHOR_OFFSET`.

Give it a run — if it lands exactly right, great; if it overshoots the other direction or is still off by some amount, that'll actually tell us something new (e.g. if it's now exactly one cell too far *south*, the anchor theory would be wrong in direction and we'd want `-1.0` instead, or if it's now off by half a cell, the anchor is center-based rather than bottom-based). Let me know what you see.

## Too Quick to Move to the Next Cell
Good — confirmed everything timed in this project (glide duration, background wandering pace, battle flashes, damage popups, result auto-advance) is already driven by real elapsed milliseconds, never raw frame counts. That means raising the FPS cap is safe — nothing will suddenly speed up or misbehave, it just gets more frames to spread the same durations across.

That points at the actual cause of the jumpiness: `.with_fps_cap(30.0)` combined with a 150ms glide means each move only gets **~4–5 frames total** to animate across — not enough to read as smooth motion, no matter how good the easing curve is. Two independent, complementary levers:Now let's also give the glide a bit more real time to play out, which helps independently of frame rate and gives the eased curve more room to actually read as a curve:**Two files, both modified:**

| File                | Change                                        |
| ------------------- | --------------------------------------------- |
| `src/main.rs`       | FPS cap raised from 30 → 60                   |
| `src/components.rs` | `MOVE_ANIM_DURATION_MS` raised from 150 → 220 |

Together these give each glide roughly **3x the frames** it had before (~4–5 → ~13), which is the real fix for jumpiness — this isn't a speed change to the movement itself (a move still completes in a bit over 0.2s either way), it's giving the same motion enough frames to actually look continuous. I confirmed first that every timed system in the project (flashes, popups, background wandering, result auto-advance) already scales off real elapsed time rather than frame counts, so raising the frame rate doesn't speed anything else up unexpectedly.

If it's still not smooth enough after this, the next knob to reach for is `MOVE_ANIM_DURATION_MS` again (push it toward 250–300ms) rather than the FPS cap — going past 60fps starts trading off against your GPU/CPU headroom for diminishing visual return.

## Jumping Behavior
Good — this file is correctly in place, matching what I last shipped. Now let's check the other two, and specifically the schedules and camera code, since the pattern you're describing (jump toward destination, snap back to origin, then glide) is a very specific timing signature.That "always, every time, same direction" consistency is the key clue — it rules out something probabilistic like frame-time jitter and points at something structural in how the ECS handles a **newly-added** component. Let me check how `MovingAnimation` gets attached versus how it's removed, and where entities get created.That confirms the mechanism I suspected: every single glide currently does two **structural** changes to the entity — `tick_animations` removes the `MovingAnimation` component entirely when a glide finishes, and `movement.rs` re-adds it as a brand-new component on the very next move. Adding or removing a component type (as opposed to just updating one that's already there) moves the entity to a different internal archetype in the ECS — and that's exactly the kind of change that can lag a frame behind for other systems reading it in the same pass, which matches "always, every move, same direction" perfectly. It's not timing jitter (that would be inconsistent); it's a structural churn that happens identically every time.

The fix: stop treating "the glide is over" as "remove the component." Instead, just let it sit in an expired state — `gliding_position` already checks `elapsed_ms >= MOVE_ANIM_DURATION_MS` and returns `None` for exactly that case, so nothing else needs to change. Once the component only ever gets *updated* (never removed and re-added), every future move becomes an in-place overwrite instead of a structural change, and the one-frame lag goes away.This is the whole fix — one file, `src/systems/animation.rs`.**Modified:** `src/systems/animation.rs` — replaces `src/systems/animation.rs`. This is the only file this fix touches; `movement.rs`, `entity_render.rs`, and `components.rs` don't need any changes for this.

**What changed:** `tick_animations` no longer removes `MovingAnimation` when a glide finishes — it just keeps advancing `elapsed_ms` past the duration and leaves the component in place. `gliding_position` was already correctly treating `elapsed_ms >= MOVE_ANIM_DURATION_MS` as "not animating," so nothing else needed to react differently — the component just sits there inert between moves now instead of being deleted and recreated.

**One honest caveat**: this fixes every move *after* an entity's first-ever move, since from then on the component already exists and every update is a simple overwrite. The very first move any given entity makes (right when a level loads, before it's ever moved) is still a genuinely new attach, so it's theoretically still exposed to the same one-frame lag. If you notice a single hiccup specifically on your very first step of a level and nowhere else, that's the remaining edge case, and the real fix for it would be pre-seeding every spawned entity with an inert `MovingAnimation` at spawn time — a bigger, more invasive change touching several spawn functions, worth doing only if it's actually still visible in practice.

Try it and let me know what you see.

### Cont.
This pixel-level evidence changes my read of the bug entirely — let me trace exactly what the numbers say.I traced the pixel positions frame-by-frame and this points at something more fundamental than a rendering glitch — it's an architecture mismatch, and I want to walk through the evidence before proposing the fix.

Measured screen X position of the player glyph across the move:

```
frame:  19-28   29     30     31     32     33-37
x:      397.1   416.2  408.1  400.2  400.2  397.2
```

It rests at 397 before and after — that's expected, since the camera always keeps the player exactly centered. But at the instant of the move, it jumps to 416 (one full cell off, in the direction *opposite* your keypress) and smoothly glides back down to 397 over the next few frames.

That's not random — it's mathematically exactly what you'd get if: the camera **snaps instantly** to center on your new tile the moment you move (which you already told me — "the background just jumps"), while the player's own glyph then plays its glide *relative to that already-snapped camera*. Since your *starting* tile is now off to one side of the new, already-centered view, the glyph appears to leap to the wrong side first, then slide into the middle — which reads exactly like "jump to new, jump back to old, then glide."

The real problem: **the player is simultaneously the thing the camera is chasing and the thing being glide-animated**, and those two can't both be true without this kind of visible conflict. The camera can't "catch up" smoothly (that's a much bigger change — the whole map would need to scroll pixel-by-pixel, not just the avatar), but we don't need it to: since the camera guarantees the player is always exactly screen-center once a move completes, the player's own glyph gains nothing from animating — only the world around it does. Enemies don't have this conflict at all, since the camera never centers on them.

So the fix is narrower than I've been treating it: **stop animating the player's own glyph specifically, and keep the smooth glide for everything else (enemies).**Now let's add the `is_player` helper and the reasoning behind excluding the player, right above `entity_render`:That's the whole fix — one file again.**Modified:** `src/systems/entity_render.rs` — replaces the previous version. No other files changed; the `animation.rs` fix from last round stays in place (it was a real, separate improvement — avoiding needless component churn — just not the cause of this particular symptom).

**What changed:** Added `is_player()`, and the player is now unconditionally excluded from the glide path — always drawn via the plain console at its current (already-recentered) position. Every other entity (enemies) still gets the full smooth glide exactly as before.

**Why the player specifically:** I pulled actual pixel positions frame-by-frame from your video (via `ffmpeg`/pixel analysis, not guessing) and the numbers were precise: idle at x=397, jump to x=416 (one full cell, opposite your direction of travel) at the move, then a smooth decay back to 397 over the next few frames. That's the exact signature of animating an entity that the camera is *also* simultaneously recentering on — the camera locks the player to screen-center the instant a move happens, so the glyph's "start of glide" position (still the old world coordinate) necessarily reads as off-center on the wrong side once measured against the new, already-centered camera. It's not a math bug, it's two systems both trying to control the same entity's screen position at once.

The properly complete fix — making the camera itself scroll smoothly in lockstep, so the world glides under a fixed avatar instead of the avatar chasing a snapped world — is a real option but a much bigger one, since the map is currently rendered on a plain integer-grid console and would need the same sub-pixel treatment as the entity glide got. Worth keeping in mind if you want that fuller "world scrolls, character floats" feel later, but this fix should make normal movement look correct and clean right now, with enemies still gliding smoothly.

