# Ever Space RRPG — CLAUDE.md

Persistent instructions for Claude Code in this repo. Kept short on purpose —
loaded every session, so token cost matters here. Full history and detailed
quirk write-ups live in `docs/DEVLOG.md`; the backlog/brainstorm list lives
in `docs/ideas.md`. Read either only when it's actually relevant (e.g. a bug
smells like something documented there, or picking the next thing to
build), not by default.

## Read this first — the rules most worth not missing

- `cargo check` after every meaningful edit, `cargo build` before calling
  anything done, and rerun the class-survivability simulation after any
  balance change. See "Build & verify" below.
- **Never `git push`** — the user pushes themselves via GitHub Desktop, and
  this environment has no credentials for it anyway. Local commits/merges
  to `master` are fine and don't need to wait for permission each time.
- **Don't try to drive the running game yourself** (synthetic input,
  screenshotting a live/interactive state) — ask the user to run it and
  send a screenshot instead. See "Build & verify" below.
- "Does this make sense?" is a request to confirm understanding, not a
  go-ahead — describe the plan and wait, don't start writing code.
- One `# <date>` header per day in `docs/journal.md` — append `##`
  sections under the existing one, never add a second header for a day
  that already has one. See "Journal" below.
- A finished backlog item in `docs/ideas.md` moves to `# Done` — it
  doesn't stay in the numbered list with a "(fixed)" note. Renumber
  whatever shifts as a result.
- When working on a new item from the Idea's todo list always make a new 
  branch.

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
  input) to verify a change — ask the user to run it and send
  screenshots instead.** A static screenshot (direct X11 window
  capture) works fine, but every real attempt at synthetic input on
  this WSLg setup — XTEST `fake_input`, `XSetInputFocus`, a synthetic
  click, an EWMH `_NET_ACTIVE_WINDOW` activation message — has failed
  to actually advance the game past its title screen, consistently
  (confirmed 2026-09-13 via a real multi-method attempt, not a one-off
  flake). See `docs/DEVLOG.md`'s Known Environment Quirks for the full
  WSLg focus-arbitration theory. Ask for a screenshot for anything
  needing live visual confirmation instead of trying to get there
  yourself.
- **Never `git push`** — the user pushes their own work via GitHub
  Desktop, and this environment has no push credentials regardless.
  Local commits and merges (including straight to `master`) are fine.
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
- **A `with_simple_console_no_bg` console's `bg` parameter is a total
  no-op** — confirmed straight from bracket-terminal's real
  `CONSOLE_NO_BG_FS` GLSL source, not guessed: the shader receives a
  background color as an input but never reads it anywhere; every
  fragment is either the glyph's own opaque texture or a hard `discard`,
  with no "solid background" code path at all. `DrawBatch::fill_region`/
  `set`/`set_bg` with a space glyph and a `bg` color on a `no_bg` console
  (e.g. `HUD_CONSOLE`) silently does nothing (2026-09-14, `render_helpers::
  draw_filled_pixel_box`). To actually paint a solid color on a `no_bg`
  console, use a full-block glyph (CP437 219, `'█'`) tinted via `fg`
  instead — the same multiply-tint trick every other tinted icon in this
  project already relies on, just aimed at a solid block.
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
- **A plain console's shader discards a pixel outright if all 3 RGB
  channels are below 0.1 (never reads alpha); a fancy console's shader
  discards only when RGB AND alpha are both below that same 0.1 cutoff,
  else falls back to the per-vertex background color** (confirmed
  against bracket-terminal's actual GLSL/WGSL source, not just
  observed). Two consequences for any sprite sheet: floor real content's
  near-black pixels to ~30+ per channel (not exactly 25.5) so they
  survive either console type, and for a sheet meant for a **plain**
  console, force truly-transparent pixels' RGB to real (0,0,0) too —
  alpha-only transparency (common after a background-removal tool) still
  renders as a solid wrong-colored block there. See `docs/DEVLOG.md` for
  the full shader-source reasoning.
- `set_fancy` renders one full cell north of the same position via plain
  `set()` — compensate with a `..._Y_ANCHOR_OFFSET` constant.
- **A specific glyph can render solid black on one console and correctly
  on another, even with provably identical color/glyph inputs and
  genuinely bright (not near-black) font pixel data** (`map_render.rs`'s
  Exit/Counter tiles, 2026-09-13) — confirmed by measuring a real screen
  recording frame-by-frame, not a guess. Traced everything that could
  explain it (the computed `ColorPair`, the font's real pixel data,
  bracket-terminal's actual shader/vertex-buffer source) and found no
  difference between the working and broken paths; the literal internal
  reason was never found (full trace in `docs/DEVLOG.md`). If this
  recurs, the faster fix is likely the same one used here: switch the
  content to whichever console type is confirmed working, rather than
  re-tracing the same shader/vertex-buffer chain again.
- A custom-sized `Camera` doesn't shrink what renders around a small map
  — the camera frames a fixed window regardless of map size. Use the
  reveal-rectangle approach for "this map should look small" instead.
- Console z-order is registration order; a later-registered console
  (including anything the Ability/Battle Bar or any future icon bar use)
  paints over lower ones wherever it actually draws something.
- **`bracket_pathfinding::DijkstraMap::build` never writes 0.0 into a
  seed tile's own array slot** (the seed enters its internal queue with
  depth 0.0, but `dm.map[seed_idx]` stays at `f32::MAX` unless a
  neighbor's relaxation pass overwrites it later, near the edge cost
  instead of 0). A greedy "step toward the lowest `dijkstra.map` value"
  bot can stall in a cycle right next to its own target as a result.
  **Special-casing "this candidate IS the literal target" is not a full
  fix** — confirmed by a second real reproduction where that patch just
  moved the same stall to a different nearby cell. The reliable fix used
  in `screens/battle.rs` was replacing `DijkstraMap` outright with an
  in-house BFS (see `bfs_distance_field`'s own doc comment for the full
  history) — uniform-cost movement doesn't need Dijkstra's priority
  queue anyway. `systems/chasing.rs` still uses `DijkstraMap` with only
  the simpler special-case patch — an unconfirmed latent risk, not a
  solved problem, if a similar "enemy won't approach" symptom shows up
  there.
- **A custom `with_font` sheet needs a glyph grid big enough to cover
  index 32, AND whatever row `32 / cols` computes needs to actually BE
  blank** — every console's `cls()` fills all never-drawn cells with
  glyph 32 by default, and this has bitten in three confirmed-real,
  distinct ways (all in `docs/DEVLOG.md`'s Known Environment Quirks):
  1. Not enough rows/cols to contain index 32 at all → an immediate
     launch-time panic (`glyph_position`'s unsigned subtraction
     underflows the moment `cls()` first runs).
  2. Enough cells, but real content sitting on the specific row
     `32 / cols` lands on → every undrawn cell of that console silently
     tiles with THAT content instead of showing nothing (hit twice for
     real, on two different sheets, each time a new class/row assignment
     happened to land there).
  3. An "empty row" that's only empty because nothing has claimed it
     yet is not a guarantee — the exact tiling bug in #2 recurred when a
     later addition (a 6th class) claimed that row.
  Fix: give every per-class/per-theme sheet its own dedicated
  row-assignment function that explicitly skips whatever row `32 / cols`
  computes for THAT sheet's own column count — never reuse another
  sheet's mapping, and re-check the division fresh for a new sheet's
  column count even if an existing mapping looks safe.
- **A new custom sprite sheet needs its own `.with_font("name.png", w,
  h)` call, separate from and BEFORE registering any console that uses
  it** — registering the console alone (`.with_simple_console_no_bg`/
  `.with_fancy_console(..., "name.png")`) compiles fine but panics at
  launch (`no entry found for key` in bracket-terminal's initializer,
  confirmed 2026-09-14 — the game would open and immediately close) if
  the matching `.with_font(...)` line was never added to the font list
  near the top of the builder chain, where every other custom sheet in
  this project already has one. Verify a new font asset actually works
  by running the game for real (`timeout 8 cargo run`, check for a
  panic in the output and confirm no process is left running) — `cargo
  check`/`build` alone can't catch this, it's a runtime-only failure.
- **`to_cp437('█')` (CP437 index 219) as a "solid fill" glyph only works
  by coincidence on a font sheet large/opaque enough that wherever
  index 219 lands is still opaque** — `render_helpers::draw_panel_fill`
  gets away with it on `ui_panels.png` (a real 45-cell sheet, always
  tinted BLACK, so the multiply zeroes out whatever garbage color got
  sampled regardless). A small custom font (e.g. `battle_bar_frame.png`,
  originally 3 cells) has no cell anywhere near index 219 — the same
  call silently sampled into an intentionally-transparent part of the
  sheet, and a REAL (non-BLACK) fill color multiplied by near-zero
  alpha there rendered as nothing (confirmed live 2026-09-14 — the
  frame drew fine, the fill never appeared at all). Fix used: add a
  dedicated solid-opaque cell to the small font itself and reference
  its own real (small, in-range) glyph index instead of reusing a
  CP437 constant meant for a full character set.

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