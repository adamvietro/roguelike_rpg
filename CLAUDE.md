# Ever Space RRPG — CLAUDE.md

Persistent instructions for Claude Code in this repo. Kept short on purpose —
loaded every session, so token cost matters here. Full history and detailed
quirk write-ups live in `docs/DEVLOG.md`; the backlog/brainstorm list lives
in `docs/ideas.md`. Read either only when it's actually relevant (e.g. a bug
smells like something documented there, or picking the next thing to
build), not by default.

## Stack

- Rust, edition `2018`. `legion` (=0.3.1) ECS, `bracket-lib` (~0.8.7)
  rendering, `serde`/`ron` (=1.0.115/=0.6.1) for data. Versions pinned with
  `=` in several places — flag any new dependency clearly before adding it.
- Repo: https://github.com/adamvietro/roguelike_rpg (public)
- **Decided new game name: "Five Blades Deep"** (chosen 2026-09-13,
  checked clear of existing games/trademarks - "Ever Space" collides
  with a real existing game and doesn't fit this project's fantasy
  dungeon-crawler genre anyway). **Not yet renamed anywhere** - crate
  name (`ever_space_rrpg`), the in-game window title, this repo's name,
  and every doc still say the old name on purpose, deferred to its own
  dedicated pass rather than tangled into other in-progress work. See
  `docs/ideas.md`'s numbered backlog for the full rename task.
- Glyph/sprite map lives in `docs/Dungeon_Font_Glyph_to_Cell_Map.md`,
  tracked in this repo — the master reference for both `dungeonfont.png`
  AND the PixelLab character sheets (`character_idle.png`,
  `character_battle.png`, `character_portrait.png`). See "Sprite sheet
  editing" / "Character sheet art" below before touching any of them.

## Build & verify — do this yourself now, don't just describe it

You have direct file access and a real terminal here, so use them:

- `cargo check` after every meaningful edit; `cargo build` before calling
  anything done. Run the actual project — no scratch lockfile juggling
  needed the way a sandboxed session used to require.
- **A clean build is not sufficient proof of correctness for two specific
  bug classes in this project**, both invisible to the compiler:
  1. **legion component-access mismatches.** `#[system]` functions declare
     access via `#[read_component]`/`#[write_component]`; querying a type
     not declared there compiles fine and panics `AccessDenied` only at
     runtime. Any time a system's query set changes (directly, or via a
     helper function it calls), re-verify its attribute list matches.
     `systems/hud.rs::hud_system_execution_tests` is a **permanent**
     regression test for this — keep it, and copy the pattern (build a
     `Schedule`, `.execute()` it for real) for any other system that's
     been a repeat source of this.
  2. **bracket-lib rendering specifics** (color blending, console z-order,
     pixel-to-cell rounding) — invisible to any compiler check. Verify by
     tracing the library source or asking for a screenshot; don't assume.
- **Don't try to drive the running game yourself (synthetic X11
  input/screenshots) to verify a change — ask the user to run it and
  send screenshots instead.** Confirmed 2026-09-13: launching the binary
  and screenshotting a static screen works fine (direct X11 window
  capture via python-xlib), but every attempt at synthetic input —
  XTEST `fake_input` key/mouse events, explicit `XSetInputFocus`, a
  synthetic click into the window, longer key holds, and a proper EWMH
  `_NET_ACTIVE_WINDOW` activation message — failed to actually advance
  the game past its title screen, consistently, across a real
  multi-method attempt, not just a one-off flake. Matches this project's
  own prior "X11 input driver flakiness" history (WSLg host-compositor
  focus arbitration is the leading theory) — not worth re-attempting
  each time it comes up. The user can drive the game and screenshot it
  themselves without this problem, so for anything needing live visual
  confirmation (does a screen render right, is something positioned
  correctly, does an animation look correct in play), ask them for a
  screenshot rather than trying to get there yourself.
- **Write a real test for logic a type-check can't confirm** (an
  algorithm's actual behavior, a bugfix's actual effect), then remove it
  before calling the work done — except the legion-access-pattern
  exception above, which stays permanently.
- Verify RON data at runtime too, not just that it parses — a missing
  field a feature depends on (e.g. a `description:` some items lacked)
  only surfaces when actually loaded and read.
- **After any class-balance change** (stats, starting kits, technique
  numbers), rerun the class-survivability simulation:
  `cargo test --release class_survivability_report -- --ignored --nocapture`
  (a permanent `#[ignore]`d test in `screens/battle.rs` — real time even in
  release, which is why it's not part of the normal suite). Plays several
  headless runs per class through the real game logic and reports how many
  reach the first dungeon shop alive.
- Double-check brace balance and re-view the *exact* file about to be
  committed after any large sequential edit, especially ones assembled
  from several separate edits — don't trust an earlier read of the file.

## Coding conventions

- LF line endings throughout, enforced by `.gitattributes` — never
  introduce CRLF.
- Data-layer helpers that query components should be generic over
  legion's `EntityStore` trait (`fn foo<T: EntityStore>(ecs: &T, ...)`)
  rather than hardcoded to `&World` or `&SubWorld`, so they work both from
  plain screen-tick methods and from inside `#[system]` functions. See
  `components.rs`'s `usable_menu_items`/`ability_bar_slots`/etc.
- The `class:` field already on item templates is the source of truth for
  "universal item" (Item Menu) vs. "class-restricted ability" (Ability
  Bar) vs. "battle-only technique" (Battle Bar) — don't add a new field
  for this distinction, reuse the existing one.
- When a large visual/architectural change is requested, talk through the
  design (what triggers it, what data it needs, what interaction model)
  before writing code — this project's Item Menu/Ability Bar/Battle Bar
  work went smoothly specifically because that conversation happened
  first. Ambiguity in a *design* request is worth a clarifying question;
  ambiguity in a small, well-specified task is not.
- When reasoning about exact pixel positions/layout in bracket-lib without
  being able to render and check, say so plainly and expect a correction
  round from a real screenshot rather than presenting a first guess as
  confidently final.
- When the user asks "Does this make sense?" (checking a plan/design out
  loud), that's a request to confirm understanding, not a go-ahead —
  describe what you're about to implement first and wait, rather than
  moving straight into writing code.

## Standing gotchas (condensed — see docs/DEVLOG.md for the full reasoning)

- `WINIT_UNIX_BACKEND=x11` is required on this WSL setup — already set
  permanently via `.cargo/config.toml`. Don't re-diagnose if this error
  reappears in a fresh clone; check that file first.
- A held key's OS auto-repeat on this WSLg/X11 stack can arrive as fake
  alternating release+press events, not one sustained press. Any "did the
  user actually let go of this key" logic needs a real-time debounce
  (~150ms), not a single-frame check.
- bracket-lib's console shader multiplies a glyph's color
  (`texture_pixel * fg_color`). Fine for monochrome text (white × grey =
  grey); for full-color custom sprite icons, a light grey barely dims them
  — use a much darker dedicated tint for "disabled" icon states, not the
  same constant used for greyed-out text.
- Converting a pixel boundary into a row/column across two consoles of
  different resolution: which way to round (floor vs. ceiling) depends on
  which side of the boundary that edge must stay on. A flat "+1"/"-1"
  is not a substitute for picking the correct rounding direction — verify
  with a test that checks the real pixel relationship, not just "the
  number looks bigger."
- A new `#[resource]` used by any system in the title-background
  schedulers (`systems/mod.rs`'s `build_title_background_*` functions)
  must be inserted in both `State::new()` and `State::return_to_title()`,
  or startup/return-to-title panics on an `Option::unwrap()`.
- A newly registered console must be added to `tick()`'s per-frame
  `ctx.cls()` sweep (one call per existing console) or content drawn
  there on one frame lingers on every later frame where nothing
  redraws that cell — invisible for a console that's always fully
  redrawn every frame (like the dungeon map), but a real bug for
  anything conditionally drawn (an icon that only appears sometimes,
  like a buff badge) once the condition goes false again.
- `CommandBuffer::add_component` calls computed from an entity's current
  state are unsafe to issue more than once per entity per tick — edits
  aren't visible until flush, so a second call reads the same stale value
  and overwrites (doesn't add to) the first. Accumulate into a local and
  apply once.
- **Confirmed from bracket-terminal's own GLSL source** (not just
  observed): a plain (`with_simple_console_no_bg`) console's fragment
  shader discards a pixel if all three of its RGB channels are below
  0.1 (25.5/255) — it never reads the alpha channel at all, so a real
  alpha channel by itself does nothing there; transparency on a plain
  console is a pure RGB colorkey. A fancy console's shader instead shows
  texture content only when (at least one RGB channel is above that same
  0.1 cutoff) AND alpha is above 0.1, else falls back to the per-vertex
  background color — this is what the transparent-background trick
  (`RGBA::from_f32(0,0,0,0)` passed to `set_fancy`) actually relies on,
  and it's also where "bracket-lib culls near-black opaque pixels"
  comes from: a fancy console's own near-black-but-opaque pixel fails
  the "at least one channel above 0.1" half of that check and gets
  treated as no-content regardless of its real alpha. Two consequences
  for any sprite sheet: floor real content's near-black pixels to at
  least ~30 in every channel (not 25.5 exactly — leave margin) so they
  survive on EITHER console type, and for a sheet meant to render on a
  **plain** console specifically, force every actually-transparent
  pixel's RGB to true (0,0,0) — setting only its alpha to 0 and leaving
  old opaque-looking RGB behind (e.g. from a background-removal tool
  that clears alpha but not color) renders as a solid, wrongly-opaque
  block, invisible to any type check and only caught by an actual
  screenshot.
- `set_fancy` renders one full cell north of the same position via plain
  `set()` — compensate with a `..._Y_ANCHOR_OFFSET` constant.
- **A specific glyph can render solid black on one console and correctly
  on another, even with identical color/glyph inputs and genuinely
  bright (not near-black) font pixel data** — confirmed real 2026-09-13
  (`map_render.rs`'s Exit/Counter tiles): rendered solid black through a
  plain console every time the camera was at rest, rendered correctly
  through a fancy console every time the camera panned, verified by
  extracting and measuring every frame of a real screen recording (not
  a guess). Traced the actual `ColorPair` being computed (correct, via
  debug logging), the font's real pixel content (bright, not corrupted),
  bracket-terminal 0.8.7's own `.wgsl` shaders for both console types,
  its GPU vertex-buffer-building code, and `FontScaler`'s UV math - every
  one identical for both paths on paper. The literal internal reason was
  never found. If this exact symptom recurs (a glyph/color that's
  provably correct in our own code still fails to render on a specific
  console), the faster fix is likely the same one used here: switch that
  content to whichever console type is confirmed working, rather than
  re-tracing the same shader/vertex-buffer path a second time.
- A custom-sized `Camera` doesn't shrink what renders around a small map
  — the camera frames a fixed window regardless of map size. Use the
  reveal-rectangle approach for "this map should look small" instead.
- Console z-order is registration order; a later-registered console
  (including anything the Ability/Battle Bar or any future icon bar use)
  paints over lower ones wherever it actually draws something.
- **`bracket_pathfinding::DijkstraMap::build` never writes 0.0 into a
  seed tile's own array slot** (confirmed from its source: the seed only
  ever enters the algorithm's internal queue with depth 0.0, but
  `dm.map[seed_idx]` itself is left at its initial `f32::MAX` unless a
  neighbor's own relaxation pass later overwrites it with ~the edge cost
  back to that neighbor, e.g. ~2.0 for one cardinal hop). A tile you seed
  Dijkstra at (a navigation target) can therefore report a WORSE distance
  than a tile genuinely one step further away, and any greedy "step
  toward whichever neighbor has the lowest `dijkstra.map` value" bot can
  end up in a stable cycle right next to the goal, worst-cased on a
  target boxed against a wall with few approach angles (confirmed via a
  real reproduction next to the Arena shop's exit tile). Fix: when
  picking among an entity's own candidate moves, treat "this candidate
  IS the literal target" as an automatic, unconditional win — never trust
  `dijkstra.map[]` for that one specific index.
- A custom `with_font` sheet needs a glyph grid big enough to cover index
  32 — every console's `cls()` fills all cells with glyph 32 by default
  (bracket-terminal's own `SimpleConsole::cls`), and `FontScaler::
  glyph_position` does an unsigned subtraction on the row it computes for
  whatever glyph it's given, with no bounds check. A small custom sheet
  (e.g. a 5-column-wide one) can compute a row of 0 for glyph 32, and
  `glyph_y - 1` panics with "attempt to subtract with overflow" the
  moment that console is ever cleared — happens at startup, before any
  real content is drawn, so it's easy to mistake for something else being
  wrong. Fix: pad the sheet's total rows/cols so index 32 lands on a real
  (even if blank/transparent) cell, not just enough for the content you
  actually placed.
  **This has a second, subtler form that doesn't crash at all**: even
  once there are enough rows to avoid the panic above, whichever specific
  row `32 / cols` (integer division) lands on needs to actually BE blank
  - if real content happens to sit there, every cell of that console that
  goes undrawn on a given frame silently shows THAT content instead of
  nothing. Confirmed for real on `character_battle.png` (8 columns, so
  `32 / 8 == 4` exactly): Mage's row happened to be row 4, and Mage's
  portrait replaced every empty cell of that console - which spans the
  full display - filling the ENTIRE screen with tiled Mage portraits on
  every single screen in the game, not just during battle. Two sheets
  with different column counts can need DIFFERENT row assignments for
  the exact same set of classes purely because `32 / cols` differs
  between them - don't assume a row mapping that's safe on one sheet is
  safe on another without checking that division again for the new
  sheet's own column count. **This isn't hypothetical caution — it
  recurred for real.** `character_idle.png` (6 columns, `32 / 6 == 5`)
  was documented as "safe" purely because nothing had been assigned to
  row 5 yet; the moment a 6th class (the hidden "Debug" class) got
  assigned there, the same tiling bug reappeared, this time filling the
  Adventure Select screen with the new class's portrait. A sheet with an
  apparently-safe empty row is not a guarantee, just an unclaimed one —
  give every per-class sheet its own dedicated row-assignment function
  from the start (don't reuse another sheet's mapping "since it already
  has a free row there"), and explicitly skip whatever row `32 / cols`
  computes for that sheet's specific column count, the same way
  `character_battle_row`/`character_idle_row` do in `components.rs`.
  **A third, crash-shaped variant (2026-09-11, `battle_backgrounds.png`,
  one glyph = one full-screen image):** a sheet doesn't need a wrong ROW
  to have live content on it to fail — it can simply not have ENOUGH
  rows/cols to contain index 32 at all, in which case `glyph_position`'s
  unsigned subtraction underflows immediately on the very first `cls()`
  (a real launch-time panic, "attempt to subtract with overflow", not a
  silent wrong-content bug). This bites hardest on a sheet with very few
  columns — a single-column, one-full-image-per-glyph sheet needs `cols *
  rows >= 33` just to give index 32 SOME valid cell, which at 1 column
  means 33 full rows. Don't pad a narrow sheet taller to reach that
  minimum — pick a wider grid instead (e.g. 6x6 instead of 1x33): total
  padded cells needed is roughly constant regardless of shape (always
  ~33), but a tall-and-narrow texture risks exceeding a GPU's max texture
  dimension where a squarer one won't.

## Journal (`docs/journal.md`) — also a source for blog posts

Each top-level `# <date>` header in the journal (e.g. `# 9/8/26`) is meant
to become one post on the user's devlog blog
(https://blog-wild-leaf-1554.fly.dev), titled "Custom Roguelike -
<date>" — confirmed against the two posts already up there: `/posts/181`
("Custom Roguelike - 9/06/26") and `/posts/182` ("Custom Roguelike -
9/07/26"). Goal, not yet built: post directly from a day's journal.md
section right after a day's work, instead of copy/pasting by hand.
**Before building any automation for this, find out whether the blog
exposes a real API vs. only a plain HTML form (likely behind login)** —
that decides whether this becomes a simple scripted HTTP call or needs a
headless browser/session cookie from the user.

**Header/session workflow (standing rule, added 2026-09-13)**: one
`# <date>` header per calendar day, created once. Every session that
lands work that same day adds `##` (or deeper) sections underneath that
SAME header — never a second `# <date>` for a day that already has one.
Check the end of the file for an existing header matching today's date
before adding a new one. While the day's work is still unpushed, leave
the header as the plain date. Once the user actually does a real `git
push` covering that day's accumulated work, rename that header to also
carry a short description of everything done that day (append to the
date, don't replace it) — e.g. `# 9/13/26 — Victory/Defeat Backgrounds,
Theme Select, Enemy Death Animations` — so the journal itself stays
scannable and the eventual blog-post title has something to draw on.

**Tags**: this blog is a personal multi-project devlog (Elixir/Phoenix
learning posts, other side projects, etc. — not just this game), with one
large tag vocabulary shared across all of it. Reuse an existing tag
whenever one already fits rather than inventing a new one — check the
full current list first at
https://blog-wild-leaf-1554.fly.dev/tags/search. **"My Roguelike" (blog
tag id 166) is the umbrella tag for this specific game** — every post
about this project needs it, and
https://blog-wild-leaf-1554.fly.dev/tags/search/?tag=166 is the fastest
way to see every past post (and its tags) for this game alone, without
wading through the blog's other projects. Tags already used on this
game's two posts so far, for reference: `dungeon-crawl`, `Economy`,
`balance`, `Simulation`, `Bug-Fix`, `Refactor`, `Sprite-Art`, `Pixellab`,
`Battle-Arena`, `Rust`, `Rusty Roguelike`, `ECS`, `Bracket-Lib`,
`My Game`, `Gamedev`, `Indiedev`, `devlog`, `game-ui`, `Solo-dev` (exact
casing as shown on the blog — it's inconsistent tag-to-tag, so match
each one's own existing casing rather than normalizing). No tag yet
exists for animation work specifically (walk/battle-idle art, the
Death/Victory/technique framework) - `Animation` was proposed as a new
one 2026-09-08, not yet confirmed as added to the blog's vocabulary.
Always alphabetize a proposed tag list when presenting it (standing user
preference, independent of this blog specifically).

## Sprite sheet editing (`resources/dungeonfont.png`)

Confirm the supplied PNG is current before editing (ask if it isn't
obviously the same session's upload). Match any reference image closely;
check every reference for a visible watermark/stock-marketplace mark
before using it — decline and ask for a different one if present. Crop to
content, don't stretch non-square references, floor near-black pixels (see
gotchas above), clear the complete target cell before pasting, and
pixel-diff the whole sheet afterward to confirm only the intended cells
changed. Full-character sprites fill their cell top-to-bottom on a true
transparent background, not a face-only bust. Update
`docs/Dungeon_Font_Glyph_to_Cell_Map.md` at the end of any session that
changes the mapping — even a codepoint-reservation-only pass with no pixel
edits counts as "changed" for this purpose.

## Character sheet art (PixelLab — `character_idle.png`,
`character_battle.png`, `character_portrait.png`)

Full detail (row assignments, zip format, confirmed gotchas) now lives in
`docs/Dungeon_Font_Glyph_to_Cell_Map.md` — it's the master reference for
BOTH this system and the dungeonfont above; read it before touching any
of these three sheets. The two most expensive-to-relearn lessons:
- **Never trust a PixelLab zip's `metadata.json` size for a frame's real
  canvas** — `Walk` in particular comes back padded larger than 32×32
  for most classes (confirmed: 40–48px, varies per class) to give the
  animation motion room, while the character's own pixel size and
  center stay constant. Center-crop every frame to exactly 32×32 before
  any further processing, or the character silently renders smaller
  (resize-based sheets) or bleeds into neighboring cells (native-paste
  sheets like `character_battle.png`).
- **Always blank the destination row/cell before pasting new content** —
  never trust a new frame's own transparency to fully replace old
  content whose silhouette doesn't perfectly match.
- Each of the three sheets needs its OWN row-assignment function keyed
  to its own column count — reusing another sheet's mapping has caused
  the tiled-portrait bug (see the glyph-32 gotcha above) twice for real,
  on two different sheets.
- **Before compositing ANY new PixelLab batch into a sheet, do a real
  pixel-health pass over every source frame first** — don't assume a
  batch is clean just because an earlier one was. 2026-09-11's full
  animation batch was initially assumed to need no segmentation work
  (clean binary alpha, no near-black content), but the user then spotted
  real stray transparent pixels in Hunter's frames specifically, and a
  closer look found the same problem across most of the batch, not just
  Hunter — a spot-check of "a couple classes look fine" isn't enough.
  Check every frame for both known failure modes: pure-black (or
  near-black) OPAQUE pixels needing the usual floor (see the glyph-32-
  adjacent near-black-pixel gotcha above), and pixels that are
  transparent but shouldn't be — real holes/gaps inside the character's
  own silhouette, as opposed to the actual background around it.