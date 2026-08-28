use crate::prelude::*;

/// Below this fraction of max HP, the player's dungeon-view glyph tints
/// red - see tinted_color.
const LOW_HEALTH_THRESHOLD: f32 = 0.3;

/// Empirically-confirmed set_fancy quirk: a glyph placed via set_fancy
/// at the same (x, y) that places it correctly via a plain console's
/// set() renders exactly one full cell too far north, consistently,
/// regardless of movement direction - see the fuller writeup that used
/// to live on this same constant name (still true, just no longer
/// specific to a single console), and the matching
/// WIGGLE_CONSOLE_Y_ANCHOR_OFFSET in render_helpers.rs /
/// MAP_SCROLL_Y_ANCHOR_OFFSET in map_render.rs (three independent
/// confirmations of the same bracket-lib behavior now). Used by both
/// fancy-console paths below - GLIDE_CONSOLE (one entity gliding while
/// the camera itself sits still) and ENTITY_SCROLL_CONSOLE (every
/// entity, while the camera itself is panning) are the same underlying
/// set_fancy draw, just fed different offsets - see
/// components::camera_render_offset for where those offsets come from.
const GLIDE_CONSOLE_Y_ANCHOR_OFFSET: f32 = 1.0;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Health)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(Frozen)]
#[read_component(MovingAnimation)]
pub fn entity_render(#[resource] camera: &Camera, ecs: &SubWorld) {
    let mut renderables = <(Entity, &Point, &Render)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    match camera_render_offset(ecs) {
        None => {
            // Camera at rest. A per-entity gliding_position check
            // decides whether THIS entity draws on the sub-pixel
            // GLIDE_CONSOLE (a monster taking its own step while the
            // player stands still - the only way to reach this branch
            // with anything actually gliding) or the plain integer
            // console 1 (everything else). The player is never
            // mid-glide whenever this branch runs - see
            // camera_render_offset's doc comment - so it always lands
            // in the plain-console case here, with no special-casing
            // needed for it.
            let offset = Point::new(camera.left_x, camera.top_y);
            let mut draw_batch = DrawBatch::new();
            draw_batch.target(1);
            let mut glide_batch = DrawBatch::new();
            glide_batch.target(GLIDE_CONSOLE);

            renderables
                .iter(ecs)
                .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
                .for_each(|(entity, pos, render)| {
                    let color = tinted_color(ecs, *entity, render.color);
                    match gliding_position(ecs, *entity) {
                        Some((fx, fy)) => draw_glyph_fancy(
                            &mut glide_batch,
                            fx - offset.x as f32,
                            fy - offset.y as f32,
                            color,
                            render.glyph,
                        ),
                        None => {
                            draw_batch.set(*pos - offset, color, render.glyph);
                        }
                    }
                });

            draw_batch.submit(5000).expect("Batch error");
            glide_batch.submit(5100).expect("Batch error");
        }
        Some((ox, oy)) => {
            // The camera itself is panning - the player is mid-glide
            // (see camera_render_offset). Every entity, moving or not,
            // now needs a fractional position derived from this same
            // (ox, oy), or a stationary one would stay snapped to its
            // old integer screen cell while the map slides underneath
            // it - see ENTITY_SCROLL_CONSOLE's doc comment in main.rs.
            //
            // This includes the player, which is no longer a special
            // case: substituting the player's own gliding_position into
            // "screen pos = world pos - (ox, oy)" lands on exactly the
            // screen center for every frame of its own glide, since
            // (ox, oy) is itself defined as "the player's eased
            // position minus screen center" (see camera_render_offset).
            // That identity is what actually fixes the old jump/
            // snap-back bug this file used to work around by excluding
            // the player from its own glide entirely - it isn't a
            // special case anymore, just the same formula every other
            // entity already used.
            let mut scroll_batch = DrawBatch::new();
            scroll_batch.target(ENTITY_SCROLL_CONSOLE);

            renderables
                .iter(ecs)
                .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
                .for_each(|(entity, pos, render)| {
                    let color = tinted_color(ecs, *entity, render.color);
                    let (fx, fy) =
                        gliding_position(ecs, *entity).unwrap_or((pos.x as f32, pos.y as f32));
                    draw_glyph_fancy(&mut scroll_batch, fx - ox, fy - oy, color, render.glyph);
                });

            scroll_batch.submit(5000).expect("Batch error");
        }
    }
}

/// Shared by both branches above: draws one glyph onto a fancy console
/// (already `.target()`ed by the caller) with a fully transparent
/// background and the Y-anchor correction applied - the same proven
/// trick END_SCREEN_FALLEN_CONSOLE's fallen-hero portrait needed,
/// without which a fancy console's normally-opaque background quad
/// would paint a visible box sliding over the map every time something
/// moved. `sx`/`sy` are already camera-relative (world position minus
/// whichever offset is active this frame) - callers do that subtraction
/// themselves, since the two branches above get their offset from
/// different places (a fixed integer Point vs. a fractional (f32, f32)
/// pair).
fn draw_glyph_fancy(
    batch: &mut DrawBatch,
    sx: f32,
    sy: f32,
    color: ColorPair,
    glyph: FontCharType,
) {
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(sx, sy + GLIDE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(1.0, 1.0),
        ColorPair::new(color.fg, bg_transparent),
        glyph,
    );
}

/// Overrides a dungeon-view entity's color for a few status indicators.
/// Frozen (Hunter's Freeze Trap) is checked first and applies to ANY
/// entity, not just the player - see components::Frozen. Everything
/// after that is player-only, as before: Invisible (stealth) takes
/// priority over low health, since a stealthed player being visually
/// flagged as "in danger" would undercut the point of being hidden.
fn tinted_color(ecs: &SubWorld, entity: Entity, base: ColorPair) -> ColorPair {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return base,
    };
    if entry.get_component::<Frozen>().is_ok() {
        return ColorPair::new(BLUE, BLACK);
    }
    if entry.get_component::<Player>().is_err() {
        return base;
    }
    if entry.get_component::<Invisible>().is_ok() || entry.get_component::<Stealthed>().is_ok() {
        return ColorPair::new(GRAY, BLACK);
    }
    if let Ok(health) = entry.get_component::<Health>() {
        if health.max > 0 && (health.current as f32 / health.max as f32) <= LOW_HEALTH_THRESHOLD {
            return ColorPair::new(RED, BLACK);
        }
    }
    base
}
