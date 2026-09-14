# Map Tile Theme Guide

Master reference for creating and integrating a new dungeon-map tile
theme (real pixel-art floor/wall textures, one set per `MapTheme`
implementation) - the map-rendering counterpart to
`Dungeon_Font_Glyph_to_Cell_Map.md`, which covers characters/enemies/
icons instead. Read this FIRST before generating a new theme's tileset
or touching `map_builder/themes.rs`.

**Status as of 2026-09-08: implemented and shipped for three themes -
Forest, Dungeon, Sewer**, randomly picked per generated dungeon level via
`map_builder::dungeon_theme_pool()` (Battle Arena's own shop/wave maps
still always use Forest specifically, unchanged). Real per-tile textures
are live in the dungeon-crawl view - verified with real screenshots, both
camera-at-rest and mid-glide/panning, and again after adding Dungeon/
Sewer. `TileType::Water` exists in the data model but isn't placed by
any generator yet (see "Deferred to a later design pass" below).

---

## Why a new system at all

Today's `MapTheme` trait returns exactly one dungeonfont *glyph* plus an
RGB tint per `TileType` (`Floor`/`Wall`/`Exit`/`Counter`) - real content
is a single colored ASCII character (`.`/`#` for Dungeon, `;`/`"` for
Forest), not a pixel-art texture. The goal here is real per-tile
textures with actual variety (multiple distinct floor/wall looks per
theme, randomly mixed), the same "give it real art" upgrade the
character/enemy sprite work already did for entities - see
`docs/ideas.md` items 7-8 and `project_enemy_sprite_sheets` /
`project_idle_animation_art` in memory for that precedent.

## The 16-cell template

Every theme's tileset is a fixed 4-column x 4-row grid, 32x32px per
cell (native resolution - map tiles already render at 32px, no upscale
needed the way `character_idle.png` needed one). Cells are numbered 1-16,
row-major (left to right, top to bottom), matching how the user
describes cell ideas to the generator.

| Row | Cells | Purpose |
| --- | --- | --- |
| 1 | 1-4 | **Floor tiles** - basic floor variants. Part of the FLOOR pool (with row 3) from day one. |
| 2 | 5-8 | **Wall tiles** - basic wall variants. The entire WALL pool for now (row 4 joins it later). |
| 3 | 9-12 | **Themed floor tiles** - additional floor variety (more decorative/thematic than row 1's base set). Part of the same FLOOR pool as row 1 - randomly picked from either, 8 variants total. |
| 4 | 13-16 | **Special wall tiles** - impassable "feature" tiles (water now; river segments, stumps, etc. later). NOT part of the random wall pool yet - placement is deliberate/targeted, and the logic for placing these doesn't exist yet ("later we can work on the logic on how to place row 4"). |

**Confirmed 2026-09-08: the FLOOR pool is rows 1+3 combined** (8
variants total - basic + themed floor mixed together, picked randomly
from either), **the WALL pool is row 2 only** (4 variants) until row 4's
placement logic exists. Row 4 is not part of any random pool - it's
placed deliberately once the deferred logic below gets designed.

**Cell #1 and #5 MUST be the theme's plain, unremarkable default look**
(confirmed 2026-09-08, right after the first real playtest) - the
generation algorithm (see "Generation algorithm" below) bases the ENTIRE
map on these two before anything else gets layered on, so they need to
be genuinely the plainest, most generic floor/wall texture in the set -
not a variant with heavy detail; anything more distinctive belongs
somewhere else in the 16.

## Generation algorithm

**Not a uniform per-tile random pick across the whole variant pool** -
tried first, confirmed too noisy in real play: every tile rolling
independently gave floor and wall no larger-scale pattern to read by,
making them hard to tell apart at a glance even though the art itself
was fine. Replaced same-day with two different rules, since floors and
walls actually needed opposite treatment:

- **Every Floor/Wall tile starts on variant 0** (cell #1/#5 - see above)
  before either rule below runs, so the plain default stays the
  dominant texture across the map, not just one option among equals.
- **Wall accents** - a scattered MINORITY of individual wall tiles
  (`WALL_ACCENT_CHANCE_PCT`, 30% as of 2026-09-08) swap to a random
  non-default wall variant, tile by tile, no blocks. A wall is usually
  only one tile thick, so there's no "width" for a block to read any
  differently from a scatter - individual accent tiles (a boulder here,
  a rubble patch there) among mostly-default wall is the natural look.
- **Floor patches** - a handful of contiguous, randomly-placed,
  randomly-sized circular blobs (`FLOOR_PATCH_COUNT_MIN/MAX`,
  `FLOOR_PATCH_RADIUS_MIN/MAX`), each stamped with ONE random Patch-style
  floor variant (see "Per-variant placement style" below) across every
  Floor tile the blob overlaps. This is what actually produces "a block
  of leaves, a block of moss" instead of noise - floors are big open
  areas where a whole natural-looking REGION of one texture reads right,
  the opposite shape from the wall accents.
- **Floor scatter** - the floor-side equivalent of wall accents:
  individual, sparse tiles (`FLOOR_SCATTER_CHANCE_PCT`, 6% as of
  2026-09-08 - deliberately much rarer than the wall accent's 30%) swap
  to a random Scatter-style floor variant, one tile at a time, never a
  block. Runs after floor patches, so a scattered accent can still land
  on top of one (a torch on a mossy patch of floor is perfectly
  plausible).

### Per-variant placement style

**Not every non-default variant wants the same treatment - confirmed
the hard way, 2026-09-08.** The floor-patches rule above originally
applied to EVERY non-default floor variant uniformly, including things
like Dungeon's "torchlight glow" - patching that as a whole region
produced an actual wall of individual torch-light pools, which no real
dungeon would have. The fix: `MapTheme::floor_variant_style(variant) ->
VariantStyle` (`Patch` or `Scatter`), checked per floor variant before
deciding whether it's eligible for the patch loop or the scatter pass
above. Defaults to `Patch` for every variant (the common case), so a
theme only needs to override the specific variants that are actually
discrete point fixtures rather than a spreadable ground cover.

**The rule of thumb for classifying a themed-floor cell (9-12):** if it
reads as "a small fixture sitting on the floor" (a torch, a grate, a
pile of bones, a pillar base) - `Scatter`. If it reads as "an area of
different ground texture" (moss, a dirt patch, a puddle, an algae bloom)
- `Patch`. Confirmed classifications so far: Dungeon's whole row 3
(cells 9-12: drain grate, bones/debris, pillar base, torchlight glow)
is Scatter, all four - none of them read as spreadable. Sewer splits:
cells 9-10 (drainage grate, rubble/debris) Scatter, cells 11-12 (shallow
puddle, glowing algae) Patch. Forest's row 3 (dirt path, path fork,
roots, flowers) is left on the Patch default - a path/root/flower area
reads fine as a region, at least for now (an actual connected path is
the "deferred to a later design pass" placement problem, not this one).

**This is also the direct answer to "how easy is it to have special
settings for each theme"**: very - `MapTheme` was already the natural
per-theme extension point (see `tile_row`, `floor_color`, etc.), so
adding one more override follows the exact same shape. The same pattern
extends to anything else that turns out to need per-theme (or even
per-variant) tuning later - a new `MapTheme` method with a sensible
default, overridden only where a theme actually needs to differ.

Implementation: `MapBuilder::assign_tile_variants` (`map_builder/mod.rs`),
called from every real map-construction path (`MapBuilder::new`,
`new_arena_shop`, `new_arena_wave`) after all tiles are in their final
TileType. Tuning knobs are the consts just above it in that file -
adjust patch/scatter count, radius, or chance there if a theme's result
still reads too busy or too sparse once real art is in.

## Numbering convention

Every theme's 16 tile ideas get listed as ONE flat numbered list, 1-16,
in the same row-major order as the table above - number 1 is row 1's
first floor tile, number 16 is row 4's last special-wall tile. The
number alone says which row/category a tile is in (1-4 floor, 5-8 wall,
9-12 themed floor, 13-16 special wall) - no need to restate the category
alongside each one. This is the exact format to use whenever specifying
a new theme, to me or to the generator directly, and the exact format
any theme's finalized cell list gets recorded in under "Themes done so
far" below.

Example (Forest, illustrative only - not a finalized list):
1. Grass
2. Moss
3. Dirt
4. Leaves
5. Trees
6. Big rock
7. Rubble / small rocks
8. Tree stump
9. *(themed floor, 9-12)*
13. *(special wall, 13-16 - water goes here)*

## Sheet & console architecture (as built)

One shared atlas font, `resources/map_tiles.png`, two consoles total
(`MAP_TILE_CONSOLE` + `MAP_TILE_SCROLL_CONSOLE`, mirroring `console 0`/
`MAP_SCROLL_CONSOLE`'s plain/fancy pair for the camera-at-rest vs.
camera-panning cases) - NOT one console per theme. A dedicated console
per theme would multiply an actually-expensive resource (registered
bracket-lib consoles) the same way "one sheet per character" would have
for the animation work; the fix there was one sheet with more ROWS, and
the same fix applies here. Both consoles had to be inserted BELOW even
`CHARACTER_IDLE_CONSOLE`/`ENEMY_IDLE_CONSOLE` in z-order (registration
order), not just below the HUD - this is the base map layer everything
else stands on, a stricter requirement than the entity sheets had. Each
theme owns a fixed 4-row block (grows downward as new themes are added,
exactly like `character_idle.png`/`enemy_idle.png` grow downward per
class/enemy) - Forest is rows 0-3, Dungeon 4-7, Sewer 9-12 (row 8
skipped - see the forbidden-row gotcha below); the next theme is 13-16.

**Adding a theme to the random pool is now one line, not a
range/match edit** - `map_builder::dungeon_theme_pool()` returns every
theme `MapBuilder::new` can pick from; `rng.random_slice_index` picks
one. Replaced the old `rng.range(0, 2)` + match the same day Sewer
became the third theme, specifically because hand-updating a hardcoded
range number for every new theme was exactly the kind of bookkeeping
this project tries to avoid baking into new code (2026-09-08 - "I want
to be able to put a few more themes" was the direct ask). A brand new
theme needs: its own `MapTheme` impl in `themes.rs` (`tile_row` pointing
at its own 4-row block), its own block composited into `map_tiles.png`,
and one more entry in `dungeon_theme_pool()`'s vec literal - nothing
else changes.

**Real gotcha hit at runtime, not caught by `cargo build`**: the
glyph-32 panic documented in CLAUDE.md ("a custom `with_font` sheet
needs a glyph grid big enough to cover index 32") applies here too, and
bit immediately - a fresh 4-row (16-cell) `map_tiles.png` doesn't even
contain index 32 (at `MAP_TILE_COLS = 4`, that's row 8), so `cls()`'s
default fill crashed with "attempt to subtract with overflow" the
instant the console existed, before any real content was even drawn.
Fixed by padding the sheet to 12 rows (leaves rows 4-11 blank for now,
comfortably past row 8). **Row 8 is now a permanently forbidden row on
this sheet, same as the character/enemy sheets' own forbidden rows** -
whichever theme's row block would otherwise start there needs to skip
to the next row instead. At 4 rows/theme, that's theme 3 (rows 8-11
normally) - it needs to start at row 9 instead, leaving row 8 blank.
Recheck this math fresh if `MAP_TILE_COLS` ever changes.

**Second real gotcha, a genuine regression caught by the user in actual
play, not by any build/test step**: `MAP_TILE_CONSOLE` was first
registered `with_simple_console` (WITH an opaque background, copying
console 0's own registration) on the reasoning that these tiles are
meant to be fully opaque floor anyway. That reasoning missed the actual
precondition console 0 relies on: a WITH-bg console paints a solid
opaque quad over its ENTIRE grid on every `cls()`, which is only safe
for a console something ALWAYS fully repaints on literally every tick
regardless of which screen is showing (console 0 gets that from
map_render OR battle's own arena-paint code OR title's own background,
between them covering every screen). `MAP_TILE_CONSOLE` is only ever
painted by map_render's tile loop, which doesn't run during the title
screen or the battle screen - on those two, it was just `cls()`'d to
solid opaque black sitting on top of everything below it in z-order,
blacking out the title background, the battle arena background, AND
(since it registers above console 2 too) the battle screen's HP/ATB
text. Dungeon exploration alone looked completely fine, which is
exactly what made this easy to ship without noticing - the console's
own most common use case fully repaints it every frame. Fixed by
switching to `with_simple_console_no_bg` (transparent clear, matching
`CHARACTER_IDLE_CONSOLE`/`ENEMY_IDLE_CONSOLE`'s own precedent) and, since
a `_no_bg` console culls any pixel under 0.1 in all 3 RGB channels,
adding the same near-black-pixel flooring (≥30/channel) the character/
enemy sheets already needed so real dark texture detail doesn't get
eaten. **General lesson for any future console here**: a WITH-bg plain
console is only a safe choice when the system that owns it is
guaranteed to fully repaint it on every single tick, on every screen -
otherwise it must be `_no_bg`.

**Third real gotcha, another genuine regression caught by the user in
actual play**: the original "below the HUD/Ability Bar" placement
(registered after ENTITY_SCROLL_CONSOLE, before CHARACTER_IDLE_CONSOLE -
correct for CHARACTER_IDLE_CONSOLE/ENEMY_IDLE_CONSOLE's own precedent)
turned out to be necessary but not sufficient for this specific console.
The OLD plain dungeonfont entity layer (the literal, unnamed "console 1"
from before this session, used by anything with no real idle-frame art
at all - the Shopkeeper, items lying on the floor, any not-yet-migrated
enemy) sits right at the very start of the chain, index 1, with no
gap to insert before it. `MAP_TILE_CONSOLE` at its old index (6) painted
real tile textures ON TOP of that whole entity layer, not under it -
the Shopkeeper vanished on any Floor/Wall tile, and any dungeonfont-
fallback enemy (Orc Warlord, still pending its own art redo) visibly
"popped" in and out of existence depending on whether it was standing
still (hidden, on the affected console) or mid-glide (visible, on
GLIDE_CONSOLE - a console already safely positioned above this whole
problem). Fixed by swapping `MAP_TILE_CONSOLE`/`MAP_TILE_SCROLL_CONSOLE`
into slots 1 and 5 respectively (right after console 0 and
MAP_SCROLL_CONSOLE) and promoting the old literal "console 1" to a real
named constant, `ENTITY_CONSOLE`, moved to slot 6 - see main.rs's own
doc comments on these four consts for the exact reasoning. **General
lesson, sharper than the one above**: "below the HUD" is the necessary
condition for any dungeon-view console, but a console drawing the base
MAP layer specifically also needs to be below literally every entity
console, including the oldest, most foundational one (console 1) - not
just the newer named ones added since. Check this explicitly for any
future console that draws terrain/background rather than a foreground
entity.

**Wall/floor contrast note (not a bug, a follow-up tuning pass)**: once
real textures were live, some themes (Sewer specifically) read as too
visually flat - wall and floor variants at the same brightness didn't
have enough inherent contrast in the art itself. Fixed with a flat
darkening multiply (`WALL_TEXTURE_SHADE`, `components.rs`) applied to
every real-texture WALL tile's tint, on top of the existing visible/
remembered brightness - a code-side fix that benefits every theme
uniformly rather than needing new art or a per-theme special case.

## Data model (as built)

- `TileType::Water` - impassable and opaque with ZERO extra logic
  needed: `can_enter_tile`/`is_opaque` (`map.rs`) already only
  special-case `Floor`/`Exit` as passable, so any other variant (like
  `Counter` already did) is automatically blocked and blocks sight. Not
  placed by any generator yet - exists so the data/render side is ready
  whenever the deferred placement logic below arrives.
- `Map::tile_variant: Vec<u8>` - which texture each Floor/Wall tile
  shows, assigned once per map in `MapBuilder::assign_tile_variants`
  (called from `MapBuilder::new`, `new_arena_shop`, and `new_arena_wave`
  - every real map-construction path) and stored rather than re-rolled
  per frame. Not a uniform per-tile random pick - see "Generation
  algorithm" above for the actual default-base/accent/patch shape.
- `MapTheme::tile_row(&self) -> Option<u16>` - `None` (the trait's
  default) for a theme still on the old single-glyph rendering, `Some`
  for a migrated one's starting row. `components::tile_render_at`
  checks this first; falls through to the old `tile_to_render`/
  dungeonfont path for any TileType without real art yet (Exit/Counter/
  Water even on a migrated theme) or for a theme with no row at all.

## Deferred to a later design pass

Neighbor-aware/targeted placement (a river threading through the middle
of a map, a tree stump sitting alone in a room) - needs its own
placement algorithm, likely similar in shape to `map_builder/prefab.rs`
(attempt a placement, retry a few times) but for a linear or scattered
feature instead of a whole room. Not started; row 4 exists in the
template now so future art doesn't need re-exporting once this gets
designed.

## Prompting the generator for a new theme

Checklist to work through together before generating a new theme's 16
cells - fill this in per theme, keep the answers here or in the
session that produces the art:

1. **Theme name & mood** - one line (e.g. "Forest - overgrown, mossy,
   dappled light").
2. **Palette constraints** - 2-3 dominant colors, any colors to avoid
   (e.g. avoid pure black/near-black per the standard floor-near-black
   gotcha shared with the character sheets).
3. **The 16-item numbered list itself** (see "Numbering convention"
   above for the exact format), thinking through each row's own job as
   it's filled in:
   - 1-4 (floor): basic ground textures, should read as clearly
     walkable, low visual noise (players will stand on these
     constantly). **#1 specifically must be the plainest, most generic
     one of the four** - the whole map starts as 100% this tile before
     any other variety gets layered on (see "Generation algorithm"
     above), so it needs to hold up as an unremarkable, everywhere
     default, not something with a lot of personality.
   - 5-8 (wall): basic impassable boundary textures, should read as
     clearly NOT walkable at a glance. **#5 has the same "plainest,
     most generic" requirement as #1**, for the same reason - it's the
     default every wall starts as.
   - 9-12 (themed floor): more decorative/specific floor variety (a
     path, a clearing, a distinctive ground feature) - still walkable,
     just more visually distinct than 1-4's plain base. For each one,
     also decide Patch or Scatter (see "Per-variant placement style"
     below) BEFORE generating - a discrete fixture (torch, grate, bone
     pile) wants Scatter, a spreadable ground texture (moss, puddle,
     algae) wants Patch. This becomes the theme's `floor_variant_style`
     override once the art is in.
   - 13-16 (special wall): impassable FEATURE tiles meant for
     deliberate placement later, not uniform random scatter (water,
     and whatever else fits the theme - a chasm, dense thicket, etc.).

---

## Generator output gotchas (confirmed 2026-09-08, Forest's real export)

- **The generator bakes visible grid-line borders into the sheet
  itself** - real usable content per cell was ~30x30px, not the full
  32x32, with a 1-3px black divider actually part of the image between
  every cell (not just a preview artifact). Always crop these out
  first (content was reliably `bounds = [(1,31),(34,64),(67,97),
  (100,130)]` for both this and the earlier reference sheet), then
  resize/pad the result up to true 32x32 - skipping this would bake a
  visible black seam grid into every tile boundary on the actual map.
- **"Single centered object" tiles don't repeat well; noise/texture
  tiles do.** Confirmed by actually tiling candidates 4x4 before
  building anything real: Grass and Trees (a repeating canopy pattern)
  read as continuous and natural when placed edge-to-edge; Big Rock (a
  single boulder centered on a plain background) produces an obvious
  clone-stamp grid instead. Matters most for rows 1-2 (the basic
  pools that get repeated in bulk) - a "hero object" tile is fine for
  rows 3-4 (themed floor / deliberately-placed special features),
  which are never meant to tile in bulk anyway.
- **Watch for near-duplicate tiles across rows meant to read as
  distinct.** Forest's row-2 "thicket" and row-4 "briar patch" came
  back nearly indistinguishable (same dark mottled texture) despite
  different prompts - not broken, but undermines row 4's whole point
  (a special feature should look distinct from ordinary wall filler).
  Worth a tighter, more visually-differentiated prompt next time
  rather than two similar "dense undergrowth" descriptions.

## Themes done so far

**Forest** (finalized 2026-09-08, `Theme_Forest_overgrown_mossy_
woodland_floor_d.png`, owns rows 0-3 of `map_tiles.png`):
1. Grass
2. Moss
3. Dirt
4. Fallen leaves
5. Trees
6. Big rock / boulder
7. Rubble / small rocks
8. Thicket
9. Dirt path
10. Path fork
11. Root-covered ground
12. Flower-dotted grass
13. Water
14. Tree stump
15. Fallen log
16. Briar patch (see gotcha above - reads very similar to #8)

**Dungeon** (finalized 2026-09-08, `Theme_Dungeon_cold_stone_corridors_
dim_torchli.png`, owns rows 4-7):
1. Plain stone flagstones
2. Cracked stone floor
3. Dusty/grimy stone floor
4. Damp, mossy stone floor
5. Plain brick wall
6. Cracked/crumbling brick wall
7. Damp mossy brick wall
8. Rubble-strewn wall section
9. Stone floor with a drain grate
10. Stone floor with scattered bones/debris
11. Stone floor with a cracked pillar base
12. Stone floor with torchlight glow/soot marks
13. Water/sewage puddle
14. Rubble pile
15. Broken, toppled pillar
16. Cracked iron grate/portcullis chunk

**Sewer** (finalized 2026-09-08, `Theme_Sewer_wet_stone_tunnels_grime_
rust_sta.png`, owns rows 9-12 - NOT 8-11, see the forbidden-row gotcha):
1. Wet stone floor
2. Grimy, slime-streaked stone floor
3. Rusted metal grating floor
4. Algae-stained stone floor
5. Plain brick tunnel wall
6. Slime-covered brick wall
7. Rusted pipe-lined wall section
8. Crumbling, cracked sewer wall
9. Floor with a drainage grate
10. Floor scattered with rubble/debris
11. Floor with a shallow puddle
12. Floor with a glowing fungus/algae patch
13. Standing sewage water
14. Large rusted pipe/valve obstacle
15. Collapsed grate/debris pile
16. Toxic sludge pool

Both came back clean on the first try - every cell distinct, no
near-duplicates like Forest's #8/#16, and the two "distinct pattern"
wall candidates that got tileability-checked (Dungeon's rubble wall,
Sewer's pipe-lined wall) both repeated well. Verified Sewer live -
real screenshot of the title-screen background rendering the new
tiles correctly (pipe walls, glowing algae patches, mossy brick) with
no defects. Dungeon uses the identical `tile_row`/rendering path already
proven for Forest and Sewer, so it wasn't separately screenshot-verified
this round (the session's input-automation driver got unreliable, not
a sign of an actual code issue) - worth a quick visual sanity check next
session if it hasn't come up in normal play by then.

**Swamp** (proposed 2026-09-13, NOT yet generated - the user asked for a
prompt list to run through the generator themselves; drafted here rather
than only in chat so it isn't lost). Mood: murky, waterlogged marshland -
decaying vegetation, thick humid air. Palette: muddy brown, dark olive
green, murky teal-green - avoid pure black/near-black (the standard
floor-near-black gotcha) and overly saturated/clean blues (should read
swampy, not tropical). Would own rows 13-16 (the next free 4-row block
after Sewer's 9-12) once generated:
1. Wet mud
2. Marsh grass tufts
3. Damp peat / dark soil
4. Shallow puddled ground
5. Tangled mangrove roots
6. Thick reed/cattail wall
7. Moss-covered rotted log wall
8. Twisted vine tangle wall
9. Lily-pad covered patch - Patch
10. Cracked dry-mud patch - Patch
11. Fallen dead tree / driftwood - Scatter
12. Glowing marsh-gas / firefly patch - Scatter
13. Murky swamp water (the liquid/moat cell - would map to `water_
    variants`/`fortress_moat_variant` the same way Forest's Water does)
14. Sunken, rotted stump
15. Half-submerged log
16. Thick reed/cattail cluster
