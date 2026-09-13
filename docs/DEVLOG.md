# Ever Space RRPG — DEVLOG

Full history, detailed reasoning, and the backlog. `CLAUDE.md` is the
lean, always-loaded file — this one is a reference to pull up when
something looks like a recurrence of a documented issue, or when picking
the next thing to build. Not auto-loaded every session; read it on demand.

---

## Current state (as of the last full session)

9/13/26, branch `enemy-death-victory-backgrounds` (not yet merged to
master): Victory/Defeat screens now use real per-theme painted
backgrounds instead of a generic mode-based pick, a 12-zip animation
batch closed out enemy Death animations plus the Shopkeeper's first-ever
art, and a new Theme Select screen makes testing a specific dungeon
theme practical without re-rolling runs.

- **Victory/Defeat backgrounds keyed to the run's own `MapTheme`
  (Forest/Dungeon/Sewer), not a generic Dungeon-Crawl bucket.** Replanned
  mid-session after the original generic version could hand a Forest run
  a stone-vault Victory scene. `components::VictoryBackground`/
  `DefeatBackground` pick a background (+ for Victory, a matching pose)
  off `MapTheme::end_scene_theme()` or Arena; each dungeon theme
  randomizes across 2 Victory scenes, Defeat is one fixed scene per
  theme. All of it shares `resources/battle_backgrounds.png`'s existing
  padded 6x6 glyph grid - no new console needed. Every one of the 11
  backgrounds (7 Victory + 4 Defeat) is confirmed watermark-free after
  several regen rounds (Dungeon's vault scene needed four attempts).
  The Amulet of Yala icon was removed from the Victory screen entirely
  (explicit user call - it read as a mismatched flat glyph next to
  painted art); the in-dungeon pickup itself is unaffected.
- **Boss Death animations, via a new independent `Battle::dying_effects`
  list rather than keeping a "dying" enemy alive in `battle.enemies`.**
  New `enemy_death.png` sheet (south-west facing, one row per boss).
  Deliberately NOT wired by keeping the enemy in the normal roster until
  its animation finishes - that would have needed auditing every
  ATB-gauge/targeting loop (plus the headless simulation's own copy of
  the battle loop) for a "still dying" guard. Instead it's a pure
  decorative overlay: rewards/removal/the fight-over check fire exactly
  as before, the animation just plays alongside. Confirmed safe by
  rerunning the normal test suite and both headless survivability
  simulations afterward.
- **Goblin's walk redone** (fixed the spear-looks-like-a-helmet art
  problem) and **the Shopkeeper's first-ever animation** - a new
  `IdleSpriteSheet::Shopkeeper` variant with its own console trio, reusing
  the existing `IdleAnimation` machinery outright since the Shopkeeper
  never moves or turns.
- **Victory's "climbing away" pose (`VictoryPose::ClimbAway`) has real
  art now** - new rows 8-13 on `character_victory.png`, north-east
  facing, one per class.
- **A new `TurnState::ThemeSelect` screen** - Debug-class-only,
  Dungeon-Crawl-only, reached from Class Select's hidden 'D' shortcut.
  Lets the chosen theme stay fixed for the WHOLE run (`MapBuilder::new`
  gained a `forced_theme` parameter, applied before tile-variant
  assignment) instead of the normal per-floor random roll.
- **Three real positioning bugs found via user screenshots, not
  guesses**: Forest Stairs' climb pose sat on top of "Press Enter...";
  the Defeat screen's fallen portrait still used a fixed col=1 left over
  from the now-removed Amulet icon; the WalkAway pose (Dungeon
  Corridor/Sewer Walk) rendered at native 1x tile scale and was nearly
  invisible - fixed via a fancy console + 4x `set_fancy` scale.
- **A text-legibility scrim was built, then correctly reverted.** Traced
  bracket-lib's actual shader source (confirmed `_no_bg` consoles discard
  near-transparent glyph pixels outright, so no print-call background
  color can ever show there) to build a real translucent backing behind
  the Victory/Defeat header text - technically correct, but the user
  disliked the flat-rectangle look and asked for it gone until real UI
  art exists. Reverted; the underlying legibility gap on very bright
  backgrounds is a known, accepted tradeoff now (see the new PixelLab UI
  item in `docs/ideas.md`).
- **Confirmed, via a real second attempt (not just XTEST again but a
  proper EWMH `_NET_ACTIVE_WINDOW` activation message), that synthetic
  input still doesn't work in this WSLg environment** - static
  screenshots work fine, driving input doesn't. Written into `CLAUDE.md`
  as a standing rule: ask the user for screenshots of live game states
  instead of attempting to drive the game directly.
- **Researched PixelLab's real UI-generation API** (`generate-ui-v2`/
  `create-ui-asset`, pulled from its actual OpenAPI spec) as a path to
  replace every hand-drawn ASCII box border in the game - logged as a
  numbered backlog item with the full endpoint details and a complete
  catalog of every existing border; nothing generated yet.
- **Still open**: `enemy_battle.png`'s own style redo (quality call, not
  a bug), 8-way diagonal facing/rotations (deprioritized), the real UI
  art itself once an API token is available.

Full narrative in `docs/journal.md`'s 9/13/26 entry.

---

## Previous session (9/11/26) — Camera bottom-row fix + enemy-portrait-positions branch

A real off-by-one regression found and fixed, then a new branch for the
multi-enemy battle-screen formation.

- **Camera viewport was silently dropping its own bottom row while the
  player stood still.** `Camera::new`/`on_player_move` (from the
  previous session's clamping work) defined `bottom_y` as `top_y +
  DISPLAY_HEIGHT`, matching `right_x`'s own formula - but
  `map_render.rs`/`entity_render.rs` consume `left_x..right_x`
  (EXCLUSIVE) and `top_y..=bottom_y` (INCLUSIVE), so `bottom_y` needed
  to be one less than that pattern. The old pre-clamp formula (`player.y
  +/- DISPLAY_HEIGHT/2`) happened to give a correct 25-row span purely
  because `DISPLAY_HEIGHT` (25) is odd - masking that the Y-loop was
  ever inclusive at all. `SimpleConsole::set` bounds-checks and drops
  out-of-range writes with no panic, so nothing crashed - the bottom row
  just silently never drew while at rest, and only reappeared for the
  ~220ms of a glide (a `FlexiConsole`/`set_fancy` path with no such
  bounds check). Reported live as "we only see the counter and the
  stairs when the character is moving" - not actually specific to those
  two tile types, just whatever happened to be sitting on the clipped
  row. Fixed with `bottom_y: top_y + DISPLAY_HEIGHT - 1`; verified with
  an exhaustive test (every corner/edge/center target point), removed
  after confirming.
- **Branch `enemy-portrait-positions`: a zigzag multi-enemy formation,
  per-theme row values, and a debug shortcut to test it** - see
  `docs/ideas.md`'s "Battle arena enemy positions" (Done) for the full
  four-round writeup (single row -> zigzag -> shifted right -> per-theme
  rows), each verified against the real theme art before shipping, not
  guessed blind from a screenshot. Added a new Debug-class cheat,
  "Battle 4" (glyph `9`), that instantly starts a real Battle against 4
  fresh Goblins - specifically because reliably finding/herding 4 real
  enemies into one fight for testing was itself the bottleneck. New
  `Templates::spawn_named_enemy_via_commands` (mirrors the existing
  `spawn_named_item_via_commands`) spawns one exact named enemy via
  `CommandBuffer`. Verified for real: `use_items`'s newly-added
  `#[resource] battle` parameter executed through a real Schedule (no
  `AccessDenied` panic), the RON data loading with the right fields, and
  the effect actually producing a 4-enemy `Battle` - all via temporary
  tests, removed after confirming per the usual convention.
- Also restructured `docs/ideas.md`'s numbering after realizing several
  already-finished items were still sitting in the "Working" list with
  "(fixed)" annotations instead of moving to Done - see the doc's own
  new standing rule at its header.

Full narrative in `docs/journal.md`'s 9/11/26 entries (multiple, same
day).

---

## Previous session (9/11/26) — Title screen upgrades: camera clamp + frozen animation fix

Branch `title-screen-upgrades`: found and fixed the real cause of
the battle-backdrop white cracks reported live (bracket-terminal's
WITH-bg console shader falls back to the flat background color for any
near-black-or-non-opaque texture pixel, same rule CLAUDE.md already
documents for `_no_bg` consoles - the art was never floored the way
every other sprite sheet in this project already is; fixed by flooring
every channel to >=30 across all three images and switching the
fallback color to black as defense-in-depth). Then a new branch for two
requested fixes:

- **Camera never had any bounds awareness.** Added `Camera::
  clamped_top_left`, a shared helper used by both `Camera::new` (the
  title screen's one-shot placement, previously centered on whatever
  random point a map architect picked as `player_start`) and
  `on_player_move` (every real step), so the fixed `DISPLAY_WIDTH x
  DISPLAY_HEIGHT` window never extends past the map's own `SCREEN_WIDTH
  x SCREEN_HEIGHT` bounds - fixes the black-void bug both on the title
  screen and during real dungeon-crawl play near any edge (confirmed by
  the user this should be one shared fix, not two separate ones).
  Exposed a real wrinkle in `components::camera_render_offset` (the
  glide's sub-pixel smoothing), whose own doc comment said it was
  written to deliberately match Camera's un-clamped math exactly - fixed
  by interpolating between the clamped camera position at BOTH ends of
  a glide instead of the player's raw position minus a constant offset.
  Verified with an exhaustive test (every possible target point on the
  map, not just samples), removed after confirming per the usual
  write-then-delete convention.
- **Title-screen background enemies read as frozen, not walking in
  place.** Root cause: `tick_idle_animation_system` only ran once every
  `BACKGROUND_MOVE_INTERVAL_MS` (400ms), and each time only added that
  one triggering frame's real elapsed time, not the ~400ms that had
  actually passed - an idle frame took ~9 real seconds to advance.
  Fixed by moving `tick_animations`/`tick_idle_animation` out of the
  throttled movement schedule and into the schedule that already runs
  every real frame, decoupling "how often does it step" from "how
  smoothly does it animate." Verified for real with screenshots 80ms
  apart - a stationary background goblin visibly cycled several walk-
  in-place poses within under half a second, then stepped to its next
  tile right on schedule. Also ran a real Schedule.execute() against
  this newly-combined system group (mirrors hud.rs's own
  hud_system_execution_tests pattern) to rule out an AccessDenied
  panic, since this was the first time these animation systems ever
  ran alongside map_render/entity_render together - passed, removed
  after confirming, same convention as the camera test above.
- **Not fixed, deliberately out of scope**: a user screenshot of a real
  2-enemy fight showed `enemy_portrait_position`'s fixed coarse-grid
  coordinates landing an enemy against the new backdrop art's own fence/
  wall scenery (tuned back when the background had nothing near the
  edges) - logged in `docs/ideas.md` instead of scope-creeping this
  branch, per the user's own call.

Full narrative in `docs/journal.md`'s 9/11/26 entries.

---

## Previous session (9/11/26) — Real painted battle-arena backgrounds

Real painted battle-arena backgrounds for Forest/Dungeon/Sewer,
replacing the old flat-tinted-glyph-plus-vignette fill - see
`docs/journal.md`'s own 9/11/26 entry for the full narrative (art
iteration, prompt tuning, the clone-stamp fixes). Technical summary:

- New `MapTheme::battle_background_row() -> Option<u16>` (mirrors
  `tile_row`'s exact shape/fallback), backed by a new
  `resources/battle_backgrounds.png` - one full 1280x800 painted scene
  per theme, not a tileable asset (JRPG-battle-backdrop style, chosen
  over more `map_tiles.png`-style tiling specifically because the user
  correctly flagged that path would need far more themed tile variety to
  avoid looking stale).
- Required a genuinely new console, `BATTLE_BACKDROP_CONSOLE` (a 1x1-cell
  console - stretches to fill the whole window as a single glyph, same
  mechanism BATTLE_PORTRAIT_COLS/ROWS's coarse grid already relies on;
  confirmed against bracket-terminal's own `calc_step`/`rebuild_vertices`
  source, not just inferred from behavior). Had to sit below the battle
  screen's own text and creature portraits, which were still bare
  literals (`2`/`3`) since the very start - the first time any "insert
  early, renumber everything after" move in this project's console list
  has needed to go below those two rather than just below the HUD. Both
  promoted to named constants (`FINE_TEXT_CONSOLE`,
  `BATTLE_PORTRAIT_CONSOLE`) as part of the insertion; every console
  index in `main.rs` shifted by exactly 1. A theme with no real
  background art yet still falls back to the original procedural fill,
  unchanged.
- Hit the glyph-32 `cls()` gotcha (see Known Environment Quirks below) in
  a new shape: a single-column font (one glyph = one full-screen image)
  needs 33+ rows just for glyph 32 to land on a valid cell, which would
  have meant an absurd 1280x26400 texture. Fixed with a 6x6 grid
  (7680x4800) instead of a tall single column - a crash from too few
  TOTAL cells, not the previously-documented "wrong row" tiling variant.
- Verification was source-confirmed plus screenshot-partial: got a real
  title-screen screenshot post-renumbering (no crash, no regression to
  consoles below the insertion point), but couldn't reach Class Select
  or a live battle myself - same WSLg synthetic-input unreliability
  already documented below (screenshots work regardless of focus,
  synthetic keyboard/mouse doesn't reliably reach the window). Left the
  game running and asked the user to check those two screens with real
  input instead of re-attempting the same xlib approach.

---

## Previous session (9/8/26) — Debug hotkey fix + branch merge

Same day (9/8/26), a fourth session: one real bug fix, then the full
merge of both outstanding branches into `master`.

**Bug fix**: Class Select's hidden 'D' (Debug) shortcut
(`screens/title.rs`) always called `start_game` (Dungeon Crawl)
regardless of `self.adventure_mode`, predating Battle Arena mode -
pressing D from the Arena's own Class Select silently dropped into a
Dungeon Crawl run instead. Found by the user trying to reach Debug in
the Arena to go find the freshly-fixed Orc Warlord. Now mirrors the same
`match self.adventure_mode` branch the normal roster selection already
uses.

**Merge**: `expand-character-animations` (Death/Victory/Technique
framework, all 6 classes/enemies) and `map-tile-themes` (real per-tile
Forest/Dungeon/Sewer rendering) were merged together, then that combined
branch was merged into `master` - explicit user request ("we should be
able to merge everything to the master"). Order: committed the
outstanding animation-branch work first (git refuses a merge that would
clobber uncommitted changes to files the merge also touches), merged
`map-tile-themes` into `expand-character-animations` (two real
conflicts, both from console-registration numbering - see below), then
merged that combined branch into `master`, which fast-forwarded cleanly
(master had no divergent commits of its own since the branch point back
at the original class-art migration - it had never been updated with
ANY of this work until now).

**The two real conflicts**, both already anticipated (see the
[[project_branch_merge_plan]] note from the previous session):
- `docs/ideas.md`: both branches' own backlog updates, combined rather
  than one replacing the other; the map-themes backlog item was marked
  done and its full write-up moved into the Done section instead of
  staying in Working.
- `main.rs`: `map-tile-themes` had independently inserted 2 new
  dungeon-view consoles earlier in the registration list
  (`MAP_TILE_CONSOLE`, `MAP_TILE_SCROLL_CONSOLE`), shifting everything
  after them by +2 - `CHARACTER_DEATH_CONSOLE`/`CHARACTER_VICTORY_
  CONSOLE`/`CHARACTER_TECHNIQUE_CONSOLE` landed at 31/32/33 instead of
  the 29/30/31 they'd been given on `expand-character-animations` alone.
  Resolved by taking `map-tile-themes`' renumbering as the base and
  re-numbering the three animation consoles on top of it. Also fixed
  along the way (pre-existing on `map-tile-themes`, not introduced by
  this merge, but directly adjacent to what it touched): the top-of-file
  console-index summary comment had drifted out of sync with the real
  registration order (`MAP_TILE_CONSOLE`/`ENTITY_CONSOLE` were listed
  swapped, a leftover from before `MAP_TILE_CONSOLE`'s own z-order fix)
  and was missing everything from `CHARACTER_DEATH_CONSOLE` on; several
  inline builder-chain comments for the tail-end consoles had the same
  kind of staleness. Re-derived the whole list from the actual builder
  chain rather than assumed correct.

**Verification**: the full 0-33 console `cls()` sweep and the builder
chain's actual registration order were checked by hand against the real
`pub const` declarations (not just assumed correct post-merge) - all 34
consoles present exactly once, in the right order. `cargo check`/
`build`/`test` all clean at every step (after the branch-combining
merge, and again after the fast-forward into `master`) - only the
pre-existing `EmptyArchitect` warning plus a new expected one
(`TileType::Water` unused, from the newly-merged map-themes work; a
known, already-documented deferred placement pass, not a regression).

User will push to the remote themselves (see [[feedback_git_push_ownership]]) -
this session did not push.

---

## Previous session (9/8/26) — Two rounds of feedback/content on the animation framework

Same day (9/8/26), a third session: two rounds of feedback/content on
top of the Death/Victory/Technique framework below, still on
`expand-character-animations` (not yet merged, at the time).

**Round 1 - feedback on Rogue's own Flurry animation after seeing it in
a real fight**, all in `OneShotAnimation`/`components.rs`:
- **No more wiggle on a technique animation.** The "Attacking" flash's
  usual shake, layered on top of an animation that already shows real
  motion, read as redundant/busy. `CHARACTER_TECHNIQUE_WIGGLE_CONSOLE`
  (console 32) is gone entirely, not just unused - it was the very last
  console in the registration chain, so removing it needed no
  renumbering.
- **Technique animations are much faster.** `frame_duration_ms` moved
  from a shared global constant onto `OneShotAnimation` itself, so each
  animation can have its own pace - Death/Victory keep the old
  `IDLE_FRAME_DURATION_MS` (350ms), but a new `TECHNIQUE_FRAME_DURATION_MS`
  (80ms) makes an attack read as fast and punchy instead of a slow held
  pose. At the old pace, a 9-frame technique would only get through ~3
  frames before `RESULT_AUTO_ADVANCE_MS` (1100ms) auto-dismissed a
  single-hit ActionResult.
- **A multi-hit/AOE technique's animation now loops** instead of
  freezing on its last frame - a new `OneShotAnimation::repeat` flag,
  decided in `resolve_player_action`'s `BattleAction::Technique` branch
  by checking the item's own `TechniqueEffect` (`battle::
  technique_effect`) for `MultiHit`/`AoeMultiHit` before building the
  animation. Without this, Flurry's 3-hit sequence (or a 9-hit Arrow
  Volley) spent most of its `HitQueue`-driven real-time span frozen on
  frame 1, since one play-through is much shorter than the whole
  multi-hit sequence takes to land.

**Round 2 - the rest of the classes' Death/Victory/technique art plus
Orc Warlord's redo**, six more zips (`Debug.zip`, `Hunter.zip`,
`Barbarian.zip`, `Amazon.zip`, `Mage.zip`, `Orc_Warlord.zip`):
- All three new sheets (`character_death.png`, `character_victory.png`,
  `character_technique.png`) went from Rogue-only to every class that
  has the art: Death/Victory now cover all 6 (Rogue, Debug, Hunter,
  Barbarian, Amazon, Mage); the technique sheet covers the 5 with a real
  AOE technique (Rogue/Flurry, Hunter/Arrow Volley, Barbarian/Whirlwind,
  Amazon/Javelin Volley, Mage/Blizzard) - Debug has none, since its
  "techniques" are cheat items (Victory/Defeat), not real attacks. All
  three sheets stayed at their existing 9-col x 8-row size - no resize
  needed, since the 5 new rows all fit in the headroom already there.
  **One real naming mismatch caught before it became a bug**: the
  Amazon zip's own animation folder was named `Spear_Volley` (after the
  class's weapon), but the actual in-battle item name (what
  `resolve_player_action` actually looks this row up by) is "Javelin
  Volley" - matched on the real name, not the zip's folder name.
- **Orc Warlord's redo fixed the earlier defect for real.** The
  regenerated batch's Fight_Stance_Idle (8 frames, native 32x32) and
  Walk (8 frames, 44x44 padded) both came back as a genuine full
  character this time, confirmed by screenshot - no more thin off-model
  sliver. One new wrinkle: Walk came back with 8 frames, exceeding the
  idle sheet's own 6-column ceiling (`MAX_IDLE_FRAMES`, shared by every
  entity on that sheet) - handled by evenly sampling 6 of the 8
  (`numpy.linspace(0, 7, 6)` -> indices 0,1,3,4,6,7) rather than
  truncating to the first 6, so the walk cycle doesn't visibly skip its
  back half. `enemy_idle_row`/`enemy_battle_row` both got a real "Orc
  Warlord" => Some(6) arm, replacing the old dungeonfont `K` fallback.
- Both sheet-update scripts LOADED the existing PNGs and touched only
  the new rows (blanking each destination row first, per this project's
  standing rule) rather than rebuilding from scratch - the enemy sheets
  in particular hold several other already-shipped enemies that aren't
  in this session's scratch directory anymore, so a from-scratch rebuild
  would have silently erased them.
- `cargo check`/`build`/`test` all clean throughout both rounds (only
  the pre-existing unrelated `EmptyArchitect` warning). Still no live
  screenshot verification - the X11 input-driver issue from the
  previous entry in this file wasn't revisited this session.

**Also this session**: confirmed `map-tile-themes` (a whole separate,
already-built real per-tile map theme system - Forest/Dungeon/Sewer, see
"Previous session (9/8/26) - map themes" below) was never merged into
`pixellab-character-art`/`expand-character-animations`, which is why it
was invisible when the user checked - it only exists on its own branch.
Both branches independently made large edits to `main.rs`'s console
list, so merging will need real conflict resolution. Decision: finish
the character-animation work first (this session's own scope), merge
both branches together afterward.

---

## Previous session (9/8/26) — Death/Victory/Technique framework for Rogue

Same day (9/8/26), an earlier session: built a reusable framework for
played-once character animations (Death, Victory, per-technique) on top
of the class-art migration below, and implemented it end-to-end for
Rogue as the first real case. New branch `expand-character-animations`
(off `pixellab-character-art`) - not yet merged.

**The core addition**: `OneShotAnimation` (`components.rs`) - plays
through a frame list once, then holds on the last frame, as opposed to
`IdleAnimation`'s permanent loop. Deliberately a separate type rather
than a flag on `IdleAnimation` - every existing idle-loop call site would
otherwise need to start handling a "hold at the end" case it never
actually hits. Its actual tick/finished logic (frame advances on
schedule, holds and stops advancing once finished, doesn't skip ahead on
a big time delta) was covered by a real temporary unit test, run once to
confirm, then removed per this project's "verify logic, don't leave
throwaway tests behind" convention - `hud_system_execution_tests` is
still the one deliberate exception to that.

**Three new sheets** (`character_death.png`, `character_victory.png`,
`character_technique.png`), one row per class (or per (class, technique)
pair for the technique sheet - see below), same "own dedicated row
function per sheet" convention every other sheet in this project follows
- full detail moved to `docs/Dungeon_Font_Glyph_to_Cell_Map.md`'s new
"One-shot animation sheets" section, including the two novel wrinkles
this batch introduced: these three share a column count (9) with no
other existing sheet, so they get their OWN forbidden-row number (3) for
the glyph-32 `cls()` gotcha - re-derived, not assumed; and the technique
sheet is keyed by a COMPOUND (class, technique name) identity rather
than by class alone, since a class can end up with several technique
animations over time (explicit user instruction: "assume however that
we will add a lot more battle animations for each class").

**Wiring, in order of where each animation shows**:
- `State::death_animation`/`victory_animation` (new fields, `main.rs`) -
  lazily built and ticked once per frame by `screens/end.rs`'s
  `game_over()`/`victory()`, reset to `None` in `return_to_title()` so a
  fresh run doesn't inherit a stale animation. `draw_end_screen_
  fallen_portrait`/`draw_end_screen_portrait` check these FIRST, falling
  back to the pre-existing rotated-glyph (Death) or still-portrait
  (Victory) logic for a class with no row yet - so this is purely
  additive, no regression for Barbarian/Amazon/Hunter/Mage/Debug.
- `Battle::player_technique_animation` (new field) - set in
  `resolve_player_action`'s `BattleAction::Technique` branch (looked up
  by the item's own class+name, same identity `Stats::record_ability_
  used` already keys on), ticked alongside the existing battle-idle
  frame counter, cleared in `dismiss_action_result` the instant `turn`
  returns to `Filling` so it can never linger into the next race.
- `BattleVictory::portrait_animation` (new field) - built once in
  `finish_battle`, ticked by `battle_victory_tick`, which now re-inserts
  its own ticked snapshot back into the resource every frame (the same
  "snapshot, mutate, re-insert" shape `battle_tick` already used for
  `battle` itself - this screen just never needed it before).
- `draw_battle_arena` gained two new parameters (`technique_glyph`,
  `victory_glyph`) that each override its existing 3-tier player-portrait
  fallback when `Some` - never both at once, since a technique animation
  only plays during a live battle and a victory animation only once the
  battle's already over.
- Four new consoles (29-32: `CHARACTER_DEATH_CONSOLE`, `CHARACTER_
  VICTORY_CONSOLE`, `CHARACTER_TECHNIQUE_CONSOLE`, `CHARACTER_TECHNIQUE_
  WIGGLE_CONSOLE`), appended at the end of the registration chain like
  every other battle/end-screen-only console before them - no dungeon
  HUD z-order constraint to respect for any of the four.

**Rogue's own first real case**: sourced from `Rogue.zip` (2026-09-08) -
`The_hooded_figure_slumps_forward_as_its_knees_buck/south` (Death),
`Victory/south` (Victory), `Flurry/east` (the AOE technique's own
animation - east, not south, matching the "battle portraits always face
east" convention). `Breathing_Idle` in the same zip is deliberately
unused per explicit standing instruction - it "doesn't look that great"
as a battle-portrait loop. All three source animations came back 44x44
padded (same center-crop-to-32x32 treatment `Walk` already needed) with
exactly 9 frames each, which is what set `EXTRA_ANIM_COLS`.

**Verification**: `cargo check`/`build`/`test` all clean (only the
pre-existing unrelated `EmptyArchitect` dead-code warning). Live
in-game screenshot verification was attempted but blocked by an
environment issue, not a code issue - see the new Known Environment
Quirks entry below. The next session picking this up should attempt a
real playthrough screenshot (Rogue into a fight, use Flurry, win or die)
once that's resolved, before treating this as fully proven rather than
just "compiles and follows every proven pattern from the class-art
migration exactly."

**Still to come, per explicit user instruction**: every other class gets
its own Victory, Death, and one-or-more per-technique animations,
delivered in future zips one class at a time - "I will give you all the
rest after we get the rogue going." Adding each one is: extract the
zip, build/verify the three sheet updates (or extend cols if a class's
frame count differs from 9), add one row-function match arm per sheet
(two for the technique sheet, per new technique) - no further
architecture work anticipated, that's the point of building this as a
framework now.

---

## Previous session (9/8/26) — Character art migration

That session (9/8/26) migrated all 5 playable classes (Barbarian, Rogue,
Amazon, Hunter, Mage) plus the hidden dev/test "Debug" class onto real
PixelLab.ai animated art across three new sheets, replacing the old
static dungeonfont class glyph everywhere it was still used. A shorter
earlier arc the same session (Battle Arena survivability simulation) got
put on pause partway through - see its own note below. Full technical
detail - row assignments, PixelLab's zip format, every gotcha - now
lives in `docs/Dungeon_Font_Glyph_to_Cell_Map.md`, which doubles as the
master reference for this system now, not just the dungeon font.

**Three new sheets, one row per class:** `character_idle.png` (dungeon/
Arena/Class-Select walk loop, 128px cells upscaled 4x from native 32x32
art), `character_battle.png` (a brand-new battle-screen portrait
animation that didn't exist before - battle previously always showed a
static glyph), `character_portrait.png` (a still pose replacing the old
class glyph at every remaining static site: Class Select's
non-highlighted row, the dungeon HUD portrait, both the in-battle and
run-ending Victory screens, and the Game Over screen's rotated fallen
pose). Each class's PixelLab zip supplies `Walk` (6 frames) for the idle
sheet and `Fight_Stance_Idle/east` (8 frames) for the battle sheet;
`Breathing_Idle` and every rotation but `south` are still unused
(directional facing is a real, unscoped architecture question - the
game has no concept of entity facing today).

**Four real bugs found and fixed, three of them genuinely novel classes
of bug for this project:**
- **The tiled-portrait bug, twice, on two different sheets.** A plain
  console's `cls()` fills every undrawn cell with glyph 32 by default;
  which (row, col) that lands on depends on THAT sheet's own column
  count. First hit on `character_battle.png` (8 cols → row 4) when Mage
  was assigned there, filling the entire battle screen with tiled Mage
  portraits. Second hit on `character_idle.png` (6 cols → row 5) when
  Debug was assigned there, filling the entire title/Adventure-Select
  screen with tiled Robot portraits. Both confirmed by user screenshot.
  Fixed by giving each sheet its own dedicated row-assignment function
  that permanently skips its own forbidden row, instead of reusing a
  shared mapping that only "happened" to still be blank there.
- **Leftover old art bleeding through new art.** Mage was the first
  class composited and that first pass skipped "blank the destination
  cell before pasting" - wherever the new frame's silhouette didn't
  fully cover the old placeholder's, old pixels stayed visible
  underneath. User caught it by eye ("the mage class still has the
  original icon below the walking animation") and correctly overruled
  an initial wrong guess that it was a code bug ("It's the artwork, not
  the code"). Confirmed via a numpy pixel-diff before fixing.
- **PixelLab pads some animation canvases larger than 32x32 without
  saying so anywhere reliable.** `metadata.json` claims every frame is
  32x32; in reality only `rotations/south.png` and (mostly)
  `Fight_Stance_Idle` actually are - `Walk` came back 40-48px for 5 of 6
  classes, only Mage's genuinely 32x32. The character's own pixel size
  stays constant regardless of canvas size (confirmed via bbox
  comparison); PixelLab just gives more-motion animations more padding.
  Caused two symptoms: most classes' walk animations rendering visibly
  smaller than intended (a padded canvas got proportionally LESS
  upscale toward the sheet's fixed 128px cell), and Debug's oversized
  `Fight_Stance_Idle` frames bleeding into neighboring battle-sheet
  cells (a native-resolution direct-paste doesn't crop, it just draws
  the source's full size). User caught this too, asking directly
  whether image sizes were being changed. Fixed by center-cropping
  every frame to a true 32x32 before any further processing.
- (Non-art) `choose_battle_action`'s flee rule fled below 25% HP
  regardless of whether the entity actually carried a Healing Potion,
  accomplishing nothing - per explicit user feedback ("no reason to
  flee unless the character has a potion"), now gated on carrying one.

**Battle Arena survivability simulation - paused, not abandoned.** Built
a second headless-bot simulation (`arena_class_survivability_report`,
mirroring the existing Dungeon Crawl one) and found/fixed three real
bugs along the way: a genuine `bracket_pathfinding::DijkstraMap` library
bug (seed tile's own distance never actually written, causing stable
navigation cycles right next to a target - replaced with a from-scratch
BFS since uniform movement cost makes that the textbook-correct
algorithm anyway), a shopping-priority ordering bug (affordability
checked after adjacency, not before, causing endless backtracking once
gold ran out), and a missing weapon-purchase policy (Barbarian dealt 0
damage with no weapon vs. any enemy with Defense ≥ 1). With all three
fixed the simulation produced trustworthy data - zero clears for any
class against the current wave/boss tuning, Mage weakest, Level 3 a
wall for everyone - but the user paused further balance work here once
it became clear the bot doesn't use any class abilities at all, only
basic attacks/items: "the bot is not using all that it could lets put a
pin in this." Numbers are a real lower bound, not a verdict.

**Enemy art started (same day, follow-on session): Goblin migrated,
enemies get their own dedicated sheets.** A design conversation preceded
the code (per CLAUDE.md's convention for architectural changes): enemies
get their own `resources/enemy_idle.png`/`resources/enemy_battle.png`
rather than more rows on the character sheets above, both because those
only had one free row left each (nowhere near enough for the 7-enemy
roster) and because enemies are a different lookup domain - keyed by
`Name`, not class. Full row-mapping now lives in `docs/
Dungeon_Font_Glyph_to_Cell_Map.md`'s new "Enemy sheets" section.

Wiring this in touched more of the codebase than the class-art session
did, because enemies are dungeon-view entities that also appear in
multi-enemy battles - both paths the player's own art didn't need to
share:
- `components.rs`: a new `IdleSpriteSheet::EnemyIdle` variant,
  `enemy_idle_row`/`idle_frames_for_enemy` and `enemy_battle_row`/
  `enemy_battle_glyph` (own dedicated row functions, per the standing
  "every sheet needs its own" rule - each independently re-derived its
  own forbidden row rather than assuming safety from character_idle_row/
  character_battle_row's matching column counts).
- `spawner/template.rs`: enemy spawn now calls `idle_frames_for_enemy`
  (name-keyed) instead of the old class-blind `idle_frames_for`.
- `systems/entity_render.rs`: all three IdleSpriteSheet match sites
  (camera-at-rest plain/glide, camera-panning scroll) gained an
  `EnemyIdle` arm - Rust's exhaustiveness check caught every one that
  would otherwise have been missed.
- `main.rs`: a THIRD "insert early, renumber everything after" console
  move (same pattern CHARACTER_IDLE_* used twice before) - the new
  ENEMY_IDLE_CONSOLE/ENEMY_IDLE_SCROLL_CONSOLE/ENEMY_IDLE_GLIDE_CONSOLE
  trio needed to land BELOW the HUD/Ability Bar layer like every other
  dungeon-view console, pushing HUD_CONSOLE through
  END_SCREEN_FALLEN_PORTRAIT_CONSOLE up by 3 (12→26). ENEMY_BATTLE_CONSOLE/
  ENEMY_BATTLE_WIGGLE_CONSOLE (27, 28) had no such constraint - the
  battle screen has no dungeon HUD to stay under - so those were simply
  appended, matching CHARACTER_BATTLE_CONSOLE's own precedent.
- `battle/mod.rs`/`screens/battle.rs`: `EnemyCombatant` gained its own
  `battle_idle_frame`/`battle_idle_elapsed_ms` (each enemy in a fight
  needs an independent counter, same reason gauge/flash/statuses already
  are per-enemy); `draw_battle_arena`'s per-enemy portrait loop gained a
  two-tier fallback (enemy_battle_glyph when the enemy has a row, else
  the old dungeonfont glyph) mirroring the player's own longer three-tier
  one.

One real, deliberate rotation difference from the class-art convention:
enemies' battle stance uses PixelLab's `south-west` rotation, not `east`
like every class - correct, not a bug, since the battle screen's layout
puts the player bottom-left and enemies upper-center/right facing off,
so an enemy facing toward the player (south-west) reads right where
`east` (matching the player's own rightward-facing stance) would not.

Verified end-to-end with a real screenshot (per CLAUDE.md's bracket-lib
rendering rule - a clean build proves nothing about actual pixels): a
live Battle Arena run showed the Goblin's new walk animation correctly
scaled and composited in the dungeon/wave view, then its new animated
battle-idle portrait rendering cleanly (no bleed, no leftover placeholder
art, transparent background against the arena backdrop) through a real
fight. Used a small ad hoc python-xlib + XTEST driver (no project skill
existed yet for driving this GUI app) to launch the game, send real
keypresses, and capture window screenshots.

Also added two backlog items per user request: more dungeon tile sets,
and a follow-on refactor of how maps get generated and tiles get
assigned (`docs/ideas.md`, items 7-8) - explicitly flagged as a
DIFFERENT kind of problem from the character/enemy/NPC sheet work above
(map-console tile graphics, not an animated actor's sprite), needing its
own design conversation before starting.

Remaining enemy rows still to do: Orc, Ogre, Ettin, Goblin Chieftain, Orc
Warlord, Ettin Overlord (6 more, one row each, same pipeline).

**Second follow-on session, same day: 6 of those 7 done, plus one real
art defect caught and held back.** User sent all 7 remaining zips at
once (asked "how many can I send at the same time" - answer: exactly 6
fit the sheets' free rows, but sending all 7 including the not-yet-
existing "Ogre Warlord" was fine too, since sheets just grow taller).
Both sheets resized 8→9 rows in one pass to fit the full roster.

**A design conversation happened before any of the 7 got processed**:
the zip list included an "Ogre Boss" with no matching `template.ron`
entry (only Goblin Chieftain/Orc Warlord/Ettin Overlord exist as
bosses) - asked the user directly rather than guessing. Turned out to be
a genuinely new enemy, "Ogre Warlord," and a second question (which
level(s) should it guard) revealed something worth knowing generally:
`Templates::spawn_boss` (`spawner/template.rs`) already picks randomly,
weighted by `frequency`, among EVERY `boss_only` template matching the
target level - so it already supported more than one possible boss per
level with zero code changes. User chose "both levels 1 and 2" - Ogre
Warlord is now a second possible pick alongside Orc Warlord (level 1)
and Ettin Overlord (level 2), placeholder stats (hp13/dmg3/speed4)
deliberately between its two level-mates. Glyph `e` (confirmed free in
the glyph-map doc) - `F` was tried first and rejected, already claimed
by Fireball's technique icon; `W`'s own "no longer free" precedent in
that doc is exactly the kind of check that caught this.

**One real, confirmed art defect: Orc Warlord's own zip.** Both its
`Walk` and `Fight_Stance_Idle` animations came back as a thin,
~9px-wide off-model sliver in a 44×44 canvas, not a full character -
confirmed genuinely broken (not a cropping bug on this project's side)
by inspecting the RAW un-cropped source frames directly, and confirmed
isolated to just the animations (its own static `rotations/south.png`
pose looked completely correct). Held back rather than shipped, the
same call made for Amazon's Walk earlier - `enemy_idle_row`/
`enemy_battle_row` have no "Orc Warlord" entry, so it still renders on
its old dungeonfont glyph (`K`), no regression. Row 6 (this sheet's
otherwise-next-free row) is left deliberately blank on both sheets,
reserved for it once a redone batch arrives - checked by rebuilding both
sheets from the RAW source zips a second time (not patching the
already-saved file) once this was caught, so the shipped sheets never
contained the broken frames at all.

Verified two of the six new enemies live (not just the sheet pixels) -
Orc's walk animation and battle portrait both confirmed via real
screenshots mid-fight, same clean result as Goblin's original
verification; the remaining four use the identical code path with no
new logic branches, so weren't each individually screenshot-checked
in-game (their sheet rows were still checked pixel-by-pixel via a
checkerboard-background preview before wiring anything in).

---

## Previous session (9/7/26) — Dungeon Crawl economy

Built a whole Dungeon Crawl economy from scratch - gold, a guaranteed
loot chest, a between-floor shop, and a first data-driven starting-kit
balance pass - plus three real bugs found and fixed along the way, one
via a user screenshot, one via the balance simulation itself, one via
actual playtesting.

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
- **The ad hoc python-xlib/XTEST driver used to screenshot-verify this
  game cannot reliably deliver keyboard input on this WSLg setup - traced
  one level deeper this session (9/8/26), not just re-observed.**
  `XGetInputFocus` on this display reports focus `None` even immediately
  after `set_input_focus` on the game's own window (both `RevertToParent`
  and `RevertToPointerRoot` tried), and even after a synthetic click
  (`XTEST` `ButtonPress`/`ButtonRelease` warped onto the window first) and
  an EWMH `_NET_ACTIVE_WINDOW` client message sent to root - none of the
  three normal ways to acquire X11 focus took effect. `ps aux` shows no
  window manager process at all (no weston/mutter/openbox/etc.), which is
  consistent with WSLg's actual architecture: each X11 window is really a
  proxy for a Win32 window on the Windows host, and keyboard focus is
  ultimately tracked by the Windows desktop's own foreground-window
  state (via WSLg's RDP-style channel), not by anything an X11 client
  running inside the WSL side can request through the X protocol alone.
  A background shell process has no way to bring the corresponding Win32
  window to the host's foreground, so XTEST key events get delivered to
  whatever (or nothing) the host currently has focused instead of this
  game's window - explaining the earlier-observed "1-2 events then decay"
  pattern as a symptom of this, not a separate flakiness bug. Screenshots
  themselves (`XGetImage` on the window) work fine regardless, since
  those don't depend on focus - it's specifically synthetic keyboard/
  mouse input that's unreliable. Until a real fix is found (something
  that can toggle actual Windows foreground-window state from inside
  WSL), don't sink more time re-attempting the same xlib approach - rely
  on code-level verification (matching an already-proven console/render
  pattern exactly, plus a real unit test for any non-rendering logic)
  and ask the user for a real screenshot when a rendering-specific detail
  genuinely needs eyes on it.
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
- **A plain console's `cls()` fills every never-drawn-this-frame cell
  with glyph 32 by default, and which (row, col) that lands on depends
  entirely on the CURRENT sheet's own column count — confirmed to bite
  twice, on two different custom sheets.** `row = 32 / cols`,
  `col = 32 % cols` (integer division/modulo). First occurrence:
  `character_battle.png` (8 cols) put glyph 32 at row 4, col 0; Mage's
  class was assigned row 4 there (reusing a shared row mapping that
  happened to work fine on a different, 6-column sheet), and the entire
  battle screen — the console spans the full display — filled with
  tiled Mage portraits on every screen in the game, not just battle.
  Second occurrence, same failure class, different sheet: `character_
  idle.png` (6 cols) put glyph 32 at row 5, col 2; the hidden "Debug"
  class got assigned row 5 there, and the title/Adventure-Select screen
  filled with tiled Robot portraits. Both confirmed by user screenshot,
  both fixed by giving that specific sheet its OWN dedicated
  row-assignment function that permanently reserves whatever row `32 /
  cols` computes for it, rather than reusing a mapping from a
  different-column-count sheet. One sheet in this project (`character_
  portrait.png`) is a genuine exception, not a third near-miss: it only
  ever populates column 0 of any row, and glyph 32's column there (2) is
  guaranteed blank no matter which row it lands on — safe to share a
  row mapping ONLY when a sheet has that specific property, never by
  default.
- **Third occurrence (9/11/26), a genuinely different failure shape from
  the two above: a crash, not silently-wrong content.** Both prior hits
  had PLENTY of rows/cols to contain glyph 32 somewhere - the bug was
  live content sitting on the wrong one. `resources/battle_backgrounds.
  png` (one glyph = one full 1280x800 painted scene, for the new battle-
  arena background art) started as a single-column sheet, 3 rows - nowhere
  NEAR enough total cells to contain index 32 at all. `FontScaler::
  glyph_position`'s unsigned subtraction underflows immediately the
  moment `cls()` first runs (a real panic, "attempt to subtract with
  overflow", at launch, before any content ever draws), not a rendering
  glitch to catch on a screenshot. Minimum total cells needed to safely
  contain index 32 is `cols * rows >= 33`, roughly independent of the
  grid's shape (33 slots either way) - the trap is picking a shape that
  technically satisfies that but produces an impractical texture, e.g.
  padding a 1-column sheet to 33 rows tall (1280x26400 - risks exceeding
  a GPU's max texture dimension for almost no real content). Fixed with a
  6x6 grid (7680x4800) instead - same total padding, much saner aspect
  ratio, comfortably under typical GPU texture limits.
- **PixelLab.ai does not export every animation state at a consistent
  canvas size, and its own `metadata.json` doesn't say so.** Confirmed
  across 6 real character batches: `rotations/south.png` is reliably
  32x32, `Fight_Stance_Idle` was 32x32 for 5 of 6 classes, but `Walk`
  varied per class (Hunter 48x48, Rogue/Barbarian/Debug 44x44, Amazon
  40x40, only Mage genuinely 32x32) - and one class's `Fight_Stance_Idle`
  (Debug) was also 44x44. Verified via bbox comparison across
  differently-sized canvases for the same class that the character's own
  absolute pixel size and center position stay constant regardless of
  canvas size - PixelLab simply pads more room around an animation with
  more motion range (arm/leg swing during a walk cycle) to avoid
  clipping, it does not scale the character itself. Trusting the canvas
  size at face value (resizing "whole canvas to target cell size," or
  direct-pasting at "native" resolution) produces two different visible
  bugs depending which sheet: a resize-based sheet renders the character
  smaller than intended for any class whose canvas got padded (since a
  bigger canvas gets proportionally LESS upscale toward the same fixed
  target cell size), while a native-paste sheet lets the oversized
  frame's extra pixels bleed into neighboring cells (a plain image paste
  at a position draws the full source size, it doesn't crop to fit).
  Fix: always check each animation state's real PNG dimensions before
  compositing, and center-crop to exactly 32x32 whenever a canvas is
  larger than that, before any further processing.

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