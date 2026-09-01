# Ever Space RRPG

A roguelike RPG built in Rust on top of the "Hands-on Rust" dungeon
crawler foundation, extended with full turn-based/ATB combat, five
playable classes, a second "Battle Arena" game mode with its own gold
economy, and a growing set of animation/rendering polish passes.

This README covers the basics: what the project is, how to build and run
it, and where things currently stand. **The project instructions doc
(pasted at the start of each AI chat session working on this project) is
the authoritative, detailed log of design decisions and known issues** -
this file is a lighter-weight orientation for anyone (including future
you) opening the repo cold.

## Tech stack

- **Language:** Rust, edition `2018`
- **Rendering/engine:** [`bracket-lib`](https://github.com/amethyst/bracket-lib) (~0.8.1) - a classic roguelike toolkit (consoles, field-of-view, pathfinding, RNG)
- **ECS:** [`legion`](https://docs.rs/legion) (`=0.3.1`)
- **Data/serialization:** `serde` (`=1.0.115`), `ron` (`=0.6.1`) for the game's data files (`resources/*.ron`)
- **Dev environment:** Windows + WSL (Ubuntu), GitHub Desktop for commits

Several dependencies are pinned to exact versions (`legion`, `serde`,
`ron`) - don't bump these without checking that nothing downstream
breaks, since legion in particular has had breaking changes across
versions.

## Building and running

```bash
cargo run
```

For a release build (recommended for actually playing - debug builds of
bracket-lib's rendering path are noticeably slower):

```bash
cargo run --release
```

### Linux/WSL note

This project pins `WINIT_UNIX_BACKEND=x11` via `.cargo/config.toml`,
which is required for `cargo run` to work on WSL/WSLg. Without it,
window creation can panic inside bracket-terminal's Wayland title-bar
font rendering. This is already handled by the checked-in config file -
nothing extra to set up.

If the game window fails to appear after a WSL/driver update (taskbar
icon only, or the process hangs with no window), try a full
`wsl --shutdown` from PowerShell and reopening WSL before assuming it's a
code issue - a stuck WSLg compositor/GPU state has caused this before.

### Verifying changes without a full run

If you just need to confirm the code compiles (no window/graphics
needed), `cargo check` is faster than `cargo run`. Note the real
`Cargo.lock` is lockfile-format `version = 4`, which needs a fairly
recent Rust toolchain (1.80+) - an older toolchain will fail to even
parse it. If you're on an older Rust and just need a compile check,
work from a **scratch copy** of the project so you never touch the real
lockfile, and downgrade `rayon`/`rayon-core` there to versions that
support your toolchain (`cargo update rayon --precise <version>`, then
same for `rayon-core`) - diff the scratch lockfile's checksum against the
real one afterward to confirm you never modified it.

## Controls

| Key(s)                                | Action                                                                                                                                                     |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Arrow keys (rebindable - see Options) | Move / attack by walking into an enemy                                                                                                                     |
| `1`-`9` / `0`                         | Use item in that inventory slot (dungeon), or pick a battle menu action (in battle)                                                                        |
| `Escape`                              | Pause (in-game) / back out of a submenu                                                                                                                    |
| `O`                                   | Options (rebind movement keys) - from the title screen or the pause menu                                                                                   |
| `H`                                   | Play history / stats - from the title screen                                                                                                               |
| `I`                                   | Items Used breakdown - from the History screen                                                                                                             |
| `Q`                                   | Quit - from the pause menu                                                                                                                                 |
| `D`                                   | Hidden shortcut on Class Select: spawns the internal Debug class (overpowered stats, instant win/lose/next-level test items) - not part of the normal game |

Movement keys can be rebound to WASD, IJKL, HJKL, or similar via the
Options screen; the rebinding covers the four movement actions only
(Escape and the number-row selectors are fixed).

## Playable classes

| Class     | Style                                       | Battle techniques                             | Out-of-combat abilities    |
| --------- | ------------------------------------------- | --------------------------------------------- | -------------------------- |
| Barbarian | Hardy melee fighter                         | Deathblow, Quick Attack, Counter Attack, Rend | -                          |
| Rogue     | Fast, evasive (10% base Evasion)            | Garrote, Dodge                                | Stealth (ambush attack)    |
| Amazon    | Ranged skirmisher, spears (5% base Evasion) | Poison Spear, Battle Cry                      | Throw Spear, Trap          |
| Hunter    | Ranged fighter, bows                        | Poison Shot, Stun, Feint                      | Shoot, Freeze Trap         |
| Mage      | Fragile spellcaster, staffs (Defense -1)    | Fireball, Burn                                | Invisible Cloak, Ice Armor |

## Game modes

- **Dungeon Crawl** - the original roguelike loop: procedurally generated
  levels, exploration, loot, and a final Amulet-of-Yala-style win
  condition.
- **Battle Arena** - a separate, feature-complete mode: Adventure Select
  → starting shop → 3 levels of waves + a boss each → shop between
  levels → Victory, with its own gold economy (priced items, per-kill
  income, an affordability check on every purchase) and its own,
  separately-tracked stats.

## Combat system

Battles use an ATB (Active Time Battle, FFVII-style) system: each
combatant has a Speed-driven gauge that fills continuously in real time,
and whoever's gauge fills first gets to act - the player via a menu, the
enemy automatically. This replaced an earlier fixed-round "whoever's
faster goes first" system. See `src/battle/mod.rs` (`BattleTurn`,
`atb_fill_rate`) for the mechanics.

## Current state / what's in progress

This section will drift out of date fast - **check the project
instructions doc's own "Current state" and "Project Goal" sections for
the real up-to-date picture**, since that's what gets refreshed at the
end of every working session. As of this writing:

- All 5 real classes are fully built; the Debug class is an intentional
  hidden placeholder.
- Battle Arena is feature-complete for a full playable loop.
- Combat just moved from fixed-round turns to the ATB system described
  above.
- A basic "walking in place" idle-animation system exists (every
  entity cycles through a small set of frames when standing still), but
  every frame currently points at the same placeholder art - real
  distinct walk-cycle art, and a possible move from one shared sprite
  sheet to a sheet per class, are both future work.
- Planned next: AOE attacks, battling multiple stacked enemies at once,
  a mouse-targeted ranged AOE, and eventually music/sound (no crate
  chosen yet - `rodio` is the leading candidate, since bracket-lib has no
  built-in audio support).

## Project structure

```
src/
  main.rs           State machine, top-level tick dispatch, console setup
  battle/           Battle mechanics: turn/ATB state, damage, buffs, status effects
  screens/          Per-TurnState rendering + input (title, battle, pause, etc.)
  systems/          Legion ECS systems (movement, animation, rendering, input, ...)
  spawner/          Player/entity spawning, template-driven monster/item data
  map_builder/      Procedural dungeon generation (rooms, automata, drunkard's walk, ...)
  components.rs     ECS component definitions
  stats.rs          Play-history stat tracking
  keymap.rs         Rebindable movement keys
resources/
  template.ron        Item/enemy/technique definitions
  starting_kits.ron   Per-class starting inventory
  dungeonfont.png     Main sprite sheet (16x16 grid of 32x32 cells, CP437-based)
  terminal8x8.png     Fallback small-text font
```