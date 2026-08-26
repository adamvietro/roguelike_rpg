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

/// Draws `render`'s glyph with a small shake instead of the plain
/// `draw_portrait` above - only for the side currently mid-"Attacking"
/// flash (see attack_wiggle_offset). Returns true if it drew (onto
/// `batch`, which must already be targeting BATTLE_PORTRAIT_WIGGLE_CONSOLE)
/// - false if `flash` isn't an active Attacking flash, in which case the
/// caller should fall back to the plain draw_portrait on console 3
/// instead. Never both for the same portrait on the same frame.
pub fn draw_wiggling_portrait(
    batch: &mut DrawBatch,
    col: i32,
    row: i32,
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
        PointF::new(
            col as f32 + offset_x,
            row as f32 + WIGGLE_CONSOLE_Y_ANCHOR_OFFSET,
        ),
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
