# UI Panel Sheet Guide

Master reference for `resources/ui_panels.png` - the real PixelLab-generated
9-slice box-border art (item 10 in `docs/ideas.md`), the map-tile-theme
counterpart to `Dungeon_Font_Glyph_to_Cell_Map.md` (characters/enemies/
icons) and `Map_Tile_Theme_Guide.md` (floor/wall tiles). Read this first
before touching `render_helpers::draw_pixel_box`/`UiPanelTheme` or
generating a new panel material.

**Status as of 2026-09-14: ALL 13 of the original real `draw_ascii_box`
sites are now converted - `draw_ascii_box` itself is dead code, removed
entirely.** Five materials now: the original Dungeon/Forest/Sewer/Swamp
(mirroring `map_builder::dungeon_theme_pool()`) plus a new `Battle`
theme (ornate wood + gold star-medallion corners) for the battle
screen's last two sites (the ability-selection box, the battle log).
Also new this round: a genuinely different UI element, a real pixel-art
bar (`render_helpers::draw_pixel_bar`, a separate asset - `resources/
battle_bar_frame.png`, a 3-cell frame not a 9-slice box) replacing the
old ASCII `[####----]` rendering for the PLAYER's own HP/ATB display in
battle (enemies keep the ASCII version, a deliberate scope choice - see
"Using it" below). Confirmed working live: the Item Menu, the Paused
screen's Hints box, and the dungeon HUD bars' border/icon overlap. Still
unconfirmed: the new Battle theme's 2 sites, the new bar, and this
round's per-bar box width. One known, understood-but-not-yet-fixed
regression: the Item Menu's title labels print ON the border instead of
above it, a side effect of the center-shift fix (see "Using it" below)
moving every box's border to its true position, including at the Item
Menu's own default scale.

This file's own "Using it" section below covers the CURRENT architecture
in full; **`docs/journal.md`'s 2026-09-14 entries are the place to look
for the blow-by-blow history of how each round's feedback led here** -
this doc doesn't try to re-narrate that here anymore (it did for a
while and became a changelog rather than a reference).

## The 5-theme, 3x3 layout

`ui_panels.png` is 3 columns x 15 rows of 32x32px cells (96x480px total,
native resolution, no upscale). The first 4 rows-bands mirror the game's
4 map themes, in the same order `map_builder::dungeon_theme_pool()`
already uses; the 5th (`Battle`, added 2026-09-14) is the first theme
NOT tied to a map theme - it's for the battle screen specifically (see
`UiPanelTheme::base_row`'s own doc comment):

| Rows | Theme | Material |
| --- | --- | --- |
| 0-2 | Dungeon | Carved stone, diamond-medallion corners |
| 3-5 | Forest | Weathered wood, carved sunburst corners |
| 6-8 | Sewer | Rusted iron + stone, riveted corners (v3 - see "Sewer needed two redo rounds" below) |
| 9-11 | Swamp | Waterlogged wood + moss, mossy corner growth |
| 12-14 | Battle | Ornate carved honey-amber wood, gold star-flower medallion corners (matched to a direct reference image) |

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

All five generated via PixelLab's `create-image-pixflux`
(`https://api.pixellab.ai/v2/create-image-pixflux`), 96x96px, with:

- `color_image` - a crop of that theme's own rows in
  `resources/map_tiles.png` (Dungeon: rows 4-7, Forest: 0-3, Sewer: 9-12,
  Swamp: 13-16 - see `map_builder/themes.rs`'s `tile_row()`), so the panel
  matches the existing tile art's real palette rather than guessing one.
  **`Battle` is the one exception** - it isn't a map theme, so there's no
  existing `map_tiles.png` band to crop from. Generated from a detailed
  TEXT description of the palette/style (warm honey-amber wood, gold
  star-flower medallions) instead, matched against a reference image the
  user provided directly rather than any `color_image` - no image file
  for that reference ended up locally accessible (a pasted-in image, not
  a saved file), so the match is prompt-driven, not palette-sampled.
  Landed close to the reference on the first generation anyway (compare
  `docs/journal.md`'s 2026-09-14 entry if a second pass is ever needed).
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

Every one of the 5 final source images passed, checked with a real script
(not eyeballed):

- **Watermark scan**: all 4 corners, upscaled 8x, visually clean.
- **Near-black opaque pixel floor**: 0 pixels below the 30/channel safe
  floor CLAUDE.md's own PixelLab gotcha requires for 4 of the 5 (every
  source image landed exactly at or above it without needing a manual
  floor pass - unusual, but confirmed by direct pixel scan, not
  assumed); `Battle` needed a real floor pass - 412 pixels below the
  floor (its outline/shadow areas), floored to 30+/channel before
  compositing.
- **Transparency**: fully opaque, 0 stray transparent pixels - these are
  meant to render as solid filled panels, not sprites with a transparent
  background.
- **Tiling seam test**: each theme's 9 pieces sliced out and re-tiled
  into a 256x160px box (8x wider, 5x taller than the 96x96 source) -
  confirmed no visible seam on any edge or the center, at a size well
  beyond what the source alone could prove.

The final composited `resources/ui_panels.png` got the same near-black/
transparency scan run again across the whole sheet after compositing,
confirming the paste operations didn't introduce anything new.

**`resources/battle_bar_frame.png` (a separate file, not part of
`ui_panels.png`)** got its own verification, adapted for its different
shape (a 3-cell horizontal row, not a 3x3 grid): near-black floor check
(clean, 0 pixels needed flooring), a PIXEL-LEVEL transparency check
across its middle channel specifically (not just a visual glance -
sampled `alpha` at multiple x-positions along the vertical center row,
confirmed genuinely 0 everywhere, since the first generation attempt
LOOKED like a hollow frame in the prompt but actually came back with an
opaque wood-grain fill in the "channel" - see "The bar frame needed a
second, more explicit prompt" below), and the same left+middle+right
retiling stress test the box materials get.

### The bar frame needed a second, more explicit prompt

First attempt asked for a frame with a "carved recessed channel" that's
"fully transparent" - came back as a solid wooden scroll/banner, fully
opaque, no transparency anywhere in the middle. The model read "recessed
channel" as a carved DETAIL to draw, not literal see-through space.
Second attempt dropped that framing entirely and described it as a
literal "empty picture frame... like an empty window frame... the
inside opening is EMPTY SPACE... NOT wood, NOT a plaque, NOT a scroll" -
landed a genuine hollow frame on the first retry. **Worth remembering**:
when asking a PixelLab prompt for a transparent/hollow region, describe
it as an empty frame/window (a shape everyone recognizes as having
nothing in the middle), not as a "carved" or "recessed" detail, which
reads as decorative content rather than absence of content.

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

**`set_fancy` scales a tile around a FIXED CENTER, not around its own
nominal position - `pixel_box_tiles` corrects for this, or a box's real
rendered position silently drifts away from what the caller asked for**
- the real root cause (2026-09-14) behind two full rounds of "still not
right" that scale/padding tuning alone couldn't fix: "the border is
overlapping the icons," "too much space on the right hand side of the
item bar." Solved directly from the real vertex shader (`base_pos =
(aPos - center) * scale + center`, `center = position + 0.5` native
cell - confirmed against bracket-terminal's actual GLSL source): a
tile's TRUE rendered edge is at `position + (1-scale)/2`, not
`position` itself, unless `scale == 1.0`. Every tile in a box shifts by
this SAME amount (a translation, not a resize), so the fill and border
stay perfectly aligned with EACH OTHER (which is why the alignment
itself was already confirmed "the right size") - but the whole box's
real position drifts relative to content on a DIFFERENT, independent
coordinate system (the bar icons, on `ABILITY_BAR_COLS`/`ROWS`), which
has no way to know about this drift at all. The shift is small at the
default scale (~11px at 0.3x, easy to miss against a thicker border)
but large at the compact one (~13.6px at 0.15x) - big enough, relative
to a now-thin border, to visibly crowd an icon on the low-coordinate
edges (left/top - reads as overlap) while leaving obvious extra space
on the high-coordinate edges (right/bottom). `pixel_box_tiles` now
subtracts `(1-scale)/2` back out of `base_col`/`base_row` before any
tile position is derived from them, canceling the shift. Verified with
a throwaway test (zero shift at scale 1.0; the shift matches the
derived formula at compact scale; feeding the correction back through
the real edge formula lands exactly at the intended position) - removed
after confirming, per this project's own testing convention.
**Confidence note**: the X-axis derivation is exact math with no
assumptions; the Y-axis (`base_row`) applies the identical correction,
but `FlexiConsole::set_fancy` flips `position.y` before this transform
runs (the same flip `WIGGLE_CONSOLE_Y_ANCHOR_OFFSET` already corrects
for separately), which makes an independent by-hand Y derivation
genuinely error-prone - the row correction's SIGN was chosen to match
the already-confirmed live symptom (border crowding down into an icon
at the top, excess room at the bottom), not re-derived from the flip in
isolation. If a fresh screenshot shows vertical alignment got WORSE,
flip that one sign first (`- center_shift` -> `+ center_shift` for
`base_row` only) before looking anywhere else.

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

**Box titles print at the box's own nominal top row (`y`) - STILL true
mechanically, but this row no longer reads as "above the border" the
way it used to, at the Item Menu's own default scale** - a real,
not-yet-fixed side effect of the center-shift fix (see "set_fancy
scales a tile around a FIXED CENTER" above): that fix corrects the
border's TRUE rendered position at every scale, including the Item
Menu's default 0.3x, where the correction moves the border by ~11px -
almost exactly one HUD_CONSOLE row. The Item Menu's title row was
originally tuned (2026-09-14, several rounds before the center-shift
fix existed) to sit "just above" the border - but tuned against the
border's OLD, buggy position, which was rendering about one row further
down than it should have. Now that the border renders at its correct
position, it moved up into the exact row the title already occupies -
confirmed live: "the label are ON THE SAME POSITION as the border and
they used to be directly above." The fix (not yet applied): print
titles at `y - 1` instead of `y`. This is safe now in a way an
identical `y + 1` nudge WASN'T when first tried, back before
`PANEL_TEXT_CONSOLE` existed: back then, text and border shared a
console where the border (registered later) could paint over and erase
the text outright (confirmed live - every title vanished, not just
shifted). Text now lives on `PANEL_TEXT_CONSOLE`, registered AFTER
`UI_PANEL_CONSOLE`, so it always renders on top regardless of row
overlap - the old "erase" failure mode can't happen anymore, which is
exactly what makes the `y - 1` move safe now where `y + 1` wasn't then.

**The Ability Bar and Battle Bar are deliberately WIDER than a tight
icon-fit; the Item Bar isn't** - `systems/hud.rs`'s `ability_bar_box_
bounds` takes an `extra_side_pad` parameter (HUD_CONSOLE columns, each
side), 0 for the Item Bar, `ABILITY_BOX_EXTRA_PAD` (3) for the Ability
Bar and Battle Bar - direct request 2026-09-14 ("wider ability boxes
for the abilities and smaller for the items"). This is a deliberate
WIDTH choice layered on top of the function's own minimum real
clearance, not more clearance-math compensation - keep those two
concerns separate if retuning either one. Every call site (there are
two per bar - hover-detection and box-drawing) must pass the SAME
value, or the hover boundary and the drawn box disagree.

**Converting a box positioned on a console OTHER than HUD_CONSOLE needs
a unit conversion first** - `draw_filled_pixel_box`/`_scaled` and every
helper underneath only understand HUD_CONSOLE cell units. The battle
log box used to be positioned in `FINE_TEXT_CONSOLE`'s own finer 160x100
grid; converting its POSITION was a straight pixel-ratio conversion
(`old_x * HUD_COLS / 160`, since both consoles span the same 1280x800
window), but its WIDTH/HEIGHT were NOT converted the same way - see
`screens/battle.rs`'s own comment on `MSG_BOX_WIDTH` for why a straight
ratio shrink would have left too narrow a box once the box's own text
ALSO moved onto the coarser `PANEL_TEXT_CONSOLE` grid. Width/height for
a cross-console box are a fresh "how much do I actually need" judgment
call, not a mechanical conversion of the old console's numbers.

## Using it: `render_helpers::draw_pixel_bar`

```rust
draw_pixel_bar(batch, x, y, width, current, max, fill_color);
```

A genuinely different rendering mechanism from every other panel-border
helper above - not a 9-slice box, a horizontal status bar (colored
proportional fill + a wood-and-gold frame drawn over it), added
2026-09-14 to replace `battle::hp_bar_string`'s ASCII `[####----]`
rendering for the player's own HP/ATB display specifically (enemies
keep the ASCII version - a deliberate scope decision: "I think we
shouldn't be able to see the enemies health and atb bar in the end," so
upgrading them now would be wasted work ahead of a later change to hide
them outright).

**Its own asset, not part of `ui_panels.png`**: `resources/
battle_bar_frame.png`, a plain 4-cell horizontal row (left cap /
tileable middle / right cap / a dedicated solid-white fill cell, glyph
indices 0/1/2/3 directly) rather than a `UiPanelTheme`-style 3x3 block,
since a bar has no top/bottom edges to tile the way a box does. Drawn
on its own new console, `BATTLE_BAR_CONSOLE` (50, fancy, same
`DISPLAY_WIDTH x DISPLAY_HEIGHT` grid as `UI_PANEL_CONSOLE`) - `batch`
must target it. Needs its own `.with_font("battle_bar_frame.png", 32,
32)` line in `main.rs`'s builder chain, registered BEFORE
`BATTLE_BAR_CONSOLE` itself - see `CLAUDE.md`'s own standing gotcha for
what happens if that line is missing (a launch-time panic, not a
compile error).

**Reuses `draw_panel_tile` directly** for the frame (no new tile-drawing
primitive needed - the exact same `set_fancy` mechanism applies, just 1
row of 3 tiles instead of a 3x3 grid) and shares `pixel_box_tiles`'
exact center-shift correction via a parallel `pixel_bar_tiles` (same
formula, same derivation - this uses the identical `set_fancy`
scale-around-a-fixed-center behavior, so the same positional drift bug
would have applied here too without correcting for it).

**The fill is one stretched `set_fancy` quad, drawn BEFORE the frame**
(same z-order reasoning as the panel boxes' own fill - `BATTLE_BAR_
CONSOLE` is a sparse `FlexiConsole` too, so the frame's own opaque wood
always paints over the fill wherever they coincide). Its width is
`current/max` of the bar's own total tile width, clamped like `hp_bar_
string`'s own ratio math; `max <= 0` renders a fully empty bar rather
than panicking. Its HEIGHT and vertical position (`BAR_FILL_HEIGHT_
FRACTION` 0.45, `BAR_FILL_TOP_FRACTION` 0.3) came from directly
measuring the real generated frame's pixel data (its opaque channel
walls sit at roughly rows 6-9 and 25-28 of each 32px tile) rather than
guessing a centered default - see `PIXEL_BAR_TILE_SCALE`'s own doc
comment for the exact numbers.

**The fill's glyph is cell 3 (a dedicated solid-white square), NOT
`to_cp437('█')`** - confirmed live 2026-09-14 that the CP437 block
character (index 219) renders NO fill at all on this font: `draw_panel_
fill` gets away with the same glyph on `ui_panels.png` only because
that's a much larger sheet, always tinted BLACK, so wherever 219
happens to sample lands opaque and the multiply zeroes it out
regardless. `battle_bar_frame.png` originally had only 3 cells -
sampling index 219 there landed in the frame's own intentionally-
transparent channel area, and a REAL (non-BLACK) fill color multiplied
by near-zero alpha rendered as nothing. Added the 4th cell specifically
so the fill never depends on where an index meant for a full CP437
character set happens to land in an unrelated, much smaller font - see
`CLAUDE.md`'s own standing gotcha for the general version of this
lesson.

`PIXEL_BAR_TILE_SCALE` (0.75, bumped from 0.5 2026-09-14 - "the health
numbers not really fitting inside the bar") is its own constant,
deliberately not reused from either panel-box scale - a status bar and
a box border are different enough visual elements that there's no
reason to assume the same number looks right for both. The fill's OWN
height is a fixed fraction of this scale, so growing the scale grows
the fill's absolute pixel height right along with it - at 0.5 the fill
rendered only ~7px tall against an ~12px overlaid text row, visibly
smaller than the text sitting on it. First-pass value, like literally
every other pixel value in this project - pending a screenshot.

**The two player bars are no longer left-aligned with each other** -
direct request 2026-09-14 ("closer together and slightly off center of
each other"): `screens/battle.rs` now has separate `PLAYER_ATB_BAR_
HUD_X`/`PLAYER_HP_BAR_HUD_X` (the HP bar sits a few columns right of
the ATB bar) instead of one shared X, and the vertical gap between
them dropped from 4 rows to 3.

**The fill's own centering math needs the SAME center-shift correction
the frame gets from `pixel_bar_tiles` - `base_col`/`base_row` are NOT
the frame's true rendered edge** - a real, confirmed bug (2026-09-14),
not the "fill stays inside the frame" request finally being acted on
for the first time (that request was already understood; the MATH
implementing it was simply wrong). `pixel_bar_tiles` shifts `base_col`/
`base_row` by `(1-scale)/2` cell-units BEFORE the frame's own true
rendered edge (see "set_fancy scales a tile around a FIXED CENTER"
above for the derivation) - the FRAME'S OWN tiles account for this
correctly (they're drawn directly at `base_col`/`base_row`, and the
correction is baked in upstream), but the ORIGINAL fill-centering code
computed the fill's position AS IF `base_col`/`base_row` already were
the true edge, missing that same `(1-s)/2` term entirely. At `PIXEL_
BAR_TILE_SCALE` (0.5) that's a 0.25 cell-unit error - HALF the bar's
own total rendered size - confirmed live as the fill rendering as a
completely separate stripe below the frame, not a subtle overlap
issue. Solved the correct formula symbolically from the real
`set_fancy` math and verified it numerically (a real script, 4
different scale values) before writing any Rust, then verified the
Rust itself with a throwaway test mirroring the same formula - removed
after confirming. Correct formulas now: `fill_center_col = base_col +
s * (fill_tiles_w - 1.0) / 2.0` (was missing `- 1.0`), `fill_center_row
= base_row + s * (TOP_FRACTION + HEIGHT_FRACTION / 2.0 - 0.5)` (was
missing `- 0.5`).

**The bars are NOT wrapped in a separate bordered panel - they sit
directly on the live background, just their own frame + fill** - a
real detour, tried and reverted the same day (2026-09-14). "The
colored bars need to be within the borders" first read as "wrap both
bars in a new box" (a `draw_filled_pixel_box_scaled` enclosing them,
same pipeline as every other converted box) - built, then directly
corrected: "I didn't want a border around the health and ATB borders I
just wanted to have the colored bars within the borders." The actual
ask was always about the FILL staying inside the bar's OWN existing
frame (`battle_bar_frame.png`'s own end caps and top/bottom strips),
which `draw_pixel_bar`'s fill positioning already handles (see
`BAR_FILL_HEIGHT_FRACTION`/`BAR_FILL_TOP_FRACTION` above) - not a
second, separate enclosing box. **Worth remembering**: "within the
borders" is genuinely ambiguous between "inside THIS element's own
border" and "inside A border drawn around it" - worth confirming which
one before building, now that it's cost a real detour once.

**Text meant to sit ON TOP of a bar needs a console later than BOTH
the fill/border AND the bar console** - a real wrinkle beyond every
other converted box's own text rule. The HP number now overlays
directly on the health bar (direct request 2026-09-14, was previously
printed on a separate line below it) via a new `BATTLE_BAR_TEXT_
CONSOLE` (51, plain no_bg, `HUD_COLS x HUD_ROWS` grid, terminal8x8.png)
- registered LAST of every console in the game. Even `PANEL_TEXT_
CONSOLE` (47), which is late enough to sit on top of any panel box's
own fill/border, is registered BEFORE `BATTLE_BAR_CONSOLE` (50) - text
on `PANEL_TEXT_CONSOLE` would render UNDER the bar's own opaque fill/
frame, not on top of it, since `BATTLE_BAR_CONSOLE` itself is
registered later. Appending yet another console (no renumbering) is
the same pattern used for `ABILITY_BAR_ICON_CONSOLE`/`PANEL_TEXT_
CONSOLE` earlier - whenever new content needs to render on top of
something ALREADY the latest-registered console, the only way is to
register something even later still.

## Still open

- A fresh screenshot confirming this round together: the fill-centering
  fix (verified numerically and by a throwaway test, but not yet
  confirmed against the real running game), the `PIXEL_BAR_TILE_SCALE`
  bump (0.5 -> 0.75), the tighter vertical gap, and the new X stagger
  between the two bars - none of it seen live yet.
- Whether a helper reducing the repeated `panel_batch`/`text_batch`
  setup boilerplate across every converted site is worth building -
  raised directly 2026-09-14 ("Is there a helper that we can make to
  make using these a lot easier?"), proposed but not yet built or
  confirmed as wanted; see `docs/journal.md`'s same-day entry for the
  concrete proposal.
- **The Item Menu's title labels print ON the border instead of above
  it** - a real, confirmed-live, not-yet-fixed regression from the
  center-shift fix (see "Box titles print at..." above for the full
  causal chain). The fix (`y - 1` instead of `y` for the title row in
  `screens/item_menu.rs`'s `print_box` and its Stats/Description boxes)
  is understood and safe to apply, just not done yet - pending explicit
  go-ahead.
- A fresh screenshot confirming this round's redundant-padding trim AND
  the new `ABILITY_BOX_EXTRA_PAD` widening - the center-shift fix's own
  overlap correction is CONFIRMED live (no more crowding), but neither
  of these has been seen live yet. The vertical (Y-axis) sign of the
  center-shift fix itself is also still unconfirmed either way - no
  report yet of it being visibly backwards, but no explicit confirmation
  it's right either (see that section's own "Confidence note" for what
  to check if it turns out wrong).
- Whether the Item Bar and Battle Bar's spacing is actually consistent
  now - flagged live as looking different from each other despite
  sharing the identical `ability_bar_box_bounds` formula (only `n` and
  `start_col` differ); not yet root-caused. Possible lead: whether
  `pixel_box_tiles`'s tile-count ROUNDING (`.round().max(2)`) lands
  differently relative to a box's nominal size at different total
  widths, in a way that reads as inconsistent spacing between a
  2-icon and a 5-icon box even with identical bounds math.
- **CONFIRMED live**: the ability-selection box, battle log border/
  content, and real color in both bars. Still unconfirmed, none seen
  live yet: the "You" label removal, the HP number now overlaid on the
  bar instead of printed below it, and the battle log's current
  position (`MSG_BOX_X`/`MSG_BOX_Y` = 10, 17 - already corrected once
  from an overcorrected 4, 12). The bordered-panel-around-both-bars
  idea was tried and reverted the same day (see "The bars are NOT
  wrapped..." above) - not something to re-attempt without the user
  explicitly asking for it again.
- Whether `player_can_act`'s "you can act now" signal (now the
  "Actions" title's color, yellow/green) reads as clearly as the old
  border-color version did - a real behavior change (signal moved from
  the whole border to one word of text), not yet confirmed to still
  work as a usable at-a-glance cue.
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
