use crate::prelude::*;

/// Below this fraction of max HP, the player's dungeon-view glyph tints
/// red - see tinted_color.
const LOW_HEALTH_THRESHOLD: f32 = 0.3;

/// Empirically-confirmed correction for GLIDE_CONSOLE's vertical
/// positioning: a glyph drawn via set_fancy at the same (x, y) that
/// places it correctly via the plain console's set() renders exactly one
/// full cell too far north, consistently, regardless of movement
/// direction, and without drifting further off over a longer glide -
/// confirmed by direct testing, not documentation (bracket-lib's source
/// isn't available to consult here). That signature - a constant,
/// direction-independent, non-accumulating one-cell error - points at
/// set_fancy anchoring a glyph's position from the bottom of its cell
/// rather than the top the way set() does, not at anything wrong in the
/// movement/lerp/camera math feeding it. Added to fy before it reaches
/// set_fancy to compensate. If a future bracket-lib upgrade changes this
/// anchoring behavior, this is the one place to adjust.
const GLIDE_CONSOLE_Y_ANCHOR_OFFSET: f32 = 1.0;

/// The player is excluded from the glide (see entity_render below) for a
/// reason specific to this camera design, not a rendering limitation:
/// Camera::on_player_move recenters left_x/top_y the instant a move is
/// processed, so the camera always keeps the player exactly at display
/// center once a move completes - the camera's whole job is to chase the
/// player. Animating the player's OWN glyph on top of a camera that's
/// simultaneously trying to keep that same glyph centered means the
/// glyph's start-of-glide screen position (still the OLD world point, now
/// read against the ALREADY-recentered camera) lands on the opposite side
/// of center from the direction just traveled - confirmed by measuring
/// actual rendered pixel positions frame-by-frame during a real move: the
/// glyph appeared one full cell off-center, opposite the direction of
/// travel, then eased back to center over the following few frames.
/// That's not a bug in the interpolation math - it's a structural
/// conflict between "this entity is being smoothly animated" and "the
/// camera is simultaneously locked onto this exact entity." Enemies have
/// no such conflict (the camera never centers on them), so they keep the
/// normal glide. Fixing this properly for the player too would mean
/// making the camera's own rendering offset scroll smoothly in lockstep
/// with the same eased position, which would require the entire map
/// (map_render.rs, currently a plain integer-grid console) to render at
/// sub-pixel precision too - a much bigger change than this bug warrants
/// right now.
fn is_player(ecs: &SubWorld, entity: Entity) -> bool {
    ecs.entry_ref(entity)
        .ok()
        .map_or(false, |e| e.get_component::<Player>().is_ok())
}

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Health)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(MovingAnimation)]
pub fn entity_render(#[resource] camera: &Camera, ecs: &SubWorld) {
    let mut renderables = <(Entity, &Point, &Render)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    // Any entity currently mid-glide (see components::gliding_position)
    // draws here instead of on console 1 above - a "fancy console" on the
    // same grid/cell size (see GLIDE_CONSOLE), with a genuinely
    // transparent background, so a sub-pixel position doesn't reveal a
    // background seam the way a fancy console's normal opaque quad would
    // (this is the same fix END_SCREEN_FALLEN_CONSOLE's fallen-hero
    // portrait needed). Registered last in main()'s builder chain, so it
    // paints on top of console 1 - correct, since a gliding entity is
    // deliberately skipped below rather than drawn on both consoles at
    // once.
    let mut glide_batch = DrawBatch::new();
    glide_batch.target(GLIDE_CONSOLE);
    let offset = Point::new(camera.left_x, camera.top_y);

    let player_fov = fov.iter(ecs).nth(0).unwrap();

    renderables
        .iter(ecs)
        .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
        .for_each(|(entity, pos, render)| {
            let color = tinted_color(ecs, *entity, render.color);
            // The player is deliberately excluded from the glide - see
            // is_player below. Every other entity still glides normally.
            let glide_target = if is_player(ecs, *entity) {
                None
            } else {
                gliding_position(ecs, *entity)
            };
            match glide_target {
                Some((fx, fy)) => {
                    let draw_pos = PointF::new(
                        fx - offset.x as f32,
                        fy - offset.y as f32 + GLIDE_CONSOLE_Y_ANCHOR_OFFSET,
                    );
                    // Fully transparent background (RGBA alpha 0) - same
                    // proven trick as the GameOver fallen portrait. Only
                    // the foreground changes vs. the plain-console draw
                    // below; color.fg carries the same low-health/stealth
                    // tinting tinted_color() already computed above.
                    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
                    glide_batch.set_fancy(
                        draw_pos,
                        0,
                        Degrees::new(0.0),
                        PointF::new(1.0, 1.0),
                        ColorPair::new(color.fg, bg_transparent),
                        render.glyph,
                    );
                }
                None => {
                    draw_batch.set(*pos - offset, color, render.glyph);
                }
            }
        });

    draw_batch.submit(5000).expect("Batch error");
    glide_batch.submit(5100).expect("Batch error");
}

/// Overrides a dungeon-view entity's color for two player-only status
/// indicators - non-player entities (enemies, items) always render with
/// their normal Render.color unchanged. Invisible (stealth) takes
/// priority over low health, since a stealthed player being visually
/// flagged as "in danger" would undercut the point of being hidden.
fn tinted_color(ecs: &SubWorld, entity: Entity, base: ColorPair) -> ColorPair {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return base,
    };
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
