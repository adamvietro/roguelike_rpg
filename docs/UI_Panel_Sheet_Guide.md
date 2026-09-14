# UI Panel Sheet Guide

Master reference for `resources/ui_panels.png` - the real PixelLab-generated
9-slice box-border art (item 10 in `docs/ideas.md`), the map-tile-theme
counterpart to `Dungeon_Font_Glyph_to_Cell_Map.md` (characters/enemies/
icons) and `Map_Tile_Theme_Guide.md` (floor/wall tiles). Read this first
before touching `render_helpers::draw_pixel_box`/`UiPanelTheme` or
generating a new panel material.

**Status as of 2026-09-14: all four materials generated and composited,
the Item Menu's 6 boxes refined across nine rounds of live screenshot
feedback, the 3 dungeon-HUD bars wired up too (not yet screenshot-
verified - see "Using it" below)** - a saturated tint crushing the
stone's own shading, the border swallowing box text at native scale, a
real no_bg-console fill bug, the fill not quite nesting inside the
border, a title-on-border attempt that hid every title outright, a blank
description panel when nothing's selected, a real detour where a
`has_title` flag briefly excluded the title row from the fill (on the
wrong theory that titles having a black background was itself the bug)
before getting reverted on direct correction - the black fill reaching
up to meet the title row was the intended look the whole time, and
excluding it just left the title illegible over a light background
instead - and, most recently, a same-console text/fill overwrite bug
where printed text let the live dungeon view show through around each
letter, fixed by moving all Item Menu text onto its own later-registered
console (`PANEL_TEXT_CONSOLE`, see "Using it" below). `draw_filled_pixel_box`
fills every box's full nominal area unconditionally now, no exceptions.
The description panel also gained its own title, the one box that never
had one. The remaining 4 of 13 sites (the shop-item tooltip, the Pause
Hints box, the battle log, and the Battle Actions box - see
`docs/ideas.md` item 10 for the full list) are still on `draw_ascii_box`.

## The 4-theme, 3x3 layout

`ui_panels.png` is 3 columns x 12 rows of 32x32px cells (96x384px total,
native resolution, no upscale). Each of the game's 4 map themes owns a
3-row band, in the same order `map_builder::dungeon_theme_pool()` already
uses:

| Rows | Theme | Material |
| --- | --- | --- |
| 0-2 | Dungeon | Carved stone, diamond-medallion corners |
| 3-5 | Forest | Weathered wood, carved sunburst corners |
| 6-8 | Sewer | Rusted iron + stone, riveted corners (v3 - see "Sewer needed two redo rounds" below) |
| 9-11 | Swamp | Waterlogged wood + moss, mossy corner growth |

Within a theme's own 3-row band, the 3x3 nine-slice grid is laid out the
standard way:

| | Col 0 | Col 1 | Col 2 |
| --- | --- | --- | --- |
| **Row 0** | top-left corner | top edge | top-right corner |
| **Row 1** | left edge | center (unused - see below) | right edge |
| **Row 2** | bottom-left corner | bottom edge | bottom-right corner |

`UiPanelTheme::glyph(local_col, local_row)` (in `render_helpers.rs`)
converts a theme + local (col, row) into the real CP437-style glyph index
bracket-lib needs (`glyph = (theme.base_row()*3 + local_row)*3 +
local_col`) - the same row-major convention every custom font in this
project already uses.

**The center tile exists in the sheet but `draw_pixel_box` doesn't draw it
yet** - v1 only draws the hollow border (4 corners + tiled edges), same
shape `draw_ascii_box` already has. Filling the interior with the real
center texture needs `UI_PANEL_CONSOLE` registered BEFORE whatever console
draws a box's own text/icons (so the fill paints underneath, not over) -
a bigger mechanical change (shifts every console index after the
insertion point) deliberately deferred until the hollow-border version is
confirmed working live. See `UI_PANEL_CONSOLE`'s own doc comment in
`main.rs`.

## Generation recipe

All four generated via PixelLab's `create-image-pixflux`
(`https://api.pixellab.ai/v2/create-image-pixflux`), 96x96px, with:

- `color_image` - a crop of that theme's own rows in
  `resources/map_tiles.png` (Dungeon: rows 4-7, Forest: 0-3, Sewer: 9-12,
  Swamp: 13-16 - see `map_builder/themes.rs`'s `tile_row()`), so the panel
  matches the existing tile art's real palette rather than guessing one.
- `outline: "single color black outline"`, matching this project's
  existing pixel-art convention.
- `shading`/`detail`: `"medium shading"`/`"medium detail"` for Dungeon/
  Forest/Swamp; Sewer needed `"detailed shading"`/`"highly detailed"` -
  see below for why.
- Prompt describes the image explicitly as a 3x3 grid of nine-slice
  pieces (corners/edges/center), each edge/center cell instructed to
  "tile seamlessly" in its own direction - verified for real afterward
  (see "Verification" below), not just asserted in the prompt.

**Important schema correction (2026-09-14): there is no dedicated
`/generate-ui-v2` or `/create-ui-asset` endpoint** - `docs/ideas.md`'s
earlier notes describing those were wrong (confirmed by pulling the
live `openapi.json`'s full endpoint list directly; neither exists).
There's also no `forced_palette` parameter on `create-image-pixflux` -
the real field is `color_image` (a reference image the palette gets
sampled from), not a literal hex list. Always fetch and read the raw
OpenAPI schema JSON directly for a parameter's real shape before
guessing at one - a summarized/fetched answer got both of these wrong
before the raw JSON was pulled and parsed directly.

### Sewer needed two redo rounds

The first Sewer attempt (a generic "rusted metal and damp brick" prompt,
medium shading) came back too murky/low-contrast to read as rusted metal
at all - technically clean (passed every pixel-health check, tiled with
no seams) but a real quality miss. Fixed in two steps, not one:

1. Sharpened the frame prompt (explicit "HIGH CONTRAST", "bright
   highlights", "clear crosshatch mesh pattern") and bumped shading/detail
   up a level - fixed the frame completely, but the CENTER cell drifted
   into a busy diagonal cracked-stone pattern that read as a distracting
   chevron/herringbone once tiled - technically seamless, but visually
   loud in a way none of the other three themes' calm centers are.
2. Re-prompted with the successful frame description unchanged, but an
   explicit, heavily emphasized instruction for the center specifically
   ("FLAT, PLAIN, nearly featureless... no diagonal patterns... calm and
   subdued") - fixed it, landed on a calm cobblestone texture.

**Worth remembering for the next material**: a prompt revision aimed at
one specific cell (here, boosting the frame's contrast) can regress a
DIFFERENT cell that wasn't the target at all (the center) - the same
lesson `Map_Tile_Theme_Guide.md`'s own Swamp write-up already learned
for tile atlases. Check every cell again after any revision, not just
the one being fixed.

## Verification done so far (per-material, before compositing)

Every one of the 4 final source images passed, checked with a real script
(not eyeballed):

- **Watermark scan**: all 4 corners, upscaled 8x, visually clean.
- **Near-black opaque pixel floor**: 0 pixels below the 30/channel safe
  floor CLAUDE.md's own PixelLab gotcha requires (every source image
  landed exactly at or above it without needing a manual floor pass -
  unusual, but confirmed by direct pixel scan, not assumed).
- **Transparency**: fully opaque, 0 stray transparent pixels - these are
  meant to render as solid filled panels, not sprites with a transparent
  background.
- **Tiling seam test**: each theme's 9 pieces sliced out and re-tiled
  into a 256x160px box (8x wider, 5x taller than the 96x96 source) -
  confirmed no visible seam on any edge or the center, at a size well
  beyond what the source alone could prove.

The final composited `resources/ui_panels.png` got the same near-black/
transparency scan run again across the whole 96x384 sheet after
compositing, confirming the paste operations didn't introduce anything
new.

## Using it: `render_helpers::draw_pixel_box`

```rust
draw_pixel_box(batch, x, y, width, height, UiPanelTheme::Dungeon, tint);
```

Same call shape as `draw_ascii_box` - `x`/`y`/`width`/`height` are in the
SAME HUD_CONSOLE cell units an existing `draw_ascii_box` call already
used, so swapping one call site is close to a one-line change. `batch`
must target `UI_PANEL_CONSOLE` specifically (a different console than
whatever draws the box's own text, since `ui_panels.png` is a different
font at a different native cell size - see `UI_PANEL_CONSOLE`'s own doc
comment in `main.rs`).

**Pass WHITE for `tint`, not the box's old `draw_ascii_box` color** -
confirmed live 2026-09-14 that a strong saturated tint (BLUE, in the
Items box's case) crushes this shaded/textured stone material into a
flat, unnatural-looking color wash. The console shader's multiply-tint
trick (same one `UNOWNED_ICON_TINT` uses elsewhere) works fine on this
project's flat, already-colorful ability icons, but not on subtle
grey-shaded stone/wood/metal art - the whole point of generating real
textured material is lost if a tint flattens it back into a solid color.
The box's own title text still carries its category color, so switching
to WHITE loses no actual information about which box is which.

**Every tile draws at `PIXEL_BOX_TILE_SCALE` (0.375x, ~12px), not the
source art's native 32px** - also confirmed live: at full native size
the border was thick enough to swallow an Item Menu box's own list text
entirely. `set_fancy` scales a glyph around its own center (confirmed
against bracket-terminal's real vertex-shader source), so `draw_pixel_box`
also closes up the spacing between tile centers by that same factor -
scaling the glyph alone without doing this would open a visible gap
between adjacent tiles.

**Use `render_helpers::draw_filled_pixel_box` (border + fill together),
not `draw_pixel_box` alone** - every real call site needs a deliberate
solid interior fill or whatever's on the console(s) underneath (for the
Item Menu, the frozen paused dungeon view) bleeds through instead of a
clean background.

**Any text printed inside a filled box needs its OWN console, registered
LATER in z-order than the fill's console - never the same console/batch,
even "after" the fill in draw order** - a real, confirmed bug (2026-09-14):
`SimpleConsole::set` (bracket-terminal's real source) REPLACES a cell's
entire `(glyph, fg, bg)` tuple outright, it doesn't layer new content onto
whatever was drawn there before. Printing text on the same console as the
fill, even "after" the fill in the same tick, OVERWRITES that cell's fill
entirely rather than painting over it - the cell's glyph becomes the
letter's own shape, and a `no_bg` console's shader discards any pixel
whose SOURCE TEXTURE is near-black, which is exactly what the "empty"
space inside a glyph's own cell looks like. The result: solid black
everywhere the fill alone covers a cell, but the live view bleeding
through around and between individual letters, since the fill that used
to occupy that exact cell no longer exists once text replaced it. Fixed
by adding `PANEL_TEXT_CONSOLE` (index 47, `no_bg`, same `HUD_COLS x
HUD_ROWS` grid as `HUD_CONSOLE`), registered after `HUD_CONSOLE` in
`main.rs`'s builder chain, and moving every `print_color`/
`print_color_centered` call that lands on top of a fill (the Item Menu's
titles, list entries, stats, and description text) onto a `text_batch`
targeting it instead of the fill's own `batch`. Text that never sits on
a fill (the Item Menu's footer, below every box) can stay on the plain
`HUD_CONSOLE` batch - this only matters for text drawn over a filled
cell.

**The fill itself has to go through the FOREGROUND channel, not the
background one** - a real, confirmed bug (2026-09-14), not a rect-math
mistake: `HUD_CONSOLE` is a `with_simple_console_no_bg` console, and its
actual fragment shader (`CONSOLE_NO_BG_FS` in bracket-terminal's real
GLSL source) takes a background color as an input but never reads it
anywhere - every fragment is either the glyph's own opaque texture or a
hard `discard`, with no "solid background" case at all. A `fill_region`
call with a space glyph and `bg=BLACK` is a silent no-op on this console
type - `draw_filled_pixel_box` instead fills with a full-block glyph
(CP437 219, `'█'`) tinted BLACK via `fg`, the same multiply-tint trick
every other tinted icon in this project already relies on, just applied
to a solid block instead of a sprite.

Two real, deliberate simplifications versus a pixel-perfect box remain
even at the smaller scale (see `draw_pixel_box`'s own doc comment for the
full reasoning):

- The box's top-left corner lands at the exact right pixel (via
  `set_fancy`'s fractional positioning), but its overall size is rounded
  to the nearest whole tile count at the smaller scale - within a few
  pixels of the original ASCII box's exact footprint, not pixel-identical.
- No real textured filled interior yet - the black fill is a flat color
  stand-in, not the sheet's own center tile art (see "The center tile
  exists..." above for why that needs a bigger, deferred change).

**Box titles print at the box's own nominal top row (`y`), NOT overlapping
the border** - a `y + 1` nudge was tried 2026-09-14 to land the title
literally "on" the border's own top edge, and turned out to be a real
architectural dead end rather than a pixel-tuning miss: the border draws
on `UI_PANEL_CONSOLE`, registered (and therefore z-ordered) AFTER
`HUD_CONSOLE`, so wherever a title's row coincides with the border's own
footprint, the border's fully opaque tile paints directly over the title
and erases it completely - confirmed live (every title on the screen
vanished, not just shifted). Reverted to row `y`, confirmed visible in
every earlier screenshot. Getting a title to read as genuinely embedded
in the border art would need a real new console layered even later than
`UI_PANEL_CONSOLE` - not attempted here; readability won out over the
border-embedded look for now.

## Still open

- A screenshot of the 3 dungeon-HUD bars, now wired up - unverified since
  they render over the LIVE dungeon view rather than a paused menu, a
  real context difference from the Item Menu worth confirming looks
  right (does a solid black bar background read well over live gameplay,
  or does it want to stay closer to see-through there specifically).
- `systems/hud.rs`'s 3 bars have the SAME same-console text/fill bug the
  Item Menu just got fixed for - they use `label_batch` (targeting
  `HUD_CONSOLE`) for both the fill AND the Ability Bar's own number
  labels. Not yet fixed; should get the identical `PANEL_TEXT_CONSOLE`
  treatment once the Item Menu fix itself is confirmed live.
- The remaining 4 of 13 sites: the shop-item tooltip, Pause Hints, the
  battle log, and the in-combat Battle Actions box (its border color
  already switches live between yellow/green - moot now that
  `draw_pixel_box` calls use WHITE regardless of category color, so this
  just needs wiring, not any special-casing for the color switch).
- A real textured filled interior (see above) - needs the
  `UI_PANEL_CONSOLE` reordering discussed above.
- Per-tile fractional stretching for genuinely pixel-perfect sizing
  (see `systems/hud.rs`'s "how do you scale these" design discussion in
  `docs/journal.md`'s 2026-09-14 entry) - not needed yet since the
  whole-tile rounding is visually negligible at this project's box sizes,
  but the plan if a smaller/tighter box ever needs it.
- `PIXEL_BOX_TILE_SCALE` (0.375) is a first-pass guess matched to
  HUD_CONSOLE's own real cell width, not yet confirmed against a real
  screenshot - the first number to retune if the border still reads too
  thick/thin once seen live.
