# UI Panel Sheet Guide

Master reference for `resources/ui_panels.png` - the real PixelLab-generated
9-slice box-border art (item 10 in `docs/ideas.md`), the map-tile-theme
counterpart to `Dungeon_Font_Glyph_to_Cell_Map.md` (characters/enemies/
icons) and `Map_Tile_Theme_Guide.md` (floor/wall tiles). Read this first
before touching `render_helpers::draw_pixel_box`/`UiPanelTheme` or
generating a new panel material.

**Status as of 2026-09-14: all four materials generated and composited,
one call site wired up (Item Menu's Items box) as a first real
integration** - not yet screenshot-verified live (this project's own
standing rule: bracket-lib pixel positioning can't be verified without a
real render). The other three Item Menu boxes, the three dungeon-HUD bars,
the Pause Hints box, the battle log, and the Battle Actions box (see
`docs/ideas.md` item 10 for the full 13-site list) are still on
`draw_ascii_box` - swap them one at a time once this first one is
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
comment in `main.rs`). `tint` recolors the source art the same way
`UNOWNED_ICON_TINT` already does elsewhere (the console shader multiplies
texture color by whatever `ColorPair` is passed) - pass the box's existing
`draw_ascii_box` color to keep the same red/blue/green/white/yellow
coding, or WHITE for the art's own true colors.

Two real, deliberate simplifications versus a pixel-perfect box (see
`draw_pixel_box`'s own doc comment for the full reasoning):

- The box's top-left corner lands at the exact right pixel (via
  `set_fancy`'s fractional positioning), but its overall size is rounded
  to the nearest whole 32px tile count - up to ~16px larger/smaller than
  the original ASCII box's exact footprint. Confirmed via the Items box's
  own real numbers (`LEFT_X=3, TOP_Y=12, LEFT_WIDTH=50, TOP_HEIGHT=11` on
  the 107x67 HUD_CONSOLE grid): the pixel-art version renders at 19x4
  tiles (608x128px) versus the ASCII version's exact 598x131px - a few
  pixels off on each axis, not pixel-identical.
- No filled interior yet (see "The center tile exists... " above).

## Still open

- Screenshot verification of the Items box (the one live call site) -
  needed before wiring up the other 12 sites.
- The 3 other Item Menu boxes, the 3 HUD bars, Pause Hints, the battle
  log, and the Battle Actions box (its border color already switches
  live between yellow/green - `draw_pixel_box`'s `tint` parameter already
  supports this the same way `draw_ascii_box` did, just needs wiring).
- A filled interior (see above) - needs the `UI_PANEL_CONSOLE` reordering
  discussed above.
- Per-tile fractional stretching for genuinely pixel-perfect sizing
  (see `systems/hud.rs`'s "how do you scale these" design discussion in
  `docs/journal.md`'s 2026-09-14 entry) - not needed yet since the
  whole-tile rounding is visually negligible at this project's box sizes,
  but the plan if a smaller/tighter box ever needs it.
