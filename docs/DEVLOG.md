# Ever Space RRPG — DEVLOG

Full history, detailed reasoning, and the backlog. `CLAUDE.md` is the
lean, always-loaded file — this one is a reference to pull up when
something looks like a recurrence of a documented issue, or when picking
the next thing to build. Not auto-loaded every session; read it on demand.

---

## Current state (as of the last full session)

That session (9/7/26) built a whole Dungeon Crawl economy from scratch -
gold, a guaranteed loot chest, a between-floor shop, and a first
data-driven starting-kit balance pass - plus three real bugs found and
fixed along the way, one via a user screenshot, one via the balance
simulation itself, one via actual playtesting.

**Gold + the `shop_only` loot flag.** `Gold` (components.rs) was Battle
Arena-exclusive - Dungeon Crawl players now get `Gold(0)` at
`start_game` too, and every existing gold codepath (traps, ranged
strikes, `buy_nearby_item`, battle kills) already gated purely on the
component's PRESENCE, so those started paying out for free, no per-mode
branching needed. Healing Potion/Dungeon Map pulled out of the ambient
floor-loot pool via a new `Template::shop_only` flag (same pattern as
`prefab_only`/`boss_only`) - still grantable by name (a chest, the shop).

**Guaranteed per-floor chest.** A new, always-attempted (not
random-one-of-three like Fortress/Turret/Bunker) prefab room
(`map_builder/prefab.rs`), guarded by 1-2 copies of that floor's
toughest non-boss enemy (`spawner::spawn_prefab_chest_guards`). Grants
gold + a Map + Potions in one lump, then a full-screen
`TurnState::ChestOpened` overlay styled like Paused (reuses
`pause_systems` outright - just `map_render`, so sprites drawn on the
map a frame ago simply aren't redrawn). New `c` glyph drawn from a
user-supplied reference image, flood-filled to true transparency from
its outer edge so interior highlights survived.

**Dungeon shop between floors.** A floor's own stairs now lead into a
shop room first (`TurnState::DungeonShopTransition`) - reuses
`MapBuilder::new_arena_shop`/`arena_rebuild_keep_player`/
`spawn_arena_shop_items`/`buy_nearby_item` completely unmodified,
stocked with a fixed Healing Potion/Dungeon Map pair instead of Arena's
class-rolled list. Leaving via the shop's own stairs routes back through
`advance_level`, which now also clears `ShoppingActive`/`ShopMessage` on
the way out (it's now reached from two places, not just a stairs step).

**Class-survivability simulation - now a permanent tool.** A headless
bot plays several runs per class through the REAL game logic
(schedulers, movement, combat resolution, chest/shop interaction, not a
simplified model) to measure how many actually reach the first shop
alive. `#[ignore]`d so it doesn't run in the normal `cargo test` (real
time even in release): `cargo test --release class_survivability_report
-- --ignored --nocapture` (see `screens/battle.rs`, and the pointer in
CLAUDE.md). First run found only 3/50 reaching the shop with Mage dying
10/10; every class's starting kit now carries 3 Healing Potions
(Barbarian gets a kit at all now) and Mage's Speed went 6 -> 7 - deaths
dropped to 2/50 after. The bot's "flee when critical" rule has a known
repath-into-the-same-enemy loop bug inflating timeout counts now instead
- see docs/ideas.md item 3, not yet fixed.

**Three real bugs found and fixed:**
- Giving Dungeon Crawl players a `Gold` component broke
  `record_enemy_kill`'s loot-vs-gold branch and the Victory screen's
  display, both of which used Gold's presence as their Arena-check -
  ability loot silently stopped dropping from Dungeon Crawl battles.
  Fixed by checking `Option<ArenaRun>` instead, which is actually
  Arena-exclusive; Arena's own behavior didn't change at all.
- The chest room's wall template fully enclosed the interior with no
  door at all (found via the user's own screenshot, "I can't get in") -
  opened one on the guards' row specifically (not the chest's own row),
  so reaching the chest requires passing them, not walking past them.
- `buy_nearby_item`'s player lookup had no `Player` filter at all, so it
  could silently grab the Shopkeeper NPC's or a `ShopStock` counter's
  `Point` instead (both exist in the same scene). Never visibly broke
  the Arena shop, apparently by luck of legion's iteration order, but
  broke the new Dungeon Crawl shop outright - found by the user
  ("I can't buy the potion"), reproduced with a real test both before
  and after the one-line fix.

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

The live, actively-maintained backlog is `docs/ideas.md` — this section
used to duplicate it and had already drifted out of sync by the time it
was noticed, so it's now just a pointer instead of a second copy that
can silently disagree with the real one.

One note worth keeping here since it's a technical observation rather
than a todo item: several brainstormed class abilities (Barbarian's
Rampage/Berserk, Amazon's Momentum/Keen Eyes) are "always-on while a
condition holds" passives — a genuinely new mechanical category. Every
effect today is a one-time consumable (Technique) or one-time
out-of-combat use (Effect), so the first true passive needs its own
system, not just a new template entry.

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