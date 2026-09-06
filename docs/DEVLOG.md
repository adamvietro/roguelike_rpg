# Ever Space RRPG — DEVLOG

Full history, detailed reasoning, and the backlog. `CLAUDE.md` is the
lean, always-loaded file — this one is a reference to pull up when
something looks like a recurrence of a documented issue, or when picking
the next thing to build. Not auto-loaded every session; read it on demand.

---

## Current state (as of the last full session)

That session was almost entirely about **out-of-combat UI**: how the
player selects an action or item outside of battle, and how that's
actually displayed on screen.

**Arrow-key cursor + pointer navigation, now used everywhere.**
`render_helpers::menu_nav` (wrap-around cursor stepping) and
`print_menu_row_centered`/`print_menu_row_left` (yellow highlight + a `►`
pointer glyph) are the shared building blocks, used on Adventure Select,
Class Select, Options, History, and the Item Menu. Number keys still work
everywhere too — purely additive. The battle menu got a real 2D cursor
(`battle::MenuCursor`): Up/Down within a column, Left/Right switches
columns and remembers the row you were on in each. Locked/unowned
techniques can be pointed at but not selected. Cross-battle cursor memory
is opt-in (Options: "Remember Last Battle Action") via
`settings::MenuMemory` + `settings::LastBattleAction`, seeded by matching
the remembered action's *name* in the current grid, not a raw position.

**True ATB cursor freedom + a narrow double-fire guard.** Cursor movement
in the battle menu is free in every state under Active (True ATB) mode;
gated to PlayerMenu under Wait mode (costs nothing there, time is frozen).
Holding Enter to fire the instant a gauge fills needed no new mechanism —
this engine's key input is level-triggered every frame already, and the
Filling→PlayerMenu transition happens before input handling runs each
frame. `State::pending_enter_release` guards the narrow case of the exact
held Enter that fired a killing blow (or fatal hit, or confirmed Adventure
Select) also immediately dismissing/confirming the very next screen — armed
only at those 3 hand-off points, with a 150ms debounce (see the X11
key-repeat quirk below for why a single-frame check wasn't enough).
Victory/Game Over now dismiss on Enter specifically, not any key.

**Item Menu (press M) + the Ability Bar / Battle Bar split.** Replaced the
old fixed "1=potion, 2=map" hotkeys with a real split between universal
items and class-restricted abilities, driven entirely by the `class:`
field templates already had:

- **Item Menu** (`TurnState::ItemMenu`, `screens/item_menu.rs`, key `M`,
  removed from `keymap::REBINDABLE_KEYS`) — a paused, arrow-navigable list
  of universal consumables (`components::usable_menu_items`). Free to
  open/browse; using an item costs a turn (jumps to `PlayerTurn`). Shows
  the selected item's description live (wrapped, "No description."
  fallback), plus a read-only right-side panel: Battle Attacks, class
  Abilities (full roster, greyed if unowned), and Weapons.
- **Number keys 1-9, then 0** now trigger Abilities directly
  (`player_input.rs::use_ability`), by fixed roster position.
- **Ability Bar** — icon row along the bottom of the dungeon view, one
  per out-of-combat ability (`components::ability_bar_slots`), centered
  as a group, raised one icon-height off the bottom edge, red box border,
  number labels (1-9, 0) above each icon, greyed if unowned, hover for
  description. 40×40px icons on `ABILITY_BAR_CONSOLE` (32×20 grid).
- **Battle Bar** — same icon size, immediately right of the Ability Bar
  with a gap, green box, no number labels (not usable outside battle),
  full technique roster with unowned greyed out, same hover behavior.
- The dungeon-exploration HUD is now deliberately minimal: health bar,
  hint text, dungeon level/gold, shop stock while shopping, and the two
  bars. The old "Battle Attacks"/"Weapons" text panels are gone from
  here, consolidated into the Item Menu's reference panel.
  `HudMousePos` (only used by those old panels' hover) was removed;
  `AbilityBarMousePos` is the one mouse-position resource left.

---

## Known Environment Quirks — full detail

(Condensed versions of the ones still worth a standing rule live in
`CLAUDE.md`. This is the complete list with reasoning, including
purely-historical entries kept so a future session doesn't re-diagnose
something already understood.)

- **`WINIT_UNIX_BACKEND=x11` required for `cargo run` on this WSL setup.**
  Without it, window creation panics inside `bracket-terminal`'s Wayland
  title-bar font rendering. Set permanently via `.cargo/config.toml`.
- `.gitignore` already covers `saves/` (`keymap.ron`, `stats.ron`,
  `battle_speed.ron`, `atb_mode.ron`, `menu_memory.ron`,
  `last_battle_action.ron`).
- If the game window fails to appear after a WSL/driver update, try a
  full `wsl --shutdown` from PowerShell before assuming it's a code
  issue — stuck WSLg compositor/GPU state across repeated crashes has
  caused this.
- A zip built without deleting the old archive first can carry forward a
  genuinely stale, since-deleted source file, causing a real "ambiguous
  module source" build error (distinct from harmless clutter like an
  `Old Fonts/` folder also sometimes present).
- **bracket-lib culls near-black opaque pixels to transparent on fancy
  consoles, regardless of declared background color** — confirmed real,
  still-open upstream bug
  (https://github.com/amethyst/bracket-lib/issues/197), no exposed
  toggle. Mitigation: floor source-art pixel values at RGB 10,10,10.
- **`set_fancy` positions a glyph roughly one full cell too far north**
  compared to the same position via a plain console's `set()`. Constant
  and direction-independent. Compensated via a `..._Y_ANCHOR_OFFSET`
  constant wherever `set_fancy` is used.
- Line endings are LF throughout, enforced by `.gitattributes`
  (`* text=auto eol=lf`, `*.png binary`).
- **Full-file replacements need exact name/content matching** — two past
  incidents (a file not actually present on disk yet; a file duplicated
  inside itself from an append instead of a full replace) both produced
  large, unrelated-looking error cascades from this one root cause. A
  third incident left a duplicate closing brace from a large multi-arm
  match rewrite. Always re-verify brace balance programmatically on any
  file with a large sequential edit, and re-view the *exact* file about
  to ship, not an earlier version of it. This has recurred even
  mid-session during "just removing scaffolding" edits (a broad
  text-replace once deleted a real permanent method sitting next to the
  test module being removed) — caught by re-viewing and re-testing before
  calling it done.
- **A new `#[resource]` used by a system in the title-background
  schedulers must be inserted in both `State::new()` and
  `State::return_to_title()`**, or the game panics on startup/return with
  an opaque `Option::unwrap()` — legion has no "missing resource" fallback
  for an `Option<T>` resource; the slot has to exist
  (`resources.insert(None::<T>)`), not just have a sensible default.
  Distinct from the legion component-access issue below, which fails
  differently (`AccessDenied` at query time, not `unwrap()` elsewhere).
- **A custom-sized `Camera` was tried once and fully reverted.** The
  camera always frames a fixed-size window around the player regardless
  of map size — resizing it doesn't shrink what renders around a small
  room. The actual fix for "this map should look small" is the
  reveal-rectangle approach (bound what's revealed, not what the camera
  frames).
- **`GLIDE_CONSOLE` renders above every other console except
  `ENTITY_SCROLL_CONSOLE`.** Any entity with an in-flight
  `MovingAnimation` gets routed there regardless of why it's moving —
  correct for real gameplay, but a decorative background entity that
  glides mid-step will paint over UI on a lower console. Tag decorative-
  only movers with the `DecorativeOnly` marker (`movement_system` already
  skips the glide animation for anything carrying it).
- **`CommandBuffer::add_component` computed from an entity's current
  state is unsafe to call more than once per entity per tick.** Edits
  aren't visible until flush, so two calls both read the same stale
  value and the second silently overwrites (doesn't add to) the first.
  Surfaced in `traps.rs` — fixed by accumulating into a local and
  applying one combined update at the end.
- **Removing a solid background from a reference image needs a flood
  fill from the image's own outer border**, not a flat color-distance
  threshold — a threshold can't distinguish disconnected exterior
  background from a similar color inside the subject. Also snap alpha to
  a clean binary split after resizing a transparent image — resizing can
  leave semi-transparent fringe pixels.
- **Console z-order is registration order**, and a later-registered
  console can silently hide content on a lower one it happens to sit
  above. `ABILITY_BAR_CONSOLE` is registered last (so icons always render
  above the dungeon view), which also means it renders above
  `HUD_CONSOLE` — relevant for anything drawn on `HUD_CONSOLE` that needs
  to stay clear of the icons' actual pixel footprint (see the rounding
  entry below).
- **Converting a pixel boundary into a row/column across two consoles of
  different resolution: the correct rounding direction (floor vs.
  ceiling) depends on which side of the boundary that edge must stay
  on.** A flat "+1"/"-1" isn't a substitute for picking the right
  direction. Surfaced fixing the Ability/Battle Bar's box borders: the
  border lives on `HUD_CONSOLE` (67 rows) but must clear an icon on
  `ABILITY_BAR_CONSOLE` (20 rows, higher z-order), so a border row whose
  PIXELS overlap the icon's pixel range gets visually painted over
  regardless of which row "looks correct" on paper. A plain truncating
  division always rounds down — safe for an edge that must stay ABOVE a
  boundary, unsafe for one that must stay BELOW one (truncation can land
  a row whose pixels still start before the boundary, quietly eating the
  padding meant to add clearance). Fix: ceiling division (the "+799
  before /800" trick) on edges needing to stay below/after a boundary.
  Verify with a test checking the actual PIXEL relationship, not just
  "the row number looks bigger" — a first version of exactly this test
  computed pixels-per-row via `800 / ROWS` (truncating before
  multiplying, losing most of a pixel of precision) instead of
  `row * 800 / ROWS`, and failed against already-correct code as a
  result. Re-derive a test's own math as carefully as the code it checks.
- **legion enforces component access per-system at RUNTIME**, based on a
  `#[system]` function's own `#[read_component]`/`#[write_component]`
  attributes — querying a component type not declared there compiles
  perfectly cleanly and only panics (`AccessDenied`) the moment that
  system actually executes. `cargo check`/`cargo build` cannot catch this
  under any circumstances. Bit for real once: `hud_system` gained a query
  on `BattleItem` (via a new helper function) without that component
  being re-added to its attribute list — it had been removed earlier when
  an old panel was deleted, then never restored when a new query on the
  same component was introduced later for an unrelated reason. Shipped,
  passed every build check, crashed the first time the game actually
  ran. Any time a system gains a new query — including indirectly, via a
  call to a helper function elsewhere — trace what components that
  helper actually touches and confirm they're all declared. Where a
  system has been a repeat source of this, keep a real integration test
  PERMANENTLY that builds a `Schedule` and calls `.execute()` against a
  live `World` — see `systems/hud.rs`'s `hud_system_execution_tests`,
  which exists specifically to catch this exact class of regression
  before it reaches a real build again. (Proven: temporarily reverting
  the fix made this exact test fail with the same panic signature the
  user hit, confirming it actually catches the bug, not just plausible
  in theory.)
- **bracket-lib's console shader combines a glyph's rendered color as a
  straight multiply** (`texture_pixel * fg_color`), confirmed directly
  against bracket-terminal's own GLSL source (`CONSOLE_NO_BG_FS`). Works
  differently for monochrome text than full-color custom sprite art. For
  a plain white TEXT glyph, multiplying by grey gives a clean grey — this
  is why `DARK_GRAY` reads fine for greyed-out technique/item names. For
  a full-color custom ICON sprite, the same multiply only scales each
  channel by that grey's brightness while preserving the sprite's exact
  hue — `DARK_GRAY` (169,169,169, ~66% brightness) barely dims a colorful
  icon enough to read as "disabled." Fixed with a dedicated, much darker
  constant for icon tinting specifically (`UNOWNED_ICON_TINT = (40,40,40)`
  in `systems/hud.rs`), kept separate from `DARK_GRAY`. A multiply-based
  tint can dim a colored sprite but can never truly desaturate it — a
  more aggressive (darker) value than looks obviously necessary on paper
  is usually the right call for this kind of treatment, and there's a
  hard ceiling on how "neutral grey" the result can ever look without a
  genuinely separate desaturated version of the art.
- **A naive "clear this flag the first frame a key isn't held" check can
  be fooled by this project's own WSLg/X11 keyboard stack**, which can
  deliver a genuinely-held key's OS auto-repeat as brief alternating fake
  release+press events rather than one sustained press (classic X11
  behavior without "detectable autorepeat" support, common on minimal
  window-manager setups like WSLg). Confirmed by tracing `ctx.key`'s
  actual source in bracket-terminal: set directly from raw winit
  `KeyboardInput` events with no per-frame reset, so it genuinely
  reflects physical key state as reported by the OS — a fake repeat-
  release IS a real `None` for one or more frames, not a bug in how this
  project reads input. `pending_enter_release`'s first version cleared on
  a single "not held" frame, which a fake release would trigger, letting
  the very next fake repeat re-arm dismissal within a couple of frames of
  Enter being held — defeating the guard. Fixed with a 150ms real-world
  debounce instead of a single-frame check. Any future "did the player
  actually let go of this key" logic needs the same treatment — a plain
  per-frame check is not reliable on this platform, full stop.

---

## Backlog

**Near-term priorities, roughly in order:**

1. **An Item Bar, mirroring the Ability Bar/Battle Bar model** — the
   user's idea. A third icon bar (or extension of the existing pair) for
   held items (Healing Potion, Dungeon Map, future universal items),
   same visual language as the Ability/Battle Bar, instead of (or
   alongside) the current M-menu. Not scoped — worth a design
   conversation first: does it REPLACE the Item Menu, or sit ALONGSIDE
   it? Where does a third bar physically fit? Own color-coded box,
   following red/green's precedent?
2. **Mouse-click-to-use on the Ability Bar/Battle Bar** — hover works,
   clicking to trigger doesn't yet; number keys only.
3. **Mouse-targeted ranged AOE outside battle** ("rain of fire" for
   ranged classes) — not started.
4. **Standing "fix issues with the battle system" bucket** — not a fixed
   list, whatever ATB/multi-enemy/cursor turns up with more play.
5. **A shop for Potions/Maps in Dungeon Crawl mode** — pull them out of
   floor loot, reuse Battle Arena shop's pricing/stock/purchase code.
   More natural now with a real universal-item pool behind the Item Menu.
6. **Idle walk-in-place animation art** — infrastructure built and
   cycling, every frame points at the same placeholder glyph. Needs real
   per-frame art, maybe a per-class sprite sheet instead of more cells in
   the shared `dungeonfont.png`.
7. **Music & sound effects** — no crate picked (`rodio` leading
   candidate; bracket-lib has no built-in audio).
8. **Stack-count badge on Ability/Battle Bar icons** (e.g. "x2" for two
   Freeze Traps) — simplified away to finish the bars in one session.
9. **Visual confirmation pass on the Ability/Battle Bar** — box-overlap
   bug is fixed and tested, but exact label/tooltip positioning was
   pixel-ratio math without a screenshot-correction round. Worth
   checking with a few different ability counts.
10. **README.md needs a manual pass** — controls changed materially (M
    for Item Menu, number keys mean abilities not potions/maps).
11. Cleanup: `arena_advance_to_next_shop` duplicates a chunk of
    `start_arena`'s shop-building code.
12. Minor: `tooltips.rs` still reads the old integer camera offset
    during a glide, instead of the smooth fractional one.

**Content/world:** Dungeon Shop (replace floor items, needs mob gold
drops first); Chests (enemy-guarded, findable loot containers).

**Future class ability brainstorm** (nothing scoped): see the project's
separate Ideas.md for the full per-class list (Rogue: Vanish/Riposte/
Backstab/Smoke Bomb/Shiv/Pickpocket; Barbarian: Rampage/Second Wind/
Reckless Swing/Berserk; Mage: Frost Bolt/Chain Lightning/Mana Shield/
Arcane Missile/Meteor/Drain Life; Hunter: Multi-shot/Trueshot/Snare Shot/
Camouflage; Amazon: Pierce Thrust/Retreating Shot/Called Shot/Weakpoint
Strike/Net Trap/Scout/Reposition/Momentum/Keen Eyes/Spear Wall). Several
of these (Rampage, Berserk, Momentum, Keen Eyes) are "always-on while a
condition holds" passives — a genuinely new mechanical category; every
effect today is a one-time consumable (Technique) or one-time
out-of-combat use (Effect), so the first true passive needs its own
system.

---

## Working conventions (adapted for Claude Code)

- Edit the real project files directly — no more zip upload/download
  round-trip. Still explain what a change does and why, not just apply
  it silently; assume it'll be reviewed (likely via GitHub Desktop) and
  possibly modified before committing.
- Flag new crate dependencies clearly — several versions in this project
  are pinned with `=`.
- If something in the codebase looks broken, inconsistent, or worth
  flagging, say so directly rather than silently working around it.
- When a bug report comes with a theory attached, verify against the
  actual code/pixels/library docs before proposing a fix — this
  project's history has repeatedly turned up a different root cause than
  the first hypothesis once actually measured (the "greyed-out icons"
  report turned out to be a shader-blending property of the rendering
  library, not a logic bug; the "Enter double-fires past a screen
  transition" fix needed a real-time debounce once a first, logically
  reasonable single-frame version was traced back to the X11 quirk
  above).
- When introducing a new play mode or major system, raise "what should we
  track for this" as its own conversation before writing game logic —
  don't patch stats tracking in after the fact.
- When the user supplies a reference image for art, check for a visible
  watermark or stock-marketplace branding before using it — flag it and
  ask for a different reference rather than proceeding.