use crate::prelude::*;

/// Wall-clock milliseconds since the last frame (see BTerm::frame_time_ms),
/// inserted as a resource every tick so animation systems advance at a
/// consistent real-world speed regardless of the current frame rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameTime(pub f32);

/// Standard ease-out cubic: fast start, gentle settle into the
/// destination tile rather than a linear, slightly mechanical glide.
pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// If `entity` currently has an in-flight MovingAnimation, returns its
/// eased fractional (x, y) for *this* frame - still in logical/world
/// coordinates, before the camera offset entity_render applies. Returns
/// None once the glide has finished (elapsed_ms has caught up to
/// MOVE_ANIM_DURATION_MS) or if the entity was never moving, so callers
/// know to fall back to drawing the entity's plain integer Point instead.
///
/// Deliberately returns a raw (f32, f32) tuple rather than a PointF -
/// this codebase has only ever *constructed* a PointF (see
/// draw_end_screen_fallen_portrait), never read x/y back off one, so
/// there's no proof here that PointF exposes public fields the way Point
/// does. Building the tuple ourselves and letting the caller make the
/// final PointF (after subtracting its own camera offset) avoids leaning
/// on that unconfirmed API surface entirely.
///
/// Does NOT consume/remove the animation - that stays tick_animations'
/// job (systems/animation.rs), so "when does a glide end" is only ever
/// decided in one place.
pub fn gliding_position(ecs: &SubWorld, entity: Entity) -> Option<(f32, f32)> {
    let entry = ecs.entry_ref(entity).ok()?;
    let anim = entry.get_component::<MovingAnimation>().ok()?;
    if anim.elapsed_ms >= MOVE_ANIM_DURATION_MS {
        return None;
    }
    let t = ease_out_cubic((anim.elapsed_ms / MOVE_ANIM_DURATION_MS).min(1.0));
    Some((
        lerp(anim.start.x as f32, anim.end.x as f32, t),
        lerp(anim.start.y as f32, anim.end.y as f32, t),
    ))
}

/// Returns the fractional world-space point map_render/entity_render
/// should currently treat as "camera center" for actual DRAWING - as
/// opposed to Camera's own left_x/top_y/right_x/bottom_y, which
/// Camera::on_player_move snaps to the player's destination tile the
/// instant a move is committed (see systems/movement.rs) and which
/// drive what world region counts as "in view" for tile iteration and
/// FOV, not how any of it lands on screen.
///
/// While the player's own MovingAnimation is in flight, this reuses its
/// eased in-between position (see gliding_position above) - the exact
/// same data entity_render already reads for any OTHER gliding entity -
/// minus half the display, so the screen visibly pans from the old
/// center to the new one over MOVE_ANIM_DURATION_MS instead of
/// snapping. See MAP_SCROLL_CONSOLE/ENTITY_SCROLL_CONSOLE in main.rs
/// for the two fancy consoles this drives. Returns None once the
/// player isn't animating (including "never has" and "glide already
/// expired"), telling callers to fall back to Camera's own integer
/// left_x/top_y - the cheap, by-far-more-common path, taken every frame
/// the player isn't actively mid-step.
///
/// Deliberately recomputed fresh from the ECS on every call rather than
/// cached on Camera and refreshed by some dedicated per-frame system: a
/// cached field is only ever as fresh as whatever schedule last wrote
/// it, and not every schedule that calls map_render runs the same
/// systems ahead of it (build_pause_scheduler, for one, runs map_render
/// completely alone). Recomputing here means there's no stale-value
/// case to reason about - whatever this returns is true for the exact
/// instant it's called, in any schedule, always.
///
/// Returns a raw (f32, f32) tuple rather than a PointF, for the same
/// reason gliding_position does above: nothing in this codebase has
/// ever read x/y fields back off a PointF, so there's no confirmed way
/// to subtract one from a Point/(i32,i32) pair. Callers build the final
/// PointF themselves after doing that subtraction in plain f32 math.
///
/// Interpolates between `Camera::clamped_top_left` of the glide's start
/// and end tile (the same clamp `Camera::new`/`on_player_move` apply),
/// rather than the player's own eased position minus a constant half-
/// window offset - the two only agree when neither endpoint is close
/// enough to a map edge for the clamp to actually do anything. Near an
/// edge, using the player's raw position would visibly disagree with
/// where the discrete camera actually lands the instant this glide
/// commits (see `Camera::clamped_top_left`'s own doc comment); lerping
/// the two ALREADY-clamped corners instead means this always agrees with
/// the real camera, whether the clamp is active for the whole step, only
/// part of it (the step that first reaches an edge), or not at all.
pub fn camera_render_offset(ecs: &SubWorld, camera: &Camera) -> Option<(f32, f32)> {
    let (player_entity, _) = find_player(ecs)?;
    let entry = ecs.entry_ref(player_entity).ok()?;
    let anim = entry.get_component::<MovingAnimation>().ok()?;
    if anim.elapsed_ms >= MOVE_ANIM_DURATION_MS {
        return None;
    }
    let t = ease_out_cubic((anim.elapsed_ms / MOVE_ANIM_DURATION_MS).min(1.0));
    let (start_left, start_top) = camera.clamped_top_left(anim.start);
    let (end_left, end_top) = camera.clamped_top_left(anim.end);
    Some((
        lerp(start_left as f32, end_left as f32, t),
        lerp(start_top as f32, end_top as f32, t),
    ))
}

