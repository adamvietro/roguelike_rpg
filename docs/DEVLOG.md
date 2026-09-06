# Ever Space RRPG — DEVLOG

Full history, detailed reasoning, and the backlog. `CLAUDE.md` is the
lean, always-loaded file — this one is a reference to pull up when
something looks like a recurrence of a documented issue, or when picking
the next thing to build. Not auto-loaded every session; read it on demand.

---

## Current state (as of the last full session)

That session (9/6/26) was almost entirely dungeon-exploration UI: a new
Item Bar, a real player-status frame, a reworked Pause screen, and a
full redesign of the Item Menu into a 5-box character dashboard - plus
a couple of real bugs found and fixed along the way.

**Item Bar.** A third icon bar (blue box) sitting immediately left of
the Ability Bar (Item | gap | Ability | gap | Battle, one row) for
universal consumables (`spawner::universal_item_names`). Sits alongside
the Item Menu rather than replacing it. Click (left mouse button) to use
directly - the first click-to-activate interaction in the game
(`components::MouseLeftJustPressed`, a real physical-click edge detector
built because bracket-lib's own `left_click` fires twice per click).
Ability Bar got the same click support shortly after, reusing the exact
same `use_ability` the number keys already call.

**Player-status frame.** Replaced the old full-width health bar with a
top-left class-portrait icon (the player's existing `Render`, no new
art) plus a compact health bar - one row tall specifically because
HUD_CONSOLE has no sub-cell text positioning, so a multi-row bar could
never actually center its "current / max" overlay. Later gained small
buff badges (`BUFF_BADGE_CONSOLE`, a new 32px-cell dungeonfont console)
for Invisible/Stealthed/IceArmored - real ability icon art, not
placeholders, on a console dedicated to being smaller than the 40px
portrait/Ability Bar icons.

**Pause screen.** Bigger text (moved off the old 8px console onto the
same BIG_TEXT_CONSOLE/HUD_CONSOLE split every other menu uses), a real
arrow-key + Enter menu (Resume/Options/Quit) reusing `battle::MenuCursor`
as-is, and the dungeon HUD's old permanent "how to play" hint moved here
as a rotating "Hints" box in the lower third of the screen.

**Item Menu redesign.** Replaced the old single potion-list-plus-
reference-panel screen with 5 boxes (Items, Equipped Items, Stats,
Battle Actions, Dungeon Actions) plus a shared description panel -
`screens/item_menu.rs`. Cursor again reuses `battle::MenuCursor`
unmodified (Left/Right switches side, Up/Down flows through both
stacked boxes on a side as one list). Only Items and Dungeon Actions are
usable via Enter; Equipped Items and Battle Actions stay browse-only.
Stats box is mode-aware (Arena Level/Wave during a Battle Arena run,
Dungeon Level otherwise).

**Two real bugs found via screenshots, not code review:**
- `ability_bar_box_bounds`'s right edge used a plain truncating division
  + flat `+1` pad - the exact pattern its OWN doc comment already flagged
  as insufficient for the bottom edge. A full-bleed icon visibly crowded
  the border until the right edge got the same ceiling-division fix the
  bottom edge already had.
- The Battle Arena shop's fixed top-left item list collided with the new
  player-status frame once that frame moved into the same corner -
  replaced with a single tooltip for whichever item is adjacent to the
  player (`components::shop_item_near`), anchored to the player's own
  screen position so it travels with them instead of needing to dodge
  anything.

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