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
/// ASCII border had. First-pass guess pending a screenshot, same as
/// every other bracket-lib pixel value in this project.
const PIXEL_BOX_TILE_SCALE: f32 = 0.375;

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
/// by stepping tile positions by `PIXEL_BOX_TILE_SCALE` cell-units per
/// tile instead of a full 1.0.
///
/// Reuses the same `WIGGLE_CONSOLE_Y_ANCHOR_OFFSET` north-anchor
/// correction `draw_portrait_fancy` applies, on the assumption (not yet
/// separately confirmed at anything other than scale 1.0) that it's a
/// property of how `set_fancy` interprets a position at all, independent
/// of the glyph's own rendered scale - same reasoning that constant's own
/// doc comment already gives for why it transfers between different
/// fancy consoles.
fn draw_panel_tile(batch: &mut DrawBatch, col: f32, row: f32, glyph: FontCharType, tint: ColorPair) {
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(col, row + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(PIXEL_BOX_TILE_SCALE, PIXEL_BOX_TILE_SCALE),
        ColorPair::new(tint.fg, bg_transparent),
        glyph,
    );
}

/// Converts a box given in HUD_CONSOLE cell units into UI_PANEL_CONSOLE's
/// own 32px-tile terms: a fractional (base_col, base_row) - the box's real
/// top-left corner, pixel-precise - and a whole (tiles_w, tiles_h) tile
/// count at `PIXEL_BOX_TILE_SCALE`, rounded to the nearest whole
/// (scaled) tile (see `draw_pixel_box`'s own doc comment for why size,
/// not position, is what's approximated). Factored out from
/// `draw_pixel_box` so the conversion math can be checked directly
/// against real numbers rather than only indirectly through whatever
/// `DrawBatch` ends up queued.
fn pixel_box_tiles(x: i32, y: i32, width: i32, height: i32) -> (f32, f32, i32, i32) {
    let px_x0 = (x * 1280) as f32 / HUD_COLS as f32;
    let px_y0 = (y * 800) as f32 / HUD_ROWS as f32;
    let px_x1 = ((x + width) * 1280) as f32 / HUD_COLS as f32;
    let px_y1 = ((y + height) * 800) as f32 / HUD_ROWS as f32;

    let tile_px = 32.0 * PIXEL_BOX_TILE_SCALE;
    let tiles_w = (((px_x1 - px_x0) / tile_px).round() as i32).max(2);
    let tiles_h = (((px_y1 - px_y0) / tile_px).round() as i32).max(2);

    (px_x0 / 32.0, px_y0 / 32.0, tiles_w, tiles_h)
}

/// The border's own REAL rendered footprint (after `pixel_box_tiles`'
/// whole-tile rounding), converted back into (x, y, width, height) on
/// HUD_CONSOLE's cell grid - confirmed live 2026-09-14 that filling to the
/// box's original, un-rounded nominal size (what `draw_filled_pixel_box`
/// did at first) doesn't exactly coincide with where the border itself
/// ends up once its own size rounds to a whole tile count, leaving the
/// fill visibly spilling past the border on some edges and short of it on
/// others. Deriving the fill's own rect from this SAME rounded footprint,
/// instead of the box's original nominal size, guarantees the two always
/// match exactly - there's no longer two independent roundings that could
/// disagree.
fn pixel_box_hud_rect(x: i32, y: i32, width: i32, height: i32) -> (i32, i32, i32, i32) {
    let (base_col, base_row, tiles_w, tiles_h) = pixel_box_tiles(x, y, width, height);
    let s = PIXEL_BOX_TILE_SCALE;

    let px_left = base_col * 32.0;
    let px_top = base_row * 32.0;
    let px_right = (base_col + tiles_w as f32 * s) * 32.0;
    let px_bottom = (base_row + tiles_h as f32 * s) * 32.0;

    let hud_x = (px_left * HUD_COLS as f32 / 1280.0).round() as i32;
    let hud_y = (px_top * HUD_ROWS as f32 / 800.0).round() as i32;
    let hud_x2 = (px_right * HUD_COLS as f32 / 1280.0).round() as i32;
    let hud_y2 = (px_bottom * HUD_ROWS as f32 / 800.0).round() as i32;

    (hud_x, hud_y, hud_x2 - hud_x, hud_y2 - hud_y)
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
/// Every tile draws at `PIXEL_BOX_TILE_SCALE` (see that constant's own
/// doc comment), not the source art's native 32px - drawn at full size
/// the border was confirmed, live, to swallow an entire Item Menu box's
/// own content. Two real simplifications versus a true pixel-perfect box
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
) {
    let (base_col, base_row, tiles_w, tiles_h) = pixel_box_tiles(x, y, width, height);
    let s = PIXEL_BOX_TILE_SCALE;
    let last_col = tiles_w - 1;
    let last_row = tiles_h - 1;

    // Corners - drawn once each, never tiled or stretched.
    draw_panel_tile(batch, base_col, base_row, theme.glyph(0, 0), tint);
    draw_panel_tile(batch, base_col + last_col as f32 * s, base_row, theme.glyph(2, 0), tint);
    draw_panel_tile(batch, base_col, base_row + last_row as f32 * s, theme.glyph(0, 2), tint);
    draw_panel_tile(
        batch,
        base_col + last_col as f32 * s,
        base_row + last_row as f32 * s,
        theme.glyph(2, 2),
        tint,
    );

    // Top/bottom edges - tiled across whatever's between the corners.
    for c in 1..last_col {
        draw_panel_tile(batch, base_col + c as f32 * s, base_row, theme.glyph(1, 0), tint);
        draw_panel_tile(
            batch,
            base_col + c as f32 * s,
            base_row + last_row as f32 * s,
            theme.glyph(1, 2),
            tint,
        );
    }

    // Left/right edges - tiled across whatever's between the corners.
    for r in 1..last_row {
        draw_panel_tile(batch, base_col, base_row + r as f32 * s, theme.glyph(0, 1), tint);
        draw_panel_tile(
            batch,
            base_col + last_col as f32 * s,
            base_row + r as f32 * s,
            theme.glyph(2, 1),
            tint,
        );
    }
}

/// A `draw_pixel_box` border plus the solid-black interior fill every real
/// call site needs alongside it (see `draw_pixel_box`'s own doc comment on
/// why it doesn't fill the interior itself yet) - `text_batch` must target
/// whatever console the box's own text/icons draw on (the fill needs to
/// land there, UNDER that text, so drawing this before the text lets it
/// naturally get overwritten at the right cells), `panel_batch` must
/// target UI_PANEL_CONSOLE. Always WHITE tint - see `draw_pixel_box`'s own
/// doc comment for why a category color crushes this shaded material.
///
/// Fills with a full-block glyph (`█`, CP437 219) tinted BLACK via `fg`,
/// NOT a space glyph with a black `bg` - confirmed live 2026-09-14 that a
/// `bg`-only fill is a silent no-op on `HUD_CONSOLE` specifically, traced
/// to `HUD_CONSOLE` being a `with_simple_console_no_bg` console: its own
/// fragment shader (`CONSOLE_NO_BG_FS` in bracket-terminal's real GLSL
/// source) receives a background color as an input but never actually
/// reads it anywhere - every fragment is either the glyph's own texture
/// (if bright enough) or a hard `discard`, with no third "solid
/// background" case at all. A "no_bg" console can still be painted a
/// solid color, just through the FOREGROUND channel on an opaque glyph
/// instead - texture_white * BLACK = black, same multiply-tint mechanism
/// every other tinted icon in this project already relies on.
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
pub fn draw_filled_pixel_box(
    text_batch: &mut DrawBatch,
    panel_batch: &mut DrawBatch,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    theme: UiPanelTheme,
) {
    // The fill's rect comes from the border's own REAL rounded footprint
    // (pixel_box_hud_rect), not the box's original nominal x/y/width/
    // height - confirmed live 2026-09-14 that using the nominal size let
    // the fill and the border (which rounds to a whole tile count) drift
    // apart by a few pixels, spilling past the border on some edges and
    // falling short on others. See pixel_box_hud_rect's own doc comment.
    let (fill_x, fill_y, fill_w, fill_h) = pixel_box_hud_rect(x, y, width, height);
    text_batch.fill_region(
        Rect::with_size(fill_x, fill_y, fill_w, fill_h),
        ColorPair::new(BLACK, BLACK),
        to_cp437('█'),
    );
    draw_pixel_box(panel_batch, x, y, width, height, theme, ColorPair::new(WHITE, BLACK));
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
