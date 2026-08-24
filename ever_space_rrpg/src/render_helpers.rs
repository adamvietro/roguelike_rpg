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
