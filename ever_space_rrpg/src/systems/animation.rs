use crate::prelude::*;

/// Advances every in-flight MovingAnimation by this frame's real elapsed
/// time (see FrameTime). Deliberately does NOT remove the component once
/// a glide finishes - entity_render's gliding_position already treats
/// elapsed_ms >= MOVE_ANIM_DURATION_MS as "not animating" and falls back
/// to the plain logical Point, so leaving the component in place costs
/// nothing.
///
/// This used to call commands.remove_component::<MovingAnimation>() here
/// once expired, with movement.rs re-adding a fresh one on the entity's
/// next move. That meant every single glide involved TWO structural
/// changes to the entity (a component being removed, then later a new
/// one of the same type added back) - and adding/removing a component
/// type moves an entity to a different archetype internally, which is a
/// heavier operation than updating a component that's already there. In
/// practice this caused a real, consistently-reproducible one-frame
/// visual glitch on every move (destination flashed briefly, then the
/// glide appeared to jump backward to the start before playing normally)
/// - other systems reading the entity that same frame were seeing it
/// one archetype-move behind. Leaving the component permanently in place
/// once an entity has moved at least once, and only ever overwriting its
/// fields from here on, avoids that churn entirely.
#[system(for_each)]
pub fn tick_animations(anim: &mut MovingAnimation, #[resource] frame_time: &FrameTime) {
    anim.elapsed_ms += frame_time.0;
}
