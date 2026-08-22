use crate::prelude::*;

/// Advances every in-flight MovingAnimation by this frame's real elapsed
/// time (see FrameTime) and drops the component once a glide finishes -
/// entity_render then falls back to the entity's plain logical Point.
/// Movement itself already happened instantly in movement.rs; this system
/// only ever affects what gets drawn, never game logic, so it's safe to
/// run every single frame regardless of TurnState (it's wired into all
/// three schedules in systems/mod.rs, including the AwaitingInput one,
/// which is what lets a glide keep playing across the many idle frames
/// spent waiting for the next keypress).
#[system(for_each)]
pub fn tick_animations(
    entity: &Entity,
    anim: &mut MovingAnimation,
    #[resource] frame_time: &FrameTime,
    commands: &mut CommandBuffer,
) {
    anim.elapsed_ms += frame_time.0;
    if anim.elapsed_ms >= MOVE_ANIM_DURATION_MS {
        commands.remove_component::<MovingAnimation>(*entity);
    }
}
