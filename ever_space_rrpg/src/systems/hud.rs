use crate::prelude::*;

/// Tint used for a greyed-out (unowned) Ability Bar / Battle Bar icon.
/// Deliberately NOT the named DARK_GRAY (169,169,169) that works fine
/// for plain text elsewhere in this HUD - bracket-lib's console shader
/// combines a glyph's color as `texture_pixel * this_color` (a straight
/// multiply, confirmed against bracket-terminal's own GLSL source), so
/// for a monochrome white TEXT glyph that multiply gives a clean grey
/// (white * grey = grey). These ability icons are full-color custom
/// sprite art, though - multiplying a colored pixel by a light grey like
/// DARK_GRAY only dims it to ~66% brightness while preserving its exact
/// hue, which reads as "slightly darker," not "disabled." A much darker
/// value crushes brightness hard enough to read as greyed-out regardless
/// of the sprite's original color, at the cost of some hue still
/// technically surviving the multiply (unavoidable without a real
/// desaturated version of each sprite - a straight multiply can dim a
/// color but can never truly desaturate it).
const UNOWNED_ICON_TINT: (u8, u8, u8) = (40, 40, 40);

/// The player-status frame's class-portrait icon lives at this
/// ABILITY_BAR_CONSOLE cell (40x40px) - one full icon-size down and right
/// from the screen's true top-left corner (cell (1,1), not (0,0)), so the
/// whole frame isn't flush against the physical screen edge. Separate
/// from the health bar's own HEALTH_BAR_* constants below because the
/// icon is drawn on a different console (the coarse icon grid, not
/// HUD_CONSOLE's fine text one).
const HEALTH_FRAME_ICON_COL: i32 = 1;
const HEALTH_FRAME_ICON_ROW: i32 = 1;

/// Leaves room, in HUD_CONSOLE columns, for the icon's real right edge at
/// (HEALTH_FRAME_ICON_COL + 1) * 40px - HUD_CONSOLE's own cells are
/// roughly 12px each (1280 / HUD_COLS), so 80px needs about 7 of them.
/// +2 beyond that bare minimum (same gap the Item/Ability/Battle bars use
/// between each other - see BAR_GROUP_GAP_COLS) rather than the exact
/// rounded-up edge: some class portraits' art (e.g. Mage's staff) reaches
/// close to its own cell's edge, and column 7 alone left only ~4px of
/// real clearance - visibly crowding the bar.
const HEALTH_BAR_START_COL: i32 = 9;
/// Lines the bar's top edge up with the icon's own top edge - both now
/// start one icon-size down from the physical top of the screen (see
/// HEALTH_FRAME_ICON_ROW), just in each console's own row units
/// (HEALTH_FRAME_ICON_ROW * 40px, converted to HUD_CONSOLE's ~12px rows).
const HEALTH_BAR_START_ROW: i32 = 3;
/// How many HUD_CONSOLE columns wide the health bar is. Confirmed via
/// screenshot at this width with the frame's current position - the
/// general how-to-play hint that used to also occupy this row moved to
/// the Paused screen (see screens/pause.rs), so there's nothing left to
/// collide with even if this grows later.
const HEALTH_BAR_WIDTH: i32 = 16;

/// Small status-effect badges, just below the class-portrait icon, for
/// lasting effects on the player (Invisible Cloak, Stealth, Ice Armor -
/// anything that persists for multiple turns/attacks rather than
/// resolving instantly). These are the abilities' own real dungeonfont
/// sprite icons - see BUFF_BADGE_CONSOLE's own doc comment in main.rs
/// for why that needed a dedicated console (32px cells: smaller than
/// the 40px portrait/Ability Bar icons, but still a resolution this
/// project's art is already drawn at elsewhere, unlike an arbitrary
/// smaller size that would need the source art scaled down).
///
/// Column/row here are in BUFF_BADGE_CONSOLE's own 32px-cell grid, NOT
/// HEALTH_FRAME_ICON_COL/ROW's 40px one - converted by hand from the
/// portrait's real pixel footprint (col/row 1 on a 40px grid = pixels
/// [40,80)x[40,80)): BUFF_BADGE_COL_START sits under the portrait's
/// left edge (40px / 32px ~= 1.25, floored - starting under rather than
/// past it), BUFF_BADGE_ROW clears its bottom edge (80px / 32px = 2.5,
/// ceiling to row 3 for real clearance, same reasoning
/// ability_bar_box_bounds' own doc comment already gives for why a
/// plain truncating division isn't enough on the far side of a gap).
const BUFF_BADGE_COL_START: i32 = 1;
const BUFF_BADGE_ROW: i32 = 3;

/// Draws one buff badge at `*col` if `active`, then advances `*col` so
/// the next badge (if any) sits immediately to its right - shared by all
/// 3 checks in hud() rather than repeating the same draw+advance for
/// each. `name` is the ability's own template name (e.g. "Ice Armor") -
/// reuses glyph_for_item_name so the badge always matches whatever
/// sprite that ability is actually mapped to, rather than a second,
/// independently-hardcoded glyph that could drift out of sync with it.
/// Drawn at full color (WHITE, i.e. no tint) rather than a distinct
/// color per buff the way an earlier plain-text version of this did -
/// this console's shader multiplies a glyph's real color by the color
/// passed in (see UNOWNED_ICON_TINT's own doc comment on this same
/// multiply for full-color custom art), so a strong tint here would
/// distort each icon's actual colors instead of just distinguishing
/// them; the icons themselves already look different from each other.
fn draw_buff_badge(batch: &mut DrawBatch, col: &mut i32, active: bool, name: &str) {
    if !active {
        return;
    }
    if let Some(glyph) = glyph_for_item_name(name) {
        draw_portrait(
            batch,
            *col,
            BUFF_BADGE_ROW,
            Render {
                color: ColorPair::new(WHITE, BLACK),
                glyph: to_cp437(glyph),
            },
        );
        *col += 1;
    }
}

/// How far below the player's own HUD_CONSOLE row the shop item tooltip
/// box starts - one console-0 tile (the player's own dungeon-view sprite)
/// is ~2.68 HUD_CONSOLE rows (32px / ~11.94px), so this needs to clear at
/// least that much to avoid sitting on top of the player, plus a little
/// breathing room. First-pass pixel guess - see CLAUDE.md's bracket-lib
/// layout gotcha - pending a screenshot.
const SHOP_TOOLTIP_ROW_OFFSET: i32 = 4;
/// The shop item tooltip box's fixed width/height, in HUD_CONSOLE cells -
/// wide enough for the longest "{name} x{count} - {price}g" line
/// currently possible, with margin either side.
const SHOP_TOOLTIP_WIDTH: i32 = 40;
// Bumped 3 -> 4 2026-09-14 on direct feedback that the text sat too
// close to the top edge, with a visibly larger gap of black space below
// it than above - the extra row gives the text (moved to box_y + 2,
// see its own print_color call) real clearance on both sides instead of
// being pinned to the row right under the border's top edge.
const SHOP_TOOLTIP_HEIGHT: i32 = 4;

/// Extra HUD_CONSOLE columns of box width, each side, for the Battle Bar
/// specifically. Originally 3 (a deliberate WIDTH choice, direct request
/// 2026-09-14: "wider ability boxes for the abilities and smaller for
/// the items"), dropped to 1 2026-09-15 on direct feedback ("Too much
/// padding on the sides") once it was the Battle Bar's own turn in this
/// session's one-bar-at-a-time pass - same value the Ability Bar's own
/// side pad already settled on the round before.
const BATTLE_BOX_EXTRA_SIDE_PAD: i32 = 1;
/// No bottom-padding parameter existed for the Battle Bar's box at all
/// before 2026-09-15 - same gap the Item Bar and Ability Bar both had,
/// same fix, same "no padding on the bottom" feedback. Scoped to only
/// the Battle Bar's own draw call, not the shared `ability_bar_box_
/// bounds` function.
const BATTLE_BOX_EXTRA_BOTTOM_PAD: i32 = 1;

/// Extra HUD_CONSOLE columns/rows added ONLY to the Item Bar's own box
/// width/height (right and bottom edges respectively) - direct feedback
/// 2026-09-15 that those two edges specifically sat too close to the
/// potion/scroll icons. Whole HUD_CONSOLE units, not fractional ones -
/// `PanelBox`'s geometry (and the `ability_bar_box_bounds` math feeding
/// it) is i32 HUD_CONSOLE cells throughout; genuine fractional-cell
/// padding would mean threading `f32` positions through that whole
/// pipeline (`pixel_box_tiles`'s own center-shift math in particular)
/// rather than just this one box's own margin, for a difference small
/// enough (a HUD_CONSOLE cell is ~12px; half of one is ~6px) it may not
/// even read as different in a screenshot - not attempted here. First-
/// pass value pending a screenshot round, same as every other bracket-
/// lib pixel value in this project.
const ITEM_BOX_EXTRA_RIGHT_PAD: i32 = 1;
const ITEM_BOX_EXTRA_BOTTOM_PAD: i32 = 1;

/// Same idea as the Item Bar's own extra pad above, but for the Ability
/// Bar specifically - direct feedback 2026-09-15 that the Ability Bar's
/// OLD side padding (it used to share `BATTLE_BOX_EXTRA_SIDE_PAD`/3 with
/// the Battle Bar) was "WAY too much," and that its bottom edge had none
/// at all, same gap the Item Bar had. Deliberately its own separate
/// constant, not a shared one with either other bar - see this session's
/// standing preference to tune one bar at a time without changing a
/// sibling's own value.
const ABILITY_BOX_EXTRA_SIDE_PAD: i32 = 1;
const ABILITY_BOX_EXTRA_BOTTOM_PAD: i32 = 1;

/// Backlog item 9, 2026-09-15: the Item/Ability/Battle Bar's own alpha
/// while the player's real map position is underneath that specific bar
/// - direct design confirmation (AskUserQuestion): fade the fill, keep
/// the border/icons fully opaque. Widened the SAME day, on direct
/// correction from a screenshot ("I want the bar and the icon and the
/// background to go transparent") - it now drives the border and icon
/// alpha too, not just the fill; renamed from `BAR_FADED_FILL_ALPHA`
/// to match. Still scoped to just these 3 dungeon bars (not the
/// player-status portrait or any other panel). Not 0.0 - a fully-
/// invisible box would read as "hidden," not "faded"; this is a first-
/// pass value pending a screenshot round, same as every other
/// bracket-lib pixel/color value in this project.
const BAR_FADE_ALPHA: f32 = 0.15;

/// Extra HUD_CONSOLE cells of slack, each side, added to a bar's own box
/// before checking whether the player is "under" it - direct correction
/// 2026-09-15 that a literal, exact containment check ("it still is
/// opaque" with the player visibly right at/just past the box's own
/// edge) missed real cases. `player_hud`'s own single point comes from
/// the player's map-tile corner, not the true center of their ~32px-wide
/// on-screen sprite (see `mouse_to_hud`'s own conversion) - a few
/// HUD_CONSOLE cells of margin covers that mismatch without needing to
/// separately track the sprite's own real footprint. First-pass value,
/// same as `BAR_FADE_ALPHA` - both pending a live screenshot to confirm
/// the right feel, not derived from an exact measurement.
const PLAYER_UNDER_BAR_MARGIN: i32 = 4;

/// True if `player_hud` (see its own call site's comment) falls inside
/// the given box's footprint - expanded by `PLAYER_UNDER_BAR_MARGIN` on
/// every side (see its own doc comment) - in the SAME HUD_CONSOLE cell
/// units `ability_bar_box_bounds` returns its own box in. The natural
/// reusable "is the player under this bar" check for all 3 dungeon
/// bars, since each already computes its own box this same way.
fn player_under_box(player_hud: Option<Point>, box_x: i32, box_y: i32, box_w: i32, box_h: i32) -> bool {
    match player_hud {
        Some(p) => {
            let x0 = box_x - PLAYER_UNDER_BAR_MARGIN;
            let y0 = box_y - PLAYER_UNDER_BAR_MARGIN;
            let x1 = box_x + box_w + PLAYER_UNDER_BAR_MARGIN;
            let y1 = box_y + box_h + PLAYER_UNDER_BAR_MARGIN;
            p.x >= x0 && p.x < x1 && p.y >= y0 && p.y < y1
        }
        None => false,
    }
}

/// The icon color for one Item/Ability/Battle Bar slot at the given
/// alpha (1.0 = the normal fully-opaque icon) - same owned/unowned tint
/// choice every bar already made, just with `alpha` threaded into both
/// the icon's own fg AND its cell's fallback bg (see `draw_filled_pixel_
/// box_scaled`'s own doc comment for why bracket-terminal's real fancy-
/// console shader needs BOTH touched to actually fade a sprite cell,
/// not just its visible content) - added 2026-09-15 for backlog item 9.
fn bar_icon_color(owned: bool, alpha: f32) -> ColorPair {
    let (r, g, b) = if owned { WHITE } else { UNOWNED_ICON_TINT };
    ColorPair::new(
        RGBA::from_u8(r, g, b, (alpha * 255.0).round() as u8),
        RGBA::from_u8(0, 0, 0, (alpha * 255.0).round() as u8),
    )
}

/// The hotkey label for Ability Bar slot `i` - matches
/// player_input.rs::use_ability's key order exactly (1-9, then 0 for the
/// 10th slot), NOT just "i + 1", which would read "10" for the 10th slot
/// - not a key that exists.
fn ability_bar_key_label(i: usize) -> &'static str {
    const LABELS: [&str; ABILITY_BAR_MAX_SLOTS] =
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
    LABELS.get(i).copied().unwrap_or("")
}

/// Converts Ability Bar column `col`'s icon position into a (col, row) on
/// HUD_CONSOLE for its number label - both consoles share the same
/// physical 1280x800 window, so this is a ratio of cell counts, same
/// technique class_select's headline/description split already uses
/// between BIG_TEXT_CONSOLE and HUD_CONSOLE. The label sits one
/// HUD_CONSOLE row above the bar's own top pixel edge (just above each
/// icon, not overlapping it) and NEAR (not exactly at) that icon's own
/// LEFT pixel edge - nudged +1 HUD_CONSOLE column right 2026-09-15 on
/// direct feedback that flush-left read as "a little too far to the
/// left." Still not centered - a single digit is narrow enough that
/// left-of-center still reads as "belonging to" the icon immediately to
/// its right, and it avoids needing to also know the label's own
/// rendered width to center it.
fn ability_bar_label_position(col: i32) -> (i32, i32) {
    let bar_top_px = ability_bar_row() * (800 / ABILITY_BAR_ROWS);
    let label_row = (bar_top_px * HUD_ROWS / 800) - 1;

    let icon_left_px = col * (1280 / ABILITY_BAR_COLS);
    let label_col = icon_left_px * HUD_COLS / 1280 + 1;

    (label_col, label_row)
}

/// The stack-count badge's (col, row) position on
/// ABILITY_BAR_BADGE_CONSOLE for the icon at Ability/Item/Battle Bar
/// column `col`, row `bar_row` - the bottom-right corner of the icon's own
/// pixel footprint, the opposite corner from the hotkey label's top-left
/// position (see ability_bar_label_position) so the two numbers are never
/// ambiguous. Uses ABILITY_BAR_BADGE_CONSOLE's own doc comment's reasoning
/// for why this can't just be HUD_CONSOLE. Both edges land on the LAST
/// HUD-grid row/col whose pixels still fall inside the icon's footprint
/// (the "-1 before converting" trick ability_bar_box_bounds' doc comment
/// already explains) rather than the row/col just past it.
fn ability_bar_count_position(col: i32, bar_row: i32) -> (i32, i32) {
    let icon_right_px = (col + 1) * (1280 / ABILITY_BAR_COLS);
    let icon_bottom_px = (bar_row + 1) * (800 / ABILITY_BAR_ROWS);

    let bottom_row = ((icon_bottom_px - 1) * HUD_ROWS) / 800;
    let right_col = ((icon_right_px - 1) * HUD_COLS) / 1280;

    // "x{n}" is at least 2 characters - start 2 cols left of the
    // rightmost one so it ends there, with one more col of margin so it
    // doesn't sit flush against the icon's true right edge.
    (right_col - 2, bottom_row)
}

/// Draws the "x{count}" stack-count badge for one Item/Ability/Battle Bar
/// icon - see docs/ideas.md's "Stack-count badge" entry for the agreed
/// design: bottom-right corner, matching the shop's own existing
/// "Healing Potion x5" quantity convention, plain white (never yellow -
/// that's the hotkey label's color, and using it here would blur the two
/// together). Does nothing for `count <= 1` so a lone item stays exactly
/// as clean as it already looks.
fn draw_stack_count_badge(batch: &mut DrawBatch, col: i32, bar_row: i32, count: i32) {
    if count <= 1 {
        return;
    }
    let (label_col, label_row) = ability_bar_count_position(col, bar_row);
    batch.print_color(
        Point::new(label_col, label_row),
        &format!("x{}", count),
        ColorPair::new(WHITE, BLACK),
    );
}

/// The (x, y, width, height) box - in HUD_CONSOLE cell terms, for
/// `render_helpers::PanelBox` - that should enclose one bar's icons (and,
/// if `has_labels`, their number labels too) as one group, for a single
/// surrounding border. `start_col`/`n` are the bar's own column range,
/// `row` its row, on ABILITY_BAR_CONSOLE - the out-of-combat Ability Bar
/// centers itself (see ability_bar_start_col) while the Battle Bar sits
/// explicitly to its right, so this takes the range directly rather than
/// recomputing it. `row` was hardcoded to `ability_bar_row()` internally
/// until the player-status frame's portrait icon (a DIFFERENT row/col on
/// this same console - see HEALTH_FRAME_ICON_COL/ROW) needed this same
/// bounding-box math too, 2026-09-14.
///
/// No nominal-cell padding beyond the rounding direction itself anymore
/// (removed 2026-09-14, see each edge's own comment below) - past
/// rounds added an extra cell of buffer on every edge specifically to
/// counteract `render_helpers::pixel_box_tiles`'s own center-shift bug
/// (a tile's true rendered position used to drift toward the box's
/// interior on some edges, away from it on others - see that function's
/// own doc comment for the full story), which read as the border
/// crowding the icons on one side and leaving too much padding on the
/// other. Now that the shift is fixed at its real source, those extra
/// buffers were pure redundant padding ("too much padding around the
/// edges," confirmed live) - removed, keeping only the ROUNDING
/// DIRECTION itself (each edge still needs to round toward whichever
/// side keeps its own real clearance, not literal zero gap).
///
/// The edges past which the box GROWS AWAY from the icons (bottom, right)
/// still need more care than a plain truncating division, on both axes:
/// ABILITY_BAR_CONSOLE (where the icons actually live) renders ABOVE
/// HUD_CONSOLE (where this box is drawn) in z-order, so any HUD_CONSOLE
/// row/column whose PIXELS overlap the icon's own pixel range gets
/// visually painted over by the icon, border or not. A plain truncating
/// division from a pixel position into HUD_CONSOLE cell units can land a
/// cell whose pixels still reach into that overlap. Both the bottom and
/// right edges round UP (ceiling division), so the chosen row/column's
/// pixels start at or after the icon's true bottom/right edge; the top
/// and left edges' plain truncating division already rounds down/away
/// from the icon (safe on those sides, since the box is growing AWAY
/// from the icon there), so they need no ceiling correction at all.
fn ability_bar_box_bounds(
    start_col: i32,
    row: i32,
    n: i32,
    has_labels: bool,
    extra_side_pad: i32,
) -> (i32, i32, i32, i32) {
    let bar_row = row;

    let icons_left_px = start_col * (1280 / ABILITY_BAR_COLS);
    let icons_right_px = (start_col + n) * (1280 / ABILITY_BAR_COLS);
    let icons_top_px = bar_row * (800 / ABILITY_BAR_ROWS);
    let icons_bottom_px = (bar_row + 1) * (800 / ABILITY_BAR_ROWS);

    // No extra "- 1" beyond the plain floor division and `extra_side_pad`
    // anymore - removed 2026-09-14 once `render_helpers::pixel_box_tiles`'s
    // own center-shift fix (see its doc comment) was in place. That extra
    // column used to double as an accidental compensation for the
    // border's real rendered position silently drifting toward the icon
    // on this exact edge - now that the drift itself is fixed at the
    // source, the plain floor division alone (which already rounds AWAY
    // from the icon, toward smaller columns) is real, sufficient
    // clearance on its own; the extra column was reading as pure excess
    // padding ("too much padding around the edges," confirmed live).
    // `extra_side_pad` (HUD_CONSOLE columns, each side) is a deliberate
    // per-bar WIDTH choice on top of that minimum, not more clearance-
    // math compensation - see BATTLE_BOX_EXTRA_SIDE_PAD/ABILITY_BOX_EXTRA_SIDE_PAD's own doc comment.
    let left = icons_left_px * HUD_COLS / 1280 - extra_side_pad;
    // Ceiling division (the "+ 1279" trick) still needed - plain
    // truncating division here rounds the right edge DOWN, i.e. toward
    // the icon's own pixels rather than past them, which is genuinely
    // insufficient clearance (confirmed live, pre-dating the center-
    // shift fix). No longer a further "+1" on top of that though -
    // removed for the same reason as the left edge's own extra column
    // above: it was compensating for the center-shift drift, which no
    // longer exists.
    let right = (icons_right_px * HUD_COLS + 1279) / 1280 + extra_side_pad;
    // Ceiling division (the "+ 799" trick) - see the right edge's own
    // doc comment for why ceiling, not truncating, division. No extra
    // buffer beyond that (removed for bottom clearance 2026-09-14, then
    // again implicitly here since the center-shift fix removed the
    // remaining need for any of these edges to over-compensate).
    let bottom = (icons_bottom_px * HUD_ROWS + 799) / 800;

    let top = if has_labels {
        let (_, label_row) = ability_bar_label_position(start_col);
        // Two rows above the label, not one - `label_row - 1` (the
        // value this used to be) put the border on the row directly
        // ADJACENT to the label with zero blank rows between them,
        // which is the minimum needed to keep the border from sitting
        // ON the label's own row (the same failure mode print_box's own
        // title nudge hit earlier) but isn't actually any real padding
        // above it - direct feedback 2026-09-15 that the hotkey numbers
        // read as having no top clearance at all. `- 2` leaves one
        // genuine blank row between the border and the label text.
        label_row - 2
    } else {
        // One row above the icon's own top edge - the same "away from
        // the icon, toward smaller rows" floor-rounding the left edge
        // relies on above needs no ceiling correction here, since
        // rounding down already moves AWAY from an icon whose reference
        // edge is below this one. No extra "breathing room" row beyond
        // this minimum anymore, for the same center-shift reasoning as
        // every other edge in this function.
        (icons_top_px * HUD_ROWS / 800) - 1
    };

    (left, top, right - left, bottom - top)
}


#[system]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(ShopStock)]
#[read_component(Price)]
#[read_component(Point)]
#[read_component(Gold)]
#[read_component(Class)]
#[read_component(BattleItem)]
#[read_component(Render)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(IceArmored)]
pub fn hud(
    ecs: &SubWorld,
    #[resource] ability_bar_mouse_pos: &AbilityBarMousePos,
    #[resource] shopping: &Option<ShoppingActive>,
    #[resource] arena_run: &Option<ArenaRun>,
    #[resource] shop_message: &Option<ShopMessage>,
    #[resource] camera: &Camera,
) {
    let mut health_query = <&Health>::query().filter(component::<Player>());
    let player_health = health_query.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(HUD_CONSOLE);
    // The general how-to-play hint that used to sit here permanently
    // moved to the Paused screen's rotating tips (see screens/pause.rs's
    // PAUSE_HINTS) - this row is now reserved for genuinely situational
    // messages (a failed/prompted shop purchase) rather than always
    // showing something.
    if let Some(ShopMessage(text)) = shop_message {
        draw_batch.print_color_centered(1, text, ColorPair::new(RED, BLACK));
    } else if shopping.is_some() {
        draw_batch.print_centered(1, "Stand next to an item, press ENTER to buy it.");
    }
    // Compact player-status frame, offset one icon-size down and right
    // from the corner: a class-portrait icon (drawn further down on
    // ABILITY_BAR_CONSOLE, see HEALTH_FRAME_ICON_COL/ROW) next to a health
    // bar. HEALTH_BAR_START_COL/ROW line the bar up with the icon;
    // HEALTH_BAR_WIDTH deliberately stops well short of HUD_COLS's full
    // width so it doesn't run into the "Explore the Dungeon..." hint text
    // centered on this same row range - first-pass pixel guesses (see
    // CLAUDE.md's bracket-lib layout gotcha) pending another screenshot.
    //
    // The old plain `bar_horizontal` block-glyph bar (no real art, just
    // tinted CP437 blocks) replaced 2026-09-14 with the same real
    // PixelLab bar the battle screen's player HP bar already uses
    // (`render_helpers::draw_pixel_bar`, `battle_bar_frame.png`) -
    // BATTLE_BAR_CONSOLE/BATTLE_BAR_TEXT_CONSOLE are global consoles
    // (registered once in main.rs, not screen-specific), and the dungeon
    // HUD and the battle screen never draw in the same frame, so reusing
    // them here is safe - no ATB bar alongside it, since ATB doesn't
    // exist outside battle.
    // BAR_TEXT_VERTICAL_CENTER_SHIFT (render_helpers.rs) - direct
    // feedback 2026-09-15 that the overlaid "current / max" text (below,
    // on a whole HUD_CONSOLE row it can't move off of without also
    // moving the number away from where it visually belongs) sat too
    // high in the bar rather than centered. See that constant's own doc
    // comment for the real math behind it - the same fix `screens/
    // battle.rs`'s player HP bar needed too, once it was that bar's turn.
    let mut health_bar_batch = DrawBatch::new();
    health_bar_batch.target(BATTLE_BAR_CONSOLE);
    draw_pixel_bar(
        &mut health_bar_batch,
        HEALTH_BAR_START_COL,
        HEALTH_BAR_START_ROW as f32 + BAR_TEXT_VERTICAL_CENTER_SHIFT,
        HEALTH_BAR_WIDTH,
        player_health.current,
        player_health.max,
        RED,
    );
    health_bar_batch.submit(0).expect("Batch error");

    let mut health_bar_text_batch = DrawBatch::new();
    health_bar_text_batch.target(BATTLE_BAR_TEXT_CONSOLE);
    health_bar_text_batch.print_color(
        Point::new(HEALTH_BAR_START_COL + 2, HEALTH_BAR_START_ROW),
        format!("{} / {}", player_health.current.max(0), player_health.max),
        ColorPair::new(WHITE, BLACK),
    );
    health_bar_text_batch.submit(0).expect("Batch error");

    let (player, map_level) = <(Entity, &Player)>::query() // (1)
        .iter(ecs)
        .find_map(|(entity, player)| Some((*entity, player.map_level)))
        .unwrap();

    // Battle Arena shows Gold in this corner instead of Dungeon Level -
    // map_level is a dungeon-crawl-only concept Arena code never touches
    // (it stays stuck at whatever it was spawned with), so showing it
    // during an Arena run would just be a stale, meaningless number.
    // Dungeon Crawl now shows Gold here too, but only while actually
    // browsing its own between-floor shop (shopping.is_some()) - that's
    // the one moment Gold matters more than which floor you're on;
    // everywhere else in Dungeon Crawl this stays Dungeon Level, unlike
    // Arena where it's Gold for the entire run.
    if arena_run.is_some() || shopping.is_some() {
        let gold = <&Gold>::query()
            .filter(component::<Player>())
            .iter(ecs)
            .nth(0)
            .map_or(0, |g| g.0);
        draw_batch.print_color_right(
            Point::new(HUD_COLS, 1),
            format!("Gold: {}", gold),
            ColorPair::new(YELLOW, BLACK),
        );
    } else {
        draw_batch.print_color_right(
            // (2)
            Point::new(HUD_COLS, 1),
            format!("Dungeon Level: {}", map_level + 1), // (3)
            ColorPair::new(YELLOW, BLACK),
        );
    }

    // The dungeon-exploration screen is deliberately bare besides the
    // above (health/level/gold) and the Ability Bar further down -
    // Battle Attacks and Weapons used to have their own panels here too,
    // but both moved to the Item Menu's read-only reference panel (press
    // M - see screens/item_menu.rs) so this screen stays clean and the
    // Ability Bar is the only thing drawing attention lower down.
    //
    // Shop stock used to list EVERY item in the shop as a fixed top-left
    // text block (rows 2+), which collided with the player-status frame
    // above once that frame moved into this same corner - replaced with a
    // single tooltip for whichever item is actually adjacent right now
    // (components::shop_item_near - the exact same lookup buy_nearby_item
    // uses, so this always shows what pressing Enter would actually buy),
    // anchored near the PLAYER's own position rather than a fixed screen
    // spot, so it travels with them along the counter.
    let player_class = entity_class(ecs, player);
    // Same map-pos -> screen-space -> HUD_CONSOLE-cell conversion the shop
    // tooltip below already uses - the player's real position, in the same
    // cell units ability_bar_box_bounds' own boxes are in, for backlog
    // item 9's "is the player under this bar" check (see
    // BAR_FADED_FILL_ALPHA's own doc comment). Computed once here rather
    // than separately per bar - all 3 dungeon bars share the same player
    // position this frame.
    let player_hud = ecs
        .entry_ref(player)
        .ok()
        .and_then(|entry| entry.get_component::<Point>().ok().copied())
        .map(|player_pos| mouse_to_hud(player_pos - Point::new(camera.left_x, camera.top_y)));
    if shopping.is_some() {
        if let Some(player_pos) = ecs
            .entry_ref(player)
            .ok()
            .and_then(|entry| entry.get_component::<Point>().ok().copied())
        {
            if let Some((_, count, name, price)) = shop_item_near(ecs, player_pos) {
                // player_pos is a map coordinate - subtract the camera's
                // own top-left to land in console 0's screen-space (32px
                // tiles, same conversion entity_render.rs uses to place
                // the player's own glyph), then mouse_to_hud to reach
                // HUD_CONSOLE's finer grid, same as tooltips.rs already
                // does for the mouse-hover tooltip.
                let player_screen =
                    player_pos - Point::new(camera.left_x, camera.top_y);
                let player_hud = mouse_to_hud(player_screen);

                let text = format!("{} x{} - {}g", name, count, price);
                let box_y = player_hud.y + SHOP_TOOLTIP_ROW_OFFSET;
                let box_x = (player_hud.x - SHOP_TOOLTIP_WIDTH / 2).max(0);

                // The real PixelLab panel border (item 10 in docs/
                // ideas.md), same as every other border on the dungeon
                // screen now. Converted to render_helpers::PanelBox
                // 2026-09-14 alongside its wider rollout - PanelBox's own
                // auto-assigned z_order (see its doc comment in render_
                // helpers.rs) replaces the manually-picked 10008/10009
                // this block used to need to stay clear of the out-of-
                // combat bar block below's own 10006/10007 (two batches
                // sharing a z_order on the same console is an ambiguous
                // draw order) - that whole class of bug is now closed off
                // by construction, not just avoided by careful numbering.
                // Swamp, not Dungeon - direct request 2026-09-14 ("I want
                // the tooltips to be the wooden and green corners").
                // PIXEL_BOX_TILE_SCALE_COMPACT, not the default scale -
                // this box is only 3 HUD_CONSOLE rows tall, short enough
                // that the default scale's border rows alone were ~50%
                // of its total height (see that constant's own doc
                // comment for the real numbers).
                let mut shop_box = PanelBox::new(
                    box_x,
                    box_y,
                    SHOP_TOOLTIP_WIDTH,
                    SHOP_TOOLTIP_HEIGHT,
                    UiPanelTheme::Swamp,
                    PIXEL_BOX_TILE_SCALE_COMPACT,
                );
                shop_box.text_color(0, 0, GREEN, BLACK, text);
                shop_box.submit();
            }
        }
    }

    draw_batch.submit(10000).expect("Batch error");

    // The Ability Bar - a row of icons along the bottom of the dungeon
    // view, one per out-of-combat ability the current class could ever
    // have (components::ability_bar_slots), greyed out when not
    // currently carried. Its own DrawBatch/console (ABILITY_BAR_CONSOLE)
    // since it uses the coarse big-glyph dungeonfont grid, not
    // HUD_CONSOLE's fine text one - see that console's own doc comment
    // in main.rs for the exact cell geometry. Submitted as its own batch,
    // targeting a HIGHER z-order than the panel batch above, so the bar
    // reliably paints over the dungeon view beneath it regardless of
    // draw order here.
    if let Some(class) = &player_class {
        let effect_roster = class_effect_names(class);
        let technique_roster = class_technique_names(class);
        let bar_mouse = ability_bar_mouse_pos.0;
        let bar_row = ability_bar_row();

        // ABILITY_BAR_ICON_CONSOLE/ABILITY_BAR_ICON_BADGE_CONSOLE, NOT
        // ABILITY_BAR_CONSOLE/ABILITY_BAR_BADGE_CONSOLE directly - see
        // ABILITY_BAR_ICON_CONSOLE's own doc comment in main.rs. Once the
        // fill below moved onto UI_PANEL_CONSOLE (registered AFTER
        // ABILITY_BAR_CONSOLE) for pixel-perfect alignment with the
        // border, its fully-opaque no-discard quad started painting
        // directly over any icon/badge drawn on the OLD earlier consoles
        // - confirmed live 2026-09-14 (every bar rendered as a solid
        // black box, no icons visible at all). These two later-registered
        // duplicates of the exact same grid/font config paint AFTER the
        // fill instead, so the icons/badges are visible again.
        let mut bar_batch = DrawBatch::new();
        bar_batch.target(ABILITY_BAR_ICON_CONSOLE);
        let mut badge_batch = DrawBatch::new();
        badge_batch.target(ABILITY_BAR_ICON_BADGE_CONSOLE);
        let mut portrait_batch = DrawBatch::new();
        // CHARACTER_PORTRAIT_HUD_ICON_CONSOLE, NOT CHARACTER_PORTRAIT_
        // HUD_CONSOLE - see that console's own doc comment in main.rs.
        // Registered well before UI_PANEL_CONSOLE, so the player-status
        // frame's new border (below) painted its opaque fill directly
        // over the portrait once that border existed - confirmed live
        // 2026-09-14, same failure mode ABILITY_BAR_ICON_CONSOLE was
        // already built to dodge for the dungeonfont-sourced icons.
        portrait_batch.target(CHARACTER_PORTRAIT_HUD_ICON_CONSOLE);
        // The real PixelLab panel border (item 10 in docs/ideas.md) for
        // the Item/Ability/Battle Bar frames - same UI_PANEL_CONSOLE/
        // draw_filled_pixel_box recipe already proven on the Item Menu -
        // now render_helpers::PanelBox instead, one instance per bar
        // (Item/Ability/Battle), converted 2026-09-14 alongside its
        // wider rollout. None of the three bars print any title text of
        // their own (the ability key labels below are unrelated,
        // absolute-positioned - see text_batch's own comment), so each
        // is just `PanelBox::new(...).submit()` where the fill used to
        // go - no shared panel_batch needed anymore for these three.
        //
        // Number-key labels (1-9/0) print on top of the Ability Bar's own
        // black fill - same same-console overwrite bug the Item Menu had
        // (see item_menu.rs's print_box and PANEL_TEXT_CONSOLE's own doc
        // comment in main.rs) would apply if these were printed straight
        // onto a SimpleConsole sharing the fill's own cells; PANEL_TEXT_
        // CONSOLE is registered later in z-order than everything else
        // this function draws to, so its own text is never at risk of
        // this regardless.
        let mut text_batch = DrawBatch::new();
        text_batch.target(PANEL_TEXT_CONSOLE);
        // (name, description-lookup key, box_y for the tooltip anchor) -
        // whichever bar's icon the mouse is currently over, checked
        // across BOTH bars so hovering either one shows its description.
        let mut hovered: Option<(String, i32)> = None;

        // Player-status frame's class-portrait icon (top-left corner,
        // next to the health bar drawn earlier on HUD_CONSOLE - see
        // HEALTH_FRAME_ICON_COL/ROW). resources/character_portrait.png's
        // still pose replaces the old "just draw the player's own
        // dungeonfont Render glyph bigger" behavior for any class with a
        // row there - see components::character_portrait_glyph - falling
        // back to the old behavior for anything without one yet.
        match character_portrait_glyph(class) {
            Some(glyph) => draw_portrait(
                &mut portrait_batch,
                HEALTH_FRAME_ICON_COL,
                HEALTH_FRAME_ICON_ROW,
                Render {
                    color: ColorPair::new(WHITE, BLACK),
                    glyph,
                },
            ),
            None => {
                if let Some(player_render) = ecs
                    .entry_ref(player)
                    .ok()
                    .and_then(|entry| entry.get_component::<Render>().ok().copied())
                {
                    draw_portrait(
                        &mut bar_batch,
                        HEALTH_FRAME_ICON_COL,
                        HEALTH_FRAME_ICON_ROW,
                        player_render,
                    );
                }
            }
        }
        // Border around the portrait, direct request 2026-09-14 - reuses
        // ability_bar_box_bounds' own icon-footprint math (n=1, no
        // labels, no extra pad - a snug fit, not a wider box like the
        // Ability/Battle Bars get) now that it takes an explicit `row`
        // instead of always `ability_bar_row()`. Swamp, matching every
        // other border already on this HUD (the shop tooltip, all 3
        // dungeon bars).
        let (icon_box_x, icon_box_y, icon_box_w, icon_box_h) = ability_bar_box_bounds(
            HEALTH_FRAME_ICON_COL,
            HEALTH_FRAME_ICON_ROW,
            1,
            false,
            0,
        );
        PanelBox::new(
            icon_box_x,
            icon_box_y,
            icon_box_w,
            icon_box_h,
            UiPanelTheme::Swamp,
            PIXEL_BOX_TILE_SCALE_COMPACT,
        )
        .submit();

        // Buff badges - the abilities' own real sprite icons, just below
        // the class portrait, for any lasting effect currently on the
        // player. See BUFF_BADGE_CONSOLE's own doc comment in main.rs
        // for why this needs its own console rather than reusing
        // ABILITY_BAR_CONSOLE or HUD_CONSOLE.
        let has_invisible = ecs
            .entry_ref(player)
            .ok()
            .and_then(|entry| entry.get_component::<Invisible>().ok().copied())
            .is_some();
        let has_stealth = ecs
            .entry_ref(player)
            .ok()
            .and_then(|entry| entry.get_component::<Stealthed>().ok().copied())
            .is_some();
        let has_ice_armor = ecs
            .entry_ref(player)
            .ok()
            .and_then(|entry| entry.get_component::<IceArmored>().ok().copied())
            .is_some();
        let mut buff_batch = DrawBatch::new();
        buff_batch.target(BUFF_BADGE_CONSOLE);
        let mut buff_badge_col = BUFF_BADGE_COL_START;
        draw_buff_badge(&mut buff_batch, &mut buff_badge_col, has_invisible, "Invisible Cloak");
        draw_buff_badge(&mut buff_batch, &mut buff_badge_col, has_stealth, "Stealth");
        draw_buff_badge(&mut buff_batch, &mut buff_badge_col, has_ice_armor, "Ice Armor");
        buff_batch.submit(10003).expect("Batch error");

        // Out-of-combat Ability Bar - centered as a group, with number
        // labels (1-9, then 0) since these ARE directly usable via
        // player_input.rs::use_ability, and a RED box.
        let ability_slots = ability_bar_slots(ecs, player, class, &effect_roster);
        let ability_n = (ability_slots.len() as i32).min(ABILITY_BAR_MAX_SLOTS as i32);
        let ability_start_col = ability_bar_start_col(ability_n);

        // Item Bar - universal consumables (Healing Potion, Dungeon Map,
        // any future item every class can carry), click-to-use rather
        // than a number key (see player_input.rs's use_item_bar_click) -
        // 1-9/0 are already fully claimed by class abilities, and this
        // keeps the bar usable without adding any new keybinds. No number
        // labels for the same reason the Battle Bar has none, and a BLUE
        // box - a third color distinct from Ability's RED and Battle's
        // GREEN. Sits immediately to the LEFT of the Ability Bar - see
        // item_bar_start_col.
        let item_roster = universal_item_names();
        let item_slots = item_bar_slots(ecs, player, &item_roster);
        let item_n = (item_slots.len() as i32).min(ABILITY_BAR_MAX_SLOTS as i32);
        let item_start_col = item_bar_start_col(ability_start_col, item_n);

        // Box bounds + fade state computed BEFORE the icon loop now
        // (backlog item 9, widened 2026-09-15 to also fade the icons
        // themselves, not just the box) - both the icons and the box
        // itself need the same alpha, so this can't wait until after the
        // loop the way the box-only version did. ITEM_BOX_EXTRA_RIGHT_PAD/
        // BOTTOM_PAD - direct feedback 2026-09-15 that this specific
        // box's own right and bottom edges (the icons' own footprint
        // doesn't reach either one, unlike the left/top - see
        // ability_bar_box_bounds' own doc comment) sit too close to the
        // icons compared to a genuine margin. Deliberately scoped to
        // ONLY this box's own bounds - ability_bar_box_bounds itself,
        // and the Ability/Battle Bars that also call it, are untouched.
        let (item_box_x, item_box_y, item_box_w0, item_box_h0) =
            ability_bar_box_bounds(item_start_col, ability_bar_row(), item_n, false, 0);
        let item_box_w = item_box_w0 + ITEM_BOX_EXTRA_RIGHT_PAD;
        let item_box_h = item_box_h0 + ITEM_BOX_EXTRA_BOTTOM_PAD;
        let item_fade_alpha = if player_under_box(player_hud, item_box_x, item_box_y, item_box_w, item_box_h)
        {
            BAR_FADE_ALPHA
        } else {
            1.0
        };

        for (i, slot) in item_slots.iter().enumerate().take(item_n as usize) {
            let col = item_start_col + i as i32;
            let owned = slot.owned.is_some();
            let glyph = glyph_for_item_name(&slot.name).unwrap_or('?');
            draw_portrait(
                &mut bar_batch,
                col,
                bar_row,
                Render { color: bar_icon_color(owned, item_fade_alpha), glyph: to_cp437(glyph) },
            );
            if bar_mouse.y == bar_row && bar_mouse.x == col {
                hovered = Some((slot.name.clone(), item_box_y));
            }
            if let Some((count, _)) = slot.owned {
                draw_stack_count_badge(&mut badge_batch, col, bar_row, count);
            }
        }
        if item_n > 0 {
            // Swamp, not Dungeon - direct request 2026-09-14, so all 3
            // dungeon HUD bars match (the other two already were/are).
            // PIXEL_BOX_TILE_SCALE_COMPACT, not the default scale - this
            // box can be as narrow as ONE icon wide, where the default
            // scale's two border columns alone were ~29% of the box's
            // total width (see that constant's own doc comment for the
            // real numbers behind this, computed from ability_bar_box_
            // bounds' own real geometry, not eyeballed).
            //
            // Fades (BAR_FADE_ALPHA) when the player's own real map
            // position is underneath this specific box - backlog item 9,
            // 2026-09-15, widened the same day to also fade the border
            // and icons (see BAR_FADE_ALPHA's own doc comment) rather
            // than just this box's own fill.
            PanelBox::new_faded(
                item_box_x,
                item_box_y,
                item_box_w,
                item_box_h,
                UiPanelTheme::Swamp,
                PIXEL_BOX_TILE_SCALE_COMPACT,
                item_fade_alpha,
            )
            .submit();
        }

        // Box bounds + fade state computed BEFORE the icon loop - see the
        // Item Bar's own comment above (backlog item 9).
        let (ability_box_x, ability_box_y, ability_box_w, ability_box_h0) = ability_bar_box_bounds(
            ability_start_col,
            ability_bar_row(),
            ability_n,
            true,
            ABILITY_BOX_EXTRA_SIDE_PAD,
        );
        // ABILITY_BOX_EXTRA_BOTTOM_PAD - same "no bottom-padding
        // parameter exists at all" gap the Item Bar had, direct feedback
        // 2026-09-15. Scoped to only this box's own bounds.
        let ability_box_h = ability_box_h0 + ABILITY_BOX_EXTRA_BOTTOM_PAD;
        let ability_fade_alpha = if player_under_box(
            player_hud,
            ability_box_x,
            ability_box_y,
            ability_box_w,
            ability_box_h,
        ) {
            BAR_FADE_ALPHA
        } else {
            1.0
        };

        for (i, slot) in ability_slots.iter().enumerate().take(ability_n as usize) {
            let col = ability_start_col + i as i32;
            let owned = slot.owned.is_some();
            let glyph = glyph_for_item_name(&slot.name).unwrap_or('?');
            draw_portrait(
                &mut bar_batch,
                col,
                bar_row,
                Render { color: bar_icon_color(owned, ability_fade_alpha), glyph: to_cp437(glyph) },
            );
            if bar_mouse.y == bar_row && bar_mouse.x == col {
                hovered = Some((slot.name.clone(), ability_box_y));
            }
            if let Some((count, _)) = slot.owned {
                draw_stack_count_badge(&mut badge_batch, col, bar_row, count);
            }

            let (label_col, label_row) = ability_bar_label_position(col);
            text_batch.print_color(
                Point::new(label_col, label_row),
                ability_bar_key_label(i),
                ColorPair::new(if owned { YELLOW } else { GRAY }, BLACK),
            );
        }
        if ability_n > 0 {
            // Swamp, not Dungeon - see the Item Bar's own comment above.
            // PIXEL_BOX_TILE_SCALE_COMPACT too - same reasoning. Fades
            // (BAR_FADE_ALPHA) when the player is underneath, including
            // the border/icons now - see BAR_FADE_ALPHA's own doc
            // comment.
            PanelBox::new_faded(
                ability_box_x,
                ability_box_y,
                ability_box_w,
                ability_box_h,
                UiPanelTheme::Swamp,
                PIXEL_BOX_TILE_SCALE_COMPACT,
                ability_fade_alpha,
            )
            .submit();
        }

        // Battle Bar - the class's in-battle Techniques, read-only
        // reference only (not usable outside a fight at all - see
        // battle::apply_player_technique/screens/battle.rs, which is the
        // only place a Technique ever actually resolves), so no number
        // labels and a GREEN box instead of red to visually tell the two
        // bars apart at a glance. Sits immediately to the right of the
        // Ability Bar - see battle_bar_start_col.
        let battle_slots = battle_bar_slots(ecs, player, &technique_roster);
        let battle_n = (battle_slots.len() as i32).min(ABILITY_BAR_MAX_SLOTS as i32);
        let battle_start_col = battle_bar_start_col(ability_start_col, ability_n);

        // Box bounds + fade state computed BEFORE the icon loop - see
        // the Item Bar's own comment above (backlog item 9).
        let (battle_box_x, battle_box_y, battle_box_w, battle_box_h0) = ability_bar_box_bounds(
            battle_start_col,
            ability_bar_row(),
            battle_n,
            false,
            BATTLE_BOX_EXTRA_SIDE_PAD,
        );
        let battle_box_h = battle_box_h0 + BATTLE_BOX_EXTRA_BOTTOM_PAD;
        let battle_fade_alpha = if player_under_box(
            player_hud,
            battle_box_x,
            battle_box_y,
            battle_box_w,
            battle_box_h,
        ) {
            BAR_FADE_ALPHA
        } else {
            1.0
        };

        for (i, slot) in battle_slots.iter().enumerate().take(battle_n as usize) {
            let col = battle_start_col + i as i32;
            let owned = slot.owned.is_some();
            let glyph = glyph_for_item_name(&slot.name).unwrap_or('?');
            draw_portrait(
                &mut bar_batch,
                col,
                bar_row,
                Render { color: bar_icon_color(owned, battle_fade_alpha), glyph: to_cp437(glyph) },
            );
            if bar_mouse.y == bar_row && bar_mouse.x == col {
                hovered = Some((slot.name.clone(), battle_box_y));
            }
            if let Some((count, _)) = slot.owned {
                draw_stack_count_badge(&mut badge_batch, col, bar_row, count);
            }
        }
        if battle_n > 0 {
            // Swamp - the first of the 3 dungeon HUD bars to switch
            // (2026-09-14); the Item and Ability Bars above joined it
            // the same day once asked for all 3 to match.
            // PIXEL_BOX_TILE_SCALE_COMPACT too - see the Item Bar's own
            // comment above. Fades (BAR_FADE_ALPHA) when the player is
            // underneath, including the border/icons now - see BAR_FADE_
            // ALPHA's own doc comment.
            PanelBox::new_faded(
                battle_box_x,
                battle_box_y,
                battle_box_w,
                battle_box_h,
                UiPanelTheme::Swamp,
                PIXEL_BOX_TILE_SCALE_COMPACT,
                battle_fade_alpha,
            )
            .submit();
        }

        bar_batch.submit(10001).expect("Batch error");
        badge_batch.submit(10004).expect("Batch error");
        portrait_batch.submit(10005).expect("Batch error");
        text_batch.submit(10007).expect("Batch error");

        // The hovered slot's tooltip (from either bar) - drawn in a real
        // render_helpers::PanelBox (Swamp, matching every other border
        // on this HUD) since 2026-09-14, was bare centered text before.
        // Text still centered PER LINE via `text_color_centered_raw`
        // (same reasoning as before: a full sentence of prose is far
        // more sensitive to being a few columns off than a single digit
        // is), so the box itself has to be horizontally centered on
        // HUD_CONSOLE's own width too, for its border to actually line
        // up with text centered on that same width (see `text_color_
        // centered`'s own doc comment in render_helpers.rs).
        //
        // Snug on both top AND bottom (RAW, not the standard inset) -
        // direct feedback 2026-09-15 that the box read as sitting
        // "right on top of" whichever bar it describes, with no real
        // gap between the two boxes' borders, AND that the text itself
        // had unwanted padding above it. Fixed both at once: text now
        // starts immediately at `dy = 1` (the box's own first real
        // interior row, right below the border, matching the bottom
        // edge's existing zero-padding convention), and TOOLTIP_GAP
        // reserves one genuinely blank row between this box's own
        // bottom border and the hovered bar's own top border (`box_y`)
        // - previously 0 (the two borders landed on the exact same
        // row), not a buffer at all.
        if let Some((name, box_y)) = hovered {
            let description =
                description_for_item_name(&name).unwrap_or_else(|| "No description.".to_string());
            // Wrapped across multiple lines rather than one long line -
            // some real descriptions (Freeze Trap's, for one) are long
            // enough to run off both edges of the screen otherwise.
            let lines = wrap_text(&format!("{}: {}", name, description), 70);
            const TOOLTIP_WIDTH: i32 = 74;
            const TOOLTIP_GAP: i32 = 1;
            let tooltip_x = (HUD_COLS - TOOLTIP_WIDTH) / 2;
            let tooltip_height = lines.len() as i32 + 2;
            let tooltip_y = box_y - TOOLTIP_GAP - tooltip_height;
            let mut tooltip_box = PanelBox::new(
                tooltip_x,
                tooltip_y,
                TOOLTIP_WIDTH,
                tooltip_height,
                UiPanelTheme::Swamp,
                PIXEL_BOX_TILE_SCALE_COMPACT,
            );
            for (i, line) in lines.iter().enumerate() {
                tooltip_box.text_color_centered_raw(1 + i as i32, WHITE, BLACK, line.clone());
            }
            tooltip_box.submit();
        }
    }
}

#[cfg(test)]
mod hud_system_execution_tests {
    use super::*;

    /// Actually EXECUTES the real hud system through a real Schedule -
    /// not just a type-check. This is the only way to catch a legion
    /// component-access declaration mismatch: querying a component type
    /// inside a #[system] function that isn't also listed in that
    /// function's own #[read_component]/#[write_component] attributes
    /// compiles just fine (the type exists, the query is well-formed)
    /// but panics at RUNTIME with `AccessDenied` the moment the system
    /// actually executes and legion checks its declared permissions.
    /// This exact bug shipped once already: components::
    /// battle_items_carried queries BattleItem, but this system's own
    /// #[read_component] list didn't include it after the Battle Bar
    /// was added, since BattleItem had been removed from the list in an
    /// earlier round (when the old Battle Attacks panel was deleted)
    /// and never re-added when a NEW BattleItem query was introduced
    /// later. `cargo check`/`cargo build` both stayed clean throughout -
    /// only running the system for real surfaces this class of bug.
    /// Also exercises the shop-tooltip path (components::shop_item_near,
    /// added alongside the player-status frame) - Shopping is Some, the
    /// player has a real Point, and a ShopStock+Name+Price+Point entity
    /// sits adjacent to it, so this actually runs the query that
    /// replaced the old fixed shop item list, not just an empty/never-
    /// taken branch.
    #[test]
    fn hud_system_runs_without_a_component_access_panic() {
        let mut world = World::default();
        let player = world.push((
            Health {
                current: 10,
                max: 10,
            },
            Player { map_level: 0 },
            Class("Barbarian".to_string()),
            Point::new(5, 5),
            // Exercises the buff-badge path (Invisible/Stealthed/
            // IceArmored) for real, not just an empty/never-taken branch.
            Invisible { moves_remaining: 3 },
        ));
        // A real BattleItem-tagged entity - exercises the exact query
        // path that previously panicked, not just an empty/never-taken
        // branch.
        world.push((
            Item,
            BattleItem,
            Carried(player),
            Name("Deathblow".to_string()),
        ));
        // Adjacent (one tile above the player) - exercises shop_item_near.
        world.push((
            ShopStock(3),
            Point::new(5, 4),
            Name("Healing Potion".to_string()),
            Price(5),
        ));

        let mut resources = Resources::default();
        resources.insert(AbilityBarMousePos(Point::zero()));
        resources.insert(Some(ShoppingActive));
        resources.insert(None::<ArenaRun>);
        resources.insert(None::<ShopMessage>);
        resources.insert(Camera::new(Point::zero()));

        let mut schedule = Schedule::builder().add_system(hud_system()).build();

        // No assertion needed beyond "this doesn't panic" - that IS the
        // regression this test exists to catch.
        schedule.execute(&mut world, &mut resources);
    }
}
