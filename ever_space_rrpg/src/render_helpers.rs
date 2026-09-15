use crate::prelude::*;

/// Draws `render`'s glyph in a single cell at (col, row) in the given
/// DrawBatch's target console coordinate space. Used to render scaled-up
/// battle portraits on the coarse BATTLE_PORTRAIT_COLS x BATTLE_PORTRAIT_ROWS
/// console (see main()): because that console's cells are much bigger than
/// the dungeon view's 32px tiles, a single glyph drawn there renders as a
/// large, stretched version of the same sprite - no repetition needed.
/// (A tiled block of many small cells, as an earlier version of this
/// function did, does NOT scale the sprite up - it repeats the same small
/// icon as a grid pattern, since tiling only produces one coherent bigger
/// image if the source art is itself split into matching fragments, which
/// our dungeonfont.png icons are not.)
pub fn draw_portrait(batch: &mut DrawBatch, col: i32, row: i32, render: Render) {
    batch.set(Point::new(col, row), render.color, render.glyph);
}

/// Same empirical correction as entity_render.rs's
/// GLIDE_CONSOLE_Y_ANCHOR_OFFSET (a glyph placed via set_fancy renders
/// one full cell too far north vs. the same position on a plain
/// console's set()) applied here too, on a DIFFERENT fancy console
/// (BATTLE_PORTRAIT_WIGGLE_CONSOLE). This is a carried-over assumption,
/// not a separately re-confirmed measurement - the reasoning is that the
/// anchor discrepancy is a property of how set_fancy interprets a
/// position in cell units generally, not something tied to one
/// console's particular pixel-per-cell size, so it should transfer. If
/// the wiggling portrait renders visibly high once you can see it, this
/// constant is the first place to check.
const WIGGLE_CONSOLE_Y_ANCHOR_OFFSET: f32 = 1.0;

/// Draws `render`'s glyph via set_fancy at a POSSIBLY-fractional (col,
/// row) position - the multi-enemy layout (see screens/battle.rs's
/// enemy_portrait_position) needs sub-cell positions the plain integer-
/// only draw_portrait above can't express (a staggered/pyramid formation
/// isn't just "which of the 5 whole rows/columns"). Same
/// WIGGLE_CONSOLE_Y_ANCHOR_OFFSET correction as draw_wiggling_portrait
/// below - both are set_fancy calls on the same fancy console, so the
/// same anchor quirk applies. This is the NOT-currently-Attacking-flash
/// case; draw_wiggling_portrait still owns the Attacking-flash shake,
/// now also updated to accept fractional coordinates for the same
/// reason.
pub fn draw_portrait_fancy(batch: &mut DrawBatch, col: f32, row: f32, render: Render) {
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(col, row + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(1.0, 1.0),
        ColorPair::new(render.color.fg, bg_transparent),
        render.glyph,
    );
}

/// Draws `render`'s glyph with a small shake instead of the plain
/// `draw_portrait`/`draw_portrait_fancy` above - only for the side
/// currently mid-"Attacking" flash (see attack_wiggle_offset). Returns
/// true if it drew (onto `batch`, which must already be targeting
/// BATTLE_PORTRAIT_WIGGLE_CONSOLE) - false if `flash` isn't an active
/// Attacking flash, in which case the caller should fall back to
/// draw_portrait/draw_portrait_fancy instead. Never both for the same
/// portrait on the same frame. Takes fractional (col, row) now (not just
/// whole numbers) for the same reason draw_portrait_fancy does.
pub fn draw_wiggling_portrait(
    batch: &mut DrawBatch,
    col: f32,
    row: f32,
    render: Render,
    flash: Option<(FlashKind, f32)>,
) -> bool {
    let remaining = match flash {
        Some((FlashKind::Attacking, remaining)) if remaining > 0.0 => remaining,
        _ => return false,
    };
    let elapsed_ms = PORTRAIT_FLASH_DURATION_MS - remaining;
    let offset_x = attack_wiggle_offset(elapsed_ms);
    // Fully transparent background (RGBA alpha 0) - the same trick that
    // let the GameOver fallen portrait and the dungeon-view tile glide
    // draw a moving glyph over a static background with no visible box
    // edge. This is the one thing different from the jiggle attempted
    // early in this project (see journal.md), which used this same
    // "coarse grid = big glyph" trick but on a console with an opaque
    // background, and had to be abandoned for exactly that reason.
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(col + offset_x, row + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(1.0, 1.0),
        ColorPair::new(render.color.fg, bg_transparent),
        render.glyph,
    );
    true
}

/// Greedily wraps `text` into lines no longer than `width` characters,
/// breaking only at word boundaries (never mid-word). Used for the
/// class-select descriptions, which vary a lot in length - some are short
/// placeholders, others (Mage's) are long enough to run off the screen
/// printed as a single line - see class_select.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// --- Arrow-key menu navigation -----------------------------------------
//
// Shared by every top-level "look at a list, pick one" screen (Adventure
// Select, Class Select, Options' rebind list, History's Overview) - a
// small, consistent cursor/pointer/highlight convention instead of each
// screen inventing its own. Number-key shortcuts stay working everywhere
// these are used (this is purely additive, not a replacement) - a
// returning player can still mash "2" without ever touching an arrow key.

/// The cursor/pointer glyph shown to the left of a selected menu row - a
/// plain CP437 right-pointing triangle (code 16), which was sitting
/// entirely unused in the project's own glyph map before this. No custom
/// art needed - it's part of the base CP437 set on every font already in
/// use (dungeonfont.png and terminal8x8.png both include it).
pub const MENU_POINTER_GLYPH: char = '►';

/// How many columns the pointer glyph sits to the left of a selected
/// row's own text.
const MENU_POINTER_GAP: i32 = 2;

/// Up/Down arrow -> a wrap-around cursor step (-1/+1); any other key ->
/// None. `len` is the number of selectable rows on screen right now (a
/// history/options list can vary in size) - wrapping is computed here so
/// every caller gets identical "top wraps to bottom and back" behavior
/// for free instead of reimplementing the modulo arithmetic per screen.
/// Returns `cursor` unchanged if `len` is 0 (nothing to navigate).
pub fn menu_nav(key: Option<VirtualKeyCode>, cursor: usize, len: usize) -> usize {
    if len == 0 {
        return cursor;
    }
    match key {
        Some(VirtualKeyCode::Up) => (cursor + len - 1) % len,
        Some(VirtualKeyCode::Down) => (cursor + 1) % len,
        _ => cursor.min(len - 1),
    }
}

/// Prints one row of a CENTERED arrow-key-navigable menu - `text` centered
/// on the currently active console, exactly like a plain
/// `ctx.print_color_centered` call, except that when `selected` is true
/// the whole row renders in YELLOW instead of `color` and a pointer glyph
/// (see MENU_POINTER_GLYPH) appears just to its left. `console_width` is
/// the active console's own column count (bracket-lib's
/// print_color_centered doesn't expose where it actually starts, so this
/// works that out independently) - pass whichever of BIG_TEXT_CONSOLE's
/// (DISPLAY_WIDTH) or HUD_CONSOLE's (HUD_COLS) width applies to the
/// console currently active.
pub fn print_menu_row_centered(
    ctx: &mut BTerm,
    console_width: i32,
    row: i32,
    color: (u8, u8, u8),
    text: &str,
    selected: bool,
) {
    let display_color = if selected { YELLOW } else { color };
    ctx.print_color_centered(row, display_color, BLACK, text);
    if selected {
        let start_col = (console_width - text.chars().count() as i32) / 2;
        ctx.print_color(
            (start_col - MENU_POINTER_GAP).max(0),
            row,
            YELLOW,
            BLACK,
            &MENU_POINTER_GLYPH.to_string(),
        );
    }
}

/// Same as print_menu_row_centered, but for a row printed at a fixed LEFT
/// column instead of centered - Options' rebind list and Class Select's
/// headline both print this way already, rather than centered.
pub fn print_menu_row_left(
    ctx: &mut BTerm,
    col: i32,
    row: i32,
    color: (u8, u8, u8),
    text: &str,
    selected: bool,
) {
    let display_color = if selected { YELLOW } else { color };
    ctx.print_color(col, row, display_color, BLACK, text);
    if selected {
        ctx.print_color(
            (col - MENU_POINTER_GAP).max(0),
            row,
            YELLOW,
            BLACK,
            &MENU_POINTER_GLYPH.to_string(),
        );
    }
}

/// Draws a hollow rectangular border - plain '-'/'|'/'+' characters, built
/// from the same DrawBatch::set + to_cp437 primitives already proven
/// throughout this file (map/portrait/arena rendering), rather than
/// reaching for a higher-level box-drawing API this project hasn't used
/// anywhere else. Used to frame the battle-menu action list - see
/// BattleTurn::PlayerMenu in battle_tick.
///
/// Named draw_ascii_box, not draw_hollow_box: bracket_lib::prelude already
/// exports its own draw_hollow_box (a different function entirely - takes
/// a &mut dyn Console and two separate RGBA fg/bg args, not a DrawBatch +
/// ColorPair). Before this project's screens/ split, this function lived
/// directly in main.rs, where a local definition silently wins over any
/// same-named glob import - so the collision was invisible. Once this
/// moved into render_helpers.rs and got glob-imported into the shared
/// prelude module alongside bracket_lib::prelude::*, both became equally
/// "just a glob import" and Rust could no longer prefer one over the
/// other, surfacing as an ambiguous-name error at every call site. Renaming
/// is the permanent fix - two same-named functions can't collide if
/// they're not actually the same name.
pub fn draw_ascii_box(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: ColorPair,
) {
    let dash = to_cp437('-');
    let pipe = to_cp437('|');
    let corner = to_cp437('+');
    for dx in 0..width {
        batch.set(Point::new(x + dx, y), color, dash);
        batch.set(Point::new(x + dx, y + height - 1), color, dash);
    }
    for dy in 0..height {
        batch.set(Point::new(x, y + dy), color, pipe);
        batch.set(Point::new(x + width - 1, y + dy), color, pipe);
    }
    batch.set(Point::new(x, y), color, corner);
    batch.set(Point::new(x + width - 1, y), color, corner);
    batch.set(Point::new(x, y + height - 1), color, corner);
    batch.set(Point::new(x + width - 1, y + height - 1), color, corner);
}

/// Which of `resources/ui_panels.png`'s four theme bands `draw_pixel_box`
/// should draw from - see that sheet's own row-mapping doc
/// (`docs/UI_Panel_Sheet_Guide.md`). Each theme owns 3 of the sheet's 12
/// rows (a 3x3 nine-slice grid, 32px cells), in the same order
/// `map_builder::dungeon_theme_pool()` already uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiPanelTheme {
    Dungeon,
    Forest,
    Sewer,
    Swamp,
}

impl UiPanelTheme {
    /// The first of this theme's 3 rows on the 3-col x 12-row
    /// `ui_panels.png` atlas.
    fn base_row(self) -> i32 {
        match self {
            UiPanelTheme::Dungeon => 0,
            UiPanelTheme::Forest => 1,
            UiPanelTheme::Sewer => 2,
            UiPanelTheme::Swamp => 3,
        }
    }

    /// The CP437 glyph index for one of this theme's 9 nine-slice pieces,
    /// given its (col, row) within that theme's own 3x3 block (0,0 =
    /// top-left corner, 1,1 = center, etc.) - `ui_panels.png` is a plain
    /// row-major atlas (glyph = row*3 + col, 3 cols), same convention
    /// bracket-lib already uses for every other custom font in this
    /// project.
    fn glyph(self, local_col: i32, local_row: i32) -> FontCharType {
        let row = self.base_row() * 3 + local_row;
        (row * 3 + local_col) as FontCharType
    }
}

/// Draws one `ui_panels.png` nine-slice tile via `draw_portrait_fancy` at
/// a fractional (col, row) on UI_PANEL_CONSOLE - `col`/`row` are already
/// How big one `ui_panels.png` tile renders on screen, as a fraction of
/// its native 32px - a border drawn at full native size read as
/// wildly oversized (confirmed live 2026-09-14: it swallowed an entire
/// Item Menu box's own list text). 0.375 * 32 = 12px, matching
/// HUD_CONSOLE's own real cell width (1280/107 ~= 11.96px) so the new
/// border reads at roughly the same visual weight the old 1-cell-thick
/// ASCII border had. Bumped 0.375 -> 0.5 (12px -> 16px) 2026-09-14 on
/// direct feedback that the Item Menu's own (large) boxes read too
/// thin/small once live - then dropped to 0.3 (~9.6px) the SAME day on
/// the opposite complaint once the dungeon HUD bars (much smaller boxes
/// than the Item Menu) were seen live. The default (large-box) scale
/// for `draw_filled_pixel_box` - see `PIXEL_BOX_TILE_SCALE_COMPACT`
/// just below for the small-box counterpart this tension ended up
/// actually needing. Deliberately doesn't touch icon/portrait
/// rendering - those go through their own separate scale (draw_portrait/
/// draw_portrait_fancy), not this constant.
const PIXEL_BOX_TILE_SCALE: f32 = 0.3;

/// The compact counterpart to `PIXEL_BOX_TILE_SCALE`, for boxes small in
/// EITHER dimension - the dungeon HUD's Item/Ability/Battle Bars (as few
/// as ONE icon wide) and the shop-item tooltip (only 3 HUD_CONSOLE rows
/// tall). Added 2026-09-14 after `PIXEL_BOX_TILE_SCALE` alone turned out
/// not to be the single-constant fix its own doc comment hoped for -
/// real numbers, not eyeballing, confirmed why: a border tile's PIXEL
/// size is fixed regardless of the box's own size, so a box small in
/// some dimension has the border eating a much bigger FRACTION of that
/// dimension than a big box does. Computed directly from this project's
/// own real geometry (`ability_bar_box_bounds`/`SHOP_TOOLTIP_WIDTH`/
/// `SHOP_TOOLTIP_HEIGHT`, not guessed): at the shared 0.3 scale, a
/// single-icon Ability Bar box's two border columns alone were ~29% of
/// its total WIDTH, and the shop tooltip's two border rows were fully
/// ~50% of its total HEIGHT - both confirmed live as "still not right"
/// even after 0.5 (too thick everywhere) was already fixed down to 0.3.
/// 0.15 (~4.8px tiles) brings the worst case (the 1-icon Ability Bar)
/// down to ~13%, comparable to what 0.3 already gives the Item Menu's
/// own much larger boxes. Still a first-pass guess pending a fresh
/// screenshot, same as `PIXEL_BOX_TILE_SCALE` itself.
pub const PIXEL_BOX_TILE_SCALE_COMPACT: f32 = 0.15;

/// Draws one `ui_panels.png` tile via `set_fancy` at `PIXEL_BOX_TILE_SCALE`,
/// at a fractional (col, row) already in UI_PANEL_CONSOLE's native 32px-cell
/// units (see `draw_pixel_box`'s own pixel-to-cell conversion) - NOT
/// `render_helpers::draw_portrait_fancy`, since that hardcodes a 1.0 scale
/// for its other callers (the multi-enemy battle formation, which needs
/// fractional POSITION but never a smaller SIZE) and changing its
/// signature would ripple into call sites this feature has nothing to do
/// with. `set_fancy` scales a glyph around its own center (confirmed
/// against bracket-terminal's real vertex-shader source, not guessed -
/// `base_pos = (aPos - center_pos) * scale + center_pos`), so shrinking
/// tiles without ALSO closing up the spacing between their centers would
/// open a gap between adjacent tiles - `draw_pixel_box` accounts for this
/// by stepping tile positions by `scale` cell-units per tile instead of a
/// full 1.0.
///
/// Reuses the same `WIGGLE_CONSOLE_Y_ANCHOR_OFFSET` north-anchor
/// correction `draw_portrait_fancy` applies, on the assumption (not yet
/// separately confirmed at anything other than scale 1.0) that it's a
/// property of how `set_fancy` interprets a position at all, independent
/// of the glyph's own rendered scale - same reasoning that constant's own
/// doc comment already gives for why it transfers between different
/// fancy consoles.
fn draw_panel_tile(
    batch: &mut DrawBatch,
    col: f32,
    row: f32,
    glyph: FontCharType,
    tint: ColorPair,
    scale: f32,
) {
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(col, row + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(scale, scale),
        ColorPair::new(tint.fg, bg_transparent),
        glyph,
    );
}

/// Converts a box given in HUD_CONSOLE cell units into UI_PANEL_CONSOLE's
/// own 32px-tile terms: a fractional (base_col, base_row) - the box's real
/// top-left corner, pixel-precise - and a whole (tiles_w, tiles_h) tile
/// count at the given `scale` (`PIXEL_BOX_TILE_SCALE` or `PIXEL_BOX_TILE_
/// SCALE_COMPACT`, whichever the caller picked), rounded to the nearest
/// whole (scaled) tile (see `draw_pixel_box`'s own doc comment for why
/// size, not position, is what's approximated). Factored out from
/// `draw_pixel_box` so the conversion math can be checked directly
/// against real numbers rather than only indirectly through whatever
/// `DrawBatch` ends up queued.
///
/// A real, confirmed positional bug fixed here 2026-09-14, on top of the
/// plain pixel-to-tile conversion: `set_fancy`'s vertex shader scales a
/// tile's raw (unscaled, one-native-cell) quad AROUND A FIXED CENTER
/// (`base_pos = (aPos - center) * scale + center`, center = position +
/// 0.5 cell - confirmed against the real GLSL source, not guessed).
/// Solved out for a corner's own true left edge: `(P - (P+0.5))*s +
/// (P+0.5) = P + (1-s)/2` - i.e. a tile positioned at `P` doesn't
/// actually render with its edge AT `P` unless `scale == 1.0`; at any
/// smaller scale it renders shifted by `+(1-scale)/2` native-cell-units
/// (0.425 cells - ~13.6px - at `PIXEL_BOX_TILE_SCALE_COMPACT`, 0.15).
/// Every tile in a box shifts by this SAME amount (a translation, not a
/// resize), so the fill and border stay aligned with EACH OTHER, but the
/// whole box's real position drifts away from whatever `x`/`y` the
/// caller actually asked for - toward the box's own interior on the
/// low-coordinate edges (crowding/appearing to overlap an icon sitting
/// just past the box's own left/top edge, on a totally independent
/// coordinate system that knows nothing about this drift) and away from
/// it on the high-coordinate edges (extra empty space past the icon on
/// the right/bottom). Confirmed live: this was small enough to go
/// unnoticed at the larger default scale (0.3, ~11.2px) but became
/// clearly visible - "the border is overlapping the icons," "too much
/// space on the right hand side of the item bar" - once `PIXEL_BOX_
/// TILE_SCALE_COMPACT` made the shift proportionally huge relative to
/// the border's own now-thin tiles. Subtracting the same `(1-scale)/2`
/// back out here, before any tile position is computed from `base_col`/
/// `base_row`, cancels it out so the border's real rendered position
/// matches the caller's intended `x`/`y` regardless of scale.
///
/// Confidence note: the X-axis derivation above is exact, solved
/// directly from the real vertex shader with no assumptions. The SAME
/// correction is applied to `base_row` too, on the reasoning that the
/// vertex shader's `vec2` transform treats X and Y identically - but
/// `FlexiConsole::set_fancy` inverts `position.y` before this transform
/// runs (`invert_pos.y = height - position.y`, the same coordinate flip
/// already behind `WIGGLE_CONSOLE_Y_ANCHOR_OFFSET`), which makes a fully
/// independent hand-derivation for Y genuinely error-prone. The row
/// correction's sign was chosen to match the ALREADY-CONFIRMED live
/// symptom (the border crowding down into an icon at the top, excess
/// room at the bottom - structurally the same pattern the X-axis fix
/// addresses), not re-derived from the flip in isolation. If a fresh
/// screenshot shows the vertical alignment got WORSE instead of better,
/// this is the first place to check - the fix for Y specifically may be
/// `+ center_shift` instead of `- center_shift`.
fn pixel_box_tiles(x: i32, y: i32, width: i32, height: i32, scale: f32) -> (f32, f32, i32, i32) {
    let px_x0 = (x * 1280) as f32 / HUD_COLS as f32;
    let px_y0 = (y * 800) as f32 / HUD_ROWS as f32;
    let px_x1 = ((x + width) * 1280) as f32 / HUD_COLS as f32;
    let px_y1 = ((y + height) * 800) as f32 / HUD_ROWS as f32;

    let tile_px = 32.0 * scale;
    let tiles_w = (((px_x1 - px_x0) / tile_px).round() as i32).max(2);
    let tiles_h = (((px_y1 - px_y0) / tile_px).round() as i32).max(2);

    let center_shift = (1.0 - scale) / 2.0;
    (px_x0 / 32.0 - center_shift, px_y0 / 32.0 - center_shift, tiles_w, tiles_h)
}

/// Draws the box's solid black interior fill as ONE `set_fancy` quad,
/// stretched via a non-uniform `PointF` scale to cover EXACTLY the same
/// real footprint as `draw_pixel_box`'s own border tiles - built from the
/// SAME `base_col`/`base_row`/`tiles_w`/`tiles_h` numbers `pixel_box_tiles`
/// already produces for the border, not an independently-rounded rect on
/// a different console's coarser cell grid (the old approach, via a now-
/// removed `pixel_box_hud_rect` - confirmed live 2026-09-14 that snapping
/// to HUD_CONSOLE's own ~12px cells could drift the fill a few pixels off
/// the border's real sub-pixel footprint, visibly spilling past or falling
/// short of it, especially with the border itself only ~12-16px thick).
///
/// Sharing the border's exact numbers works because this fill lives on
/// UI_PANEL_CONSOLE too (not HUD_CONSOLE) - confirmed against
/// bracket-terminal's real source that `with_fancy_console` consoles are
/// backed by `FlexiConsole`, a SPARSE console whose `set_fancy` PUSHES a
/// new `FlexiTile` onto a `Vec` rather than overwriting a fixed per-cell
/// array the way `SimpleConsole::set` does - so this fill and the
/// border's own corner/edge tiles can freely coexist and even overlap on
/// the same console without either one wiping the other out (draw this
/// BEFORE the border so the border's own art still paints over it,
/// same reasoning `draw_filled_pixel_box` already documents).
///
/// The position/scale math: `rebuild_vertices` (bracket-terminal's real
/// fancy-console backend) always builds one native-cell-sized quad
/// starting AT a tile's `position` and scales it around ITS OWN center
/// (`position + 0.5` cell) by `t.scale`, independently per axis (GLSL
/// `base_pos *= aScale` is a component-wise vec2 multiply, confirmed
/// straight from the real vertex shader source, not guessed) - so a
/// single glyph CAN be stretched into an arbitrary non-square rectangle,
/// not just resized uniformly. `draw_pixel_box`'s own tiles sit at
/// `base_col + i*s` for column `i`, each `s` wide, so the border's real
/// left/right edges work out to `base_col + 0.5 -/+ s/2` and
/// `base_col + 0.5 + s*(tiles_w - 1) -/+ s/2` respectively - center
/// `base_col + 0.5 + s*(tiles_w - 1)/2`, width `tiles_w * s`. Matching
/// that with one `set_fancy` call means: `position = center - 0.5`
/// (`base_col + s*(tiles_w - 1)/2`), `scale = (tiles_w*s, tiles_h*s)`.
fn draw_panel_fill(
    batch: &mut DrawBatch,
    base_col: f32,
    base_row: f32,
    tiles_w: i32,
    tiles_h: i32,
    s: f32,
) {
    let center_col = base_col + s * (tiles_w - 1) as f32 / 2.0;
    let center_row = base_row + s * (tiles_h - 1) as f32 / 2.0;
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(center_col, center_row + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(tiles_w as f32 * s, tiles_h as f32 * s),
        ColorPair::new(BLACK, bg_transparent),
        to_cp437('█'),
    );
}

/// Real pixel-art replacement for `draw_ascii_box` - draws a hollow
/// nine-slice border (4 corners + 4 tiled edges, no filled interior yet,
/// same hollow shape `draw_ascii_box` already has) from
/// `resources/ui_panels.png` instead of `-`/`|`/`+` characters. See
/// `docs/UI_Panel_Sheet_Guide.md` for the full sheet layout and the
/// generation recipe behind it.
///
/// `x`/`y`/`width`/`height` are in the SAME HUD_CONSOLE cell units a
/// `draw_ascii_box` call already used - this converts them to real screen
/// pixels internally (same ratio math `systems/hud.rs`'s pixel-overlap
/// helpers already use) so an existing call site's box position carries
/// over unchanged, just swap the function. `batch` must already be
/// targeting UI_PANEL_CONSOLE.
///
/// Every tile draws at the given `scale` (`PIXEL_BOX_TILE_SCALE` or
/// `PIXEL_BOX_TILE_SCALE_COMPACT` - see those constants' own doc
/// comments), not the source art's native 32px - drawn at full size the
/// border was confirmed, live, to swallow an entire Item Menu box's own
/// content. Two real simplifications versus a true pixel-perfect box
/// remain even at the smaller scale, both because tiles only ever draw as
/// whole units (no per-tile stretching yet - see UI_PANEL_CONSOLE's own
/// doc comment in main.rs for why that's deliberate for now):
/// - The box's own TOP-LEFT corner lands at the exact right pixel (via
///   `set_fancy`'s fractional positioning), but its overall WIDTH/HEIGHT
///   is rounded to the nearest whole tile count at the smaller scale -
///   within a few pixels of the original ASCII box's exact footprint,
///   not pixel-identical.
/// - `width`/`height` below this function's own minimum (needs at least
///   2 tiles per axis to have distinct corners) are clamped up to that
///   minimum rather than drawing something degenerate.
pub fn draw_pixel_box(
    batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    theme: UiPanelTheme,
    tint: ColorPair,
    scale: f32,
) {
    let (base_col, base_row, tiles_w, tiles_h) = pixel_box_tiles(x, y, width, height, scale);
    let s = scale;
    let last_col = tiles_w - 1;
    let last_row = tiles_h - 1;

    // Corners - drawn once each, never tiled or stretched.
    draw_panel_tile(batch, base_col, base_row, theme.glyph(0, 0), tint, s);
    draw_panel_tile(batch, base_col + last_col as f32 * s, base_row, theme.glyph(2, 0), tint, s);
    draw_panel_tile(batch, base_col, base_row + last_row as f32 * s, theme.glyph(0, 2), tint, s);
    draw_panel_tile(
        batch,
        base_col + last_col as f32 * s,
        base_row + last_row as f32 * s,
        theme.glyph(2, 2),
        tint,
        s,
    );

    // Top/bottom edges - tiled across whatever's between the corners.
    for c in 1..last_col {
        draw_panel_tile(batch, base_col + c as f32 * s, base_row, theme.glyph(1, 0), tint, s);
        draw_panel_tile(
            batch,
            base_col + c as f32 * s,
            base_row + last_row as f32 * s,
            theme.glyph(1, 2),
            tint,
            s,
        );
    }

    // Left/right edges - tiled across whatever's between the corners.
    for r in 1..last_row {
        draw_panel_tile(batch, base_col, base_row + r as f32 * s, theme.glyph(0, 1), tint, s);
        draw_panel_tile(
            batch,
            base_col + last_col as f32 * s,
            base_row + r as f32 * s,
            theme.glyph(2, 1),
            tint,
            s,
        );
    }
}

/// A `draw_pixel_box` border plus the solid-black interior fill every real
/// call site needs alongside it (see `draw_pixel_box`'s own doc comment on
/// why it doesn't fill the interior itself yet). `panel_batch` must target
/// UI_PANEL_CONSOLE - fill AND border both live there now (see
/// `draw_panel_fill`'s own doc comment for why that's safe on this
/// specific sparse console type). Always WHITE tint on the border - see
/// `draw_pixel_box`'s own doc comment for why a category color crushes
/// this shaded material.
///
/// Any text a caller prints on top of this box (title, list entries, a
/// number label) needs its OWN separately-registered, later console
/// (PANEL_TEXT_CONSOLE) - NOT this function's `panel_batch`, and NOT
/// HUD_CONSOLE either. See PANEL_TEXT_CONSOLE's own doc comment in
/// main.rs and `screens/item_menu.rs`'s `print_box` for the full story:
/// HUD_CONSOLE (and any other `SimpleConsole`) stores one fixed `Tile`
/// per cell, so text printed there REPLACES whatever was in that cell
/// rather than layering onto it - fine for the border/fill above, which
/// no longer touches HUD_CONSOLE at all, but still a real trap for any
/// TEXT this box's caller prints, since HUD_CONSOLE is a `SimpleConsole`
/// even though this fill isn't drawn there anymore.
///
/// Fills with a full-block glyph (`█`, CP437 219) tinted BLACK via `fg`,
/// same multiply-tint mechanism every other tinted icon in this project
/// already relies on (texture_white * BLACK = black) - not a `bg` color,
/// which was confirmed live 2026-09-14 to be a silent no-op on the
/// `no_bg` console this fill used to target (see `docs/UI_Panel_Sheet_
/// Guide.md` for the full shader-source trace); moving the fill to
/// UI_PANEL_CONSOLE's fancy shader (`SPRITE_CONSOLE_FS` - `FragColor =
/// original * ourColor`, no discard at all) keeps working the same way
/// for the same underlying reason.
///
/// Fills the box's FULL nominal area, title row included - a `has_title`
/// flag briefly existed here (2026-09-14) to carve the title's own row
/// out of the fill, on the theory that a title inheriting a black
/// background must have been unintentional. Reverted the same day on
/// direct correction: the black fill reaching all the way up to meet the
/// title row was specifically the look wanted ("I like the background
/// sticking up out of the top to where the label is"), and - more
/// importantly - a title row left OUTSIDE the fill has nothing solid
/// behind it at all, so wherever the live background behind the menu is
/// light, the title becomes hard to read exactly like any other
/// unfilled text would. One full-box fill for every caller, no
/// exceptions.
///
/// Draws at `PIXEL_BOX_TILE_SCALE`, the default sized for large boxes
/// (the Item Menu, the Paused screen's Hints box) - use `draw_filled_
/// pixel_box_scaled` directly for anything small in either dimension
/// (the dungeon HUD bars, the shop tooltip), which need `PIXEL_BOX_
/// TILE_SCALE_COMPACT` instead. See that constant's own doc comment for
/// the real numbers behind why one scale doesn't fit every box size.
pub fn draw_filled_pixel_box(
    panel_batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    theme: UiPanelTheme,
) {
    draw_filled_pixel_box_scaled(panel_batch, x, y, width, height, theme, PIXEL_BOX_TILE_SCALE);
}

/// `draw_filled_pixel_box` with an explicit `scale` instead of always
/// `PIXEL_BOX_TILE_SCALE` - see that function's own doc comment, and
/// `PIXEL_BOX_TILE_SCALE_COMPACT`'s, for when to reach for this instead.
pub fn draw_filled_pixel_box_scaled(
    panel_batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    theme: UiPanelTheme,
    scale: f32,
) {
    let (base_col, base_row, tiles_w, tiles_h) = pixel_box_tiles(x, y, width, height, scale);
    // Fill drawn BEFORE the border so the border's own corner/edge art
    // still paints over it wherever they'd otherwise coincide - see
    // draw_panel_fill's own doc comment for why this doesn't erase the
    // fill the way it would on a SimpleConsole (it can't; this console
    // only ever adds tiles, never overwrites one already queued).
    draw_panel_fill(panel_batch, base_col, base_row, tiles_w, tiles_h, scale);
    draw_pixel_box(panel_batch, x, y, width, height, theme, ColorPair::new(WHITE, BLACK), scale);
}

/// Small horizontal shake for a portrait mid-"Attacking" flash - a few
/// quick back-and-forth oscillations that decay to nothing exactly as the
/// flash itself expires, so the portrait is back in its resting spot the
/// instant the white flash fades (no separate timer - this only needs the
/// same remaining-ms value Battle::enemy_flash/player_flash already
/// track). Only ever called for FlashKind::Attacking - a "Hit" flash
/// keeps its plain color tint with no motion, since a wiggle reads as the
/// attacker's own flourish, not something the target should do.
///
/// Returned in fractional CELLS, not pixels - draw_battle_arena adds this
/// straight onto the portrait's normal (col, row) before handing it to
/// set_fancy on BATTLE_PORTRAIT_WIGGLE_CONSOLE, which shares console 3's
/// coarse, huge-celled grid - so even a small fraction of a cell here
/// reads as a very visible shake.
pub fn attack_wiggle_offset(elapsed_ms: f32) -> f32 {
    const WIGGLE_CYCLES: f32 = 3.0;
    const WIGGLE_AMPLITUDE_CELLS: f32 = 0.12;
    let progress = (elapsed_ms / PORTRAIT_FLASH_DURATION_MS).min(1.0);
    let decay = 1.0 - progress;
    let phase = progress * WIGGLE_CYCLES * std::f32::consts::TAU;
    WIGGLE_AMPLITUDE_CELLS * decay * phase.sin()
}

/// Tints `base`'s foreground color for a brief post-action flash - white
/// for Attacking, red for Hit - or returns it unchanged once the flash has
/// expired or was never set. Background is left untouched: console 3 is a
/// plain no_bg console, so its background is never actually rendered
/// anyway. See Battle::enemy_flash/player_flash.
pub fn flash_tint(base: ColorPair, flash: Option<(FlashKind, f32)>) -> ColorPair {
    if let Some((kind, remaining)) = flash {
        if remaining > 0.0 {
            let flash_color = match kind {
                FlashKind::Attacking => WHITE,
                FlashKind::Hit => RED,
            };
            return ColorPair::new(flash_color, base.bg);
        }
    }
    base
}

/// Draws a stylized tree/feature silhouette - a stepped triangular canopy
/// over a short trunk - centered at (cx, cy) on whatever console the given
/// DrawBatch targets. Fills both background and foreground (a brighter
/// shade of the same color) so it reads as a solid shape rather than the
/// thin scattered marks a foreground-only glyph gives.
pub fn draw_tree(
    batch: &mut DrawBatch,
    glyph: FontCharType,
    canopy_color: RGB,
    trunk_color: RGB,
    cx: i32,
    cy: i32,
) {
    let canopy_fg = RGB::from_f32(
        (canopy_color.r * 1.3).min(1.0),
        (canopy_color.g * 1.3).min(1.0),
        (canopy_color.b * 1.3).min(1.0),
    );
    let widths = [1, 3, 5, 7, 5, 3, 1];
    for (i, &w) in widths.iter().enumerate() {
        let row = cy - 3 + i as i32;
        let half = w / 2;
        for dx in -half..=half {
            batch.set(
                Point::new(cx + dx, row),
                ColorPair::new(canopy_fg, canopy_color),
                glyph,
            );
        }
    }

    let trunk_fg = RGB::from_f32(
        (trunk_color.r * 1.3).min(1.0),
        (trunk_color.g * 1.3).min(1.0),
        (trunk_color.b * 1.3).min(1.0),
    );
    for row in (cy + 4)..=(cy + 5) {
        batch.set(
            Point::new(cx, row),
            ColorPair::new(trunk_fg, trunk_color),
            glyph,
        );
    }
}

/// Blends `base` brighter near the center of a w x h grid (a "clearing")
/// and darker toward the edges (deeper shadow), rather than only ever
/// darkening outward from a neutral center.
pub fn vignette(base: RGB, x: i32, y: i32, w: i32, h: i32) -> RGB {
    let cx = w as f32 / 2.0;
    let cy = h as f32 / 2.0;
    let dx = (x as f32 - cx) / cx;
    let dy = (y as f32 - cy) / cy;
    let dist = (dx * dx + dy * dy).sqrt().min(1.0);
    let factor = 1.35 - dist * 0.8;
    RGB::from_f32(
        (base.r * factor).min(1.0),
        (base.g * factor).min(1.0),
        (base.b * factor).min(1.0),
    )
}

/// Component-wise multiplies `base` by `tint` (each in 0.0-1.0), clamped -
/// used to recolor the same floor/wall palette a level's theme already
/// defines rather than hardcoding a whole separate palette for the
/// GameOver/Victory backgrounds. E.g. a reddish tint darkens/desaturates
/// toward red for defeat; a warm gold tint brightens toward gold for
/// victory.
pub fn tint_color(base: RGB, tint: RGB) -> RGB {
    RGB::from_f32(
        (base.r * tint.r).min(1.0),
        (base.g * tint.g).min(1.0),
        (base.b * tint.b).min(1.0),
    )
}
