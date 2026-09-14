# UI Panel Sheet Guide

Master reference for `resources/ui_panels.png` - the real PixelLab-generated
9-slice box-border art (item 10 in `docs/ideas.md`), the map-tile-theme
counterpart to `Dungeon_Font_Glyph_to_Cell_Map.md` (characters/enemies/
icons) and `Map_Tile_Theme_Guide.md` (floor/wall tiles). Read this first
before touching `render_helpers::draw_pixel_box`/`UiPanelTheme` or
generating a new panel material.

**Status as of 2026-09-14: all four materials generated and composited,
all 6 Item Menu boxes wired up, four real rounds of live screenshot
feedback fixed so far** - a saturated tint crushing the stone's own
shading (fixed: tint WHITE, not a category color), the border rendering
at native 32px and swallowing box text (fixed: `PIXEL_BOX_TILE_SCALE`), a
real no_bg-console fill bug that left the frozen dungeon view bleeding
through box interiors (fixed: fill via the foreground channel, not
background), title placement and box spacing (title nudged onto the
border's own row, `BOX_GAP`/`SUB_GAP` widened), and the fill not quite
nesting inside the border once it rendered solid (fixed: the fill's rect
now derives from the border's own already-rounded footprint instead of
being computed independently - see "Using it" below for all of these).
Still pending a FIFTH screenshot to confirm the fill/border alignment fix
actually landed right. The remaining 7 of 13 sites (the three
dungeon-HUD bars, the Pause Hints box, the battle log, and the Battle
Actions box - see `docs/ideas.md` item 10 for the full list) are still on
`draw_ascii_box` - swap them one at a time once the Item Menu is
confirmed correct.

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
clean background. `draw_filled_pixel_box` draws the fill on the SAME
console/batch the box's own text uses, before that text, so the text
naturally overwrites the fill at its own cells with no extra console/
z-order needed.

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

**Box titles print one row below the box's own nominal top row**
(`y + 1`, not `y`) - the pixel-art border tiles get an extra north-anchor
correction (`WIGGLE_CONSOLE_Y_ANCHOR_OFFSET`, see `draw_panel_tile`) that
plain `print_color`'d text never goes through, so a title printed at the
box's exact nominal row visibly sat above where the border's own top edge
actually rendered instead of "on" it. A one-row nudge, not a scaled
offset - simpler and more directly informed by what the screenshot
actually showed, but still a first-pass guess like everything else here.

## Still open

- A fourth screenshot, confirming the fill-bug fix, the title-row nudge,
  and the widened box spacing (see `screens/item_menu.rs`'s `BOX_GAP`/
  `SUB_GAP`) all actually look right together - needed before wiring up
  the other 12 sites.
- The remaining 7 of 13 sites: the 3 HUD bars, Pause Hints, the battle
  log, and the Battle Actions box (its border color already switches
  live between yellow/green - moot now that `draw_pixel_box` calls use
  WHITE regardless of category color, so this just needs wiring, not any
  special-casing for the color switch).
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
