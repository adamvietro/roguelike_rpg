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
Right now 