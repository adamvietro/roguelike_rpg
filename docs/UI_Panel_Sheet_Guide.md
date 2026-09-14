# UI Panel Sheet Guide

Master reference for `resources/ui_panels.png` - the real PixelLab-generated
9-slice box-border art (item 10 in `docs/ideas.md`), the map-tile-theme
counterpart to `Dungeon_Font_Glyph_to_Cell_Map.md` (characters/enemies/
icons) and `Map_Tile_Theme_Guide.md` (floor/wall tiles). Read this first
before touching `render_helpers::draw_pixel_box`/`UiPanelTheme` or
generating a new panel material.

**Status as of 2026-09-14: all four materials generated and composited,
every border on the dungeon-exploration screen AND the Paused screen's
Hints box now converted (12 of 13 real sites); the Hints box confirmed
"Perfect" live, the dungeon HUD bars and shop tooltip still getting
their geometry corrected across rounds - see "Using it" below** -
across fourteen-plus rounds of live feedback, most recently: the
Ability Bar's icons reading as clipped/overlapped by its own border
(traced to a top-clearance formula that was numerically identical to
the unlabeled bars' despite needing to also clear the number label -
fixed), noticeably excess dead space below the bar icons (the bottom
edge's extra clearance row, sized for a much thicker pre-compact-scale
border, trimmed off), and the shop tooltip's text sitting too close to
its top edge (moved down into a taller box). Earlier rounds: a
saturated tint crushing the stone's own shading, the border swallowing
box text at native scale, a real no_bg-console fill bug, the fill not
quite nesting inside the border, a title-on-border attempt that hid
every title outright, a blank description panel when nothing's
selected, a real detour where a `has_title` flag briefly excluded the
title row from the fill (reverted on direct correction), a
same-console text/fill overwrite bug fixed with `PANEL_TEXT_CONSOLE`, a
fill/border sub-pixel alignment fix, a border-scale round-trip (0.375
-> 0.5 -> 0.3) that turned out to be masking a deeper issue, the
pixel-perfect fill's own regression hiding every bar icon (fixed with
`ABILITY_BAR_ICON_CONSOLE`/`ABILITY_BAR_ICON_BADGE_CONSOLE`), every
dungeon-screen panel (Item/Ability/Battle Bars, the shop tooltip) plus
the Pause Hints box converging on the Swamp material, and a genuine
second border scale, `PIXEL_BOX_TILE_SCALE_COMPACT`, for boxes small in
either dimension (see "Using it" below for the last few). `draw_filled_
pixel_box` fills every box's full nominal area unconditionally, no
exceptions; the Item Menu alone stays Dungeon/stone. The remaining 2 of
13 sites (the battle log and the in-combat Battle Actions box - see
`docs/ideas.md` item 10 for the full list) are battle-only and still on
`draw_ascii_box`.

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
shape `draw_ascii_box` already has; the solid interior is a flat black
`draw_panel_fill` stand-in (see "Using it" below), not this real
texture. The console-reordering concern this used to note (needing
`UI_PANEL_CONSOLE` registered BEFORE whatever draws a box's own text) no
longer applies - `PANEL_TEXT_CONSOLE` is now registered LAST in the
whole builder chain specifically so text always paints over both the
fill and the border regardless of where either lives, so a real
textured center tile could draw on `UI_PANEL_CONSOLE` (alongside the
fill and border it already shares that console with, same
`FlexiConsole`-stacking reasoning) without any reordering at all. Still
deferred, just no longer blocked on that.

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

**Every tile draws at a `scale` parameter (`PIXEL_BOX_TILE_SCALE` 0.3x
for large boxes, `PIXEL_BOX_TILE_SCALE_COMPACT` 0.15x for small ones -
see "Two scales, not one" below), not the source art's native 32px** -
at full native size the border was thick enough to swallow an Item
Menu box's own list text entirely. `set_fancy` scales a glyph around
its own center (confirmed against bracket-terminal's real vertex-shader
source), so `draw_pixel_box` also closes up the spacing between tile
centers by that same factor - scaling the glyph alone without doing
this would open a visible gap between adjacent tiles. Icon/portrait
rendering elsewhere doesn't share either constant, so retuning them
only affects the border/fill, never icon size.

**Two scales, not one: `PIXEL_BOX_TILE_SCALE` (0.3x, large boxes) and
`PIXEL_BOX_TILE_SCALE_COMPACT` (0.15x, boxes small in EITHER
dimension)** - a real architecture change (2026-09-14), not another
retune of the same number. The single-constant approach round-tripped
twice on live feedback (0.375x first pass -> 0.5x when the Item Menu's
own large boxes read too thin -> 0.3x on the OPPOSITE complaint once
the dungeon HUD bars were seen at 0.5x) and STILL wasn't right - the
dungeon HUD bars and shop tooltip still read wrong even at 0.3x.
Diagnosed with real numbers instead of guessing a fourth value: a
border tile's PIXEL size is fixed by whatever scale is passed, so a box
small in some dimension has the border eating a much BIGGER FRACTION
of that dimension than a large box does, regardless of scale. Computed
directly from this project's own geometry (`ability_bar_box_bounds`,
`SHOP_TOOLTIP_WIDTH`/`HEIGHT`) at 0.3x: a single-icon Ability Bar box's
two border COLUMNS alone were ~29% of its total width, and the shop
tooltip's two border ROWS were ~50% of its total height - both far
worse than the Item Menu's boxes or the Paused screen's Hints box
(confirmed "Perfect" at 0.3x) ever hit, since neither of those is small
in any dimension. `draw_filled_pixel_box_scaled` (new, takes an
explicit `scale`) is the real entry point now; `draw_filled_pixel_box`
is a thin wrapper defaulting to `PIXEL_BOX_TILE_SCALE`, unchanged for
the Item Menu and Hints. The 3 dungeon HUD bars and the shop tooltip -
the sites that actually have this problem - now call `_scaled` with
`PIXEL_BOX_TILE_SCALE_COMPACT` directly.

**Use `render_helpers::draw_filled_pixel_box` (border + fill together),
not `draw_pixel_box` alone** - every real call site needs a deliberate
solid interior fill or whatever's on the console(s) underneath (for the
Item Menu, the frozen paused dungeon view; for the dungeon HUD bars, the
live view) bleeds through instead of a clean background. Its signature
takes just one `panel_batch` (targeting `UI_PANEL_CONSOLE`) - fill and
border both live there now (see next point).

**The fill lives on `UI_PANEL_CONSOLE` itself, drawn as ONE stretched
`set_fancy` quad positioned/sized from the exact same numbers the border
uses** - this is the real fix (2026-09-14) for a fill/border alignment
bug: the fill used to be computed independently (`pixel_box_hud_rect`,
now removed) by rounding the border's real pixel footprint to
`HUD_CONSOLE`'s own coarse ~12-16px cell grid - a different console and
resolution than the border's own sub-pixel-precise `set_fancy`
positions, so the two could drift a few pixels apart, visibly spilling
past or falling short of the border especially once the border itself
is only ~16px thick. Confirmed safe to put fill and border tiles on the
SAME console by tracing bracket-terminal's real source: `with_fancy_
console` consoles (`UI_PANEL_CONSOLE`) are backed by `FlexiConsole`, a
SPARSE console whose `set_fancy` PUSHES a new tile onto a `Vec` -
unlike `HUD_CONSOLE`'s `SimpleConsole`, which stores one fixed `Tile`
per cell and overwrites it on every `set` call, `FlexiConsole` never
erases anything already queued, so overlapping `set_fancy` calls just
layer in draw order instead. The GLSL vertex shader was also checked
directly to confirm `set_fancy`'s `scale: PointF` applies independently
per axis (`base_pos *= aScale`, a component-wise `vec2` multiply) - so
one glyph CAN be stretched into an arbitrary rectangle, not just resized
uniformly, which is what makes covering the whole interior in a single
draw call possible. See `render_helpers::draw_panel_fill`'s own doc
comment for the exact position/scale math.

**Any text printed inside a filled box still needs its OWN console,
registered LATER in z-order than the fill's console - never the same
console/batch as the fill, even "after" it in draw order** - this is
still real and still applies, even though the fill itself moved off
`HUD_CONSOLE`: `HUD_CONSOLE` (and `PANEL_TEXT_CONSOLE` itself) remain
plain `SimpleConsole`s, where `set` REPLACES a cell's entire
`(glyph, fg, bg)` tuple outright rather than layering onto whatever was
there before. Fixed by adding `PANEL_TEXT_CONSOLE` (index 47, `no_bg`,
same `HUD_COLS x HUD_ROWS` grid as `HUD_CONSOLE`), registered LAST in
`main.rs`'s builder chain (after even `UI_PANEL_CONSOLE`), and moving
every `print_color`/`print_color_centered` call that lands on top of a
fill (Item Menu titles/list entries/stats/description text, the Ability
Bar's own number-key labels) onto a `text_batch` targeting it. Text
that never sits on a fill (the Item Menu's footer, below every box) can
stay on the plain `HUD_CONSOLE` batch.

**The fill's own color goes through the FOREGROUND channel, not the
background one** - `draw_panel_fill` fills with a full-block glyph
(CP437 219, `'█'`) tinted BLACK via `fg`, the same multiply-tint trick
every other tinted icon in this project already relies on
(`texture_white * BLACK = black`). Originally this was a hard
requirement forced by a `no_bg`-console bug (`HUD_CONSOLE`'s fragment
shader never reads `bg` at all - see `CLAUDE.md`'s own standing gotcha);
now that the fill lives on `UI_PANEL_CONSOLE`'s fancy shader
(`SPRITE_CONSOLE_FS` - `FragColor = original * ourColor`, no discard at
all), the same `fg`-tint approach still works for the same underlying
reason, just without the `no_bg` console's discard behavior to work
around.

**A box that has ICONS inside it (not just text) needs those icons on a
console registered AFTER `UI_PANEL_CONSOLE` too, same requirement as
text** - a real regression (2026-09-14), the direct cost of the fill's
own no-discard fancy shader: once the fill became a fully opaque
`set_fancy` quad with no discard case at all, it started painting
directly over anything drawn on a console registered BEFORE
`UI_PANEL_CONSOLE` (46) - specifically the dungeon HUD bars' own icon
portraits (`ABILITY_BAR_CONSOLE`, 24) and stack-count badges
(`ABILITY_BAR_BADGE_CONSOLE`, 26), both registered well before it.
Confirmed live: every bar rendered as a solid black box, no icons
visible at all. Fixed with `ABILITY_BAR_ICON_CONSOLE` (48) and
`ABILITY_BAR_ICON_BADGE_CONSOLE` (49) - plain duplicates of those two
consoles' exact grid/font config, registered at the very end (after
`PANEL_TEXT_CONSOLE`) instead of inserted at their original position -
inserting there would have meant renumbering every constant between
the old and new position, a much bigger and riskier mechanical change
than appending two new ones. The OLD `ABILITY_BAR_CONSOLE`/
`ABILITY_BAR_BADGE_CONSOLE` stay registered (and in the `cls()` sweep)
purely because a mouse-position translation (`ctx.set_active_console`,
`AbilityBarMousePos`) is keyed to their grid dimensions - identical
between old and new, so that math needed no change, only the actual
drawing moved.

One real, deliberate simplification versus a pixel-perfect box remains
(see `draw_pixel_box`'s own doc comment for the full reasoning): no real
textured filled interior yet - the black fill is a flat color stand-in,
not the sheet's own center tile art (see "The center tile exists..."
above). The box's own top-left corner and overall footprint both now
land pixel-exact (fill and border share identical math), so nothing
about SIZE is approximated anymore, only the interior's TEXTURE.

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

- A fresh screenshot confirming this round's geometry fixes together:
  the Ability Bar's own top-clearance bump (`label_row - 1` ->
  `label_row - 2`, meant to stop the border from reading as clipping
  into the icons), the trimmed bottom clearance on all 3 bars (`+ 1`
  dropped), and the shop tooltip's text moved down into a taller box
  (`SHOP_TOOLTIP_HEIGHT` 3 -> 4, text row `box_y + 1` -> `box_y + 2`).
  None of this confirmed live yet - reasoned through by comparing the
  Ability Bar's formula against the (working) Item/Battle Bar formula
  and finding it numerically identical despite needing extra clearance
  for the number label, not by tracing the exact rendering-level cause
  of the reported overlap.
- The remaining 2 of 13 sites: the battle log and the in-combat Battle
  Actions box (its border color already switches live between yellow/
  green - moot now that `draw_pixel_box` calls use WHITE regardless of
  category color, so this just needs wiring, not any special-casing for
  the color switch) - both battle-only, the only screen with any
  `draw_ascii_box` left in the whole project. Given how narrow/short
  some of THEIR boxes might be too (the Battle Actions box in
  particular), check whether they need `PIXEL_BOX_TILE_SCALE_COMPACT`
  from the start rather than the default, once converted.
- A real textured filled interior (see "The center tile exists..."
  above) - no longer blocked on console reordering, just not built yet.
- Per-tile fractional stretching for genuinely pixel-perfect SIZING to
  arbitrary pixel dimensions (see `systems/hud.rs`'s "how do you scale
  these" design discussion in `docs/journal.md`'s 2026-09-14 entry) -
  distinct from the fill/border ALIGNMENT fix above (which is already
  pixel-perfect relative to each other); this is about the box's overall
  width/height still rounding to a whole tile count. Not needed yet
  since that rounding is visually negligible at this project's box
  sizes, but the plan if a smaller/tighter box ever needs it.
- `PIXEL_BOX_TILE_SCALE_COMPACT` (0.15) is a first-pass value computed
  from real geometry (see "Two scales, not one" above for the exact
  numbers), not yet confirmed against a screenshot - the next number to
  retune if the bars/tooltip still read wrong.
