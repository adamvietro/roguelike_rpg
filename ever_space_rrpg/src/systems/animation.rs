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

/// Advances every entity's IdleAnimation ("walking in place" - see that
/// component's own doc comment) by this frame's real elapsed time,
/// wrapping back to frame 0 after the last frame. Advances during an
/// in-flight MovingAnimation (mid-glide) too, not just while standing
/// still - changed 2026-09-11 alongside real directional Walk art (see
/// IdleAnimation's own doc comment for the full reasoning): this used to
/// deliberately pause here on the theory that "real movement already has
/// its own animation," but the glide only ever tweened POSITION, never a
/// per-frame pose, so pausing this just froze the character's pose for
/// the whole glide - reported as "the player and enemies become static"
/// while actually moving. No more MovingAnimation lookup needed here at
/// all now that there's no special case to gate on.
#[system(for_each)]
pub fn tick_idle_animation(idle: &mut IdleAnimation, #[resource] frame_time: &FrameTime) {
    if idle.frames.is_empty() {
        return;
    }

    idle.elapsed_ms += frame_time.0;
    if idle.elapsed_ms >= IDLE_FRAME_DURATION_MS {
        idle.elapsed_ms -= IDLE_FRAME_DURATION_MS;
        idle.frame_index = (idle.frame_index + 1) % idle.frames.len();
    }
}

/// Advances any in-flight EffectAnimation (an out-of-combat ability's
/// brief dungeon-view animation override - see that component's own doc
/// comment) and removes it once it finishes, so entity_render's
/// idle_glyph/idle_sheet fall back to the entity's ordinary IdleAnimation
/// loop again. A plain #[system] with an explicit query (not
/// `#[system(for_each)]` like tick_idle_animation above) since this one
/// needs a CommandBuffer to remove the component once done, and no
/// for_each system elsewhere in this project has been proven to accept
/// one directly - mirrors systems/traps.rs's own explicit-query shape
/// for the same reason. Collects finished entities into a local Vec
/// before issuing any commands, rather than removing mid-iteration -
/// same "don't mutate through a CommandBuffer while still iterating the
/// query it would affect" caution as every other CommandBuffer use in
/// this project.
#[system]
#[write_component(EffectAnimation)]
pub fn tick_effect_animation(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] frame_time: &FrameTime,
) {
    let mut finished = Vec::new();
    <(Entity, &mut EffectAnimation)>::query()
        .iter_mut(ecs)
        .for_each(|(entity, effect)| {
            effect.0.tick(frame_time.0);
            if effect.0.finished() {
                finished.push(*entity);
            }
        });
    for entity in finished {
        commands.remove_component::<EffectAnimation>(entity);
    }
}
