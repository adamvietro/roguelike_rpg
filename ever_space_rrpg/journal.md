# Getting Started
Right now I will be taking the code that I made while working through the Hands on Rust book. I created a new project with Claude and it has a full set of instructions.
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