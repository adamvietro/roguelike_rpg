use crate::prelude::*;

mod animation;
mod chasing;
mod end_turn;
mod entity_render;
mod fov;
mod hud;
mod map_render;
mod movement;
mod player_input;
mod random_move;
mod tooltips;
mod traps;
mod use_items;

pub fn build_input_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(player_input::player_input_system())
        .add_system(fov::fov_system())
        .flush()
        .add_system(animation::tick_animations_system())
        .add_system(animation::tick_idle_animation_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(tooltips::tooltips_system())
        .build()
}

pub fn build_player_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(use_items::use_items_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(animation::tick_animations_system())
        .add_system(animation::tick_idle_animation_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

pub fn build_monster_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(random_move::random_move_system())
        .add_system(chasing::chasing_system())
        .flush()
        .add_system(use_items::use_items_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(traps::traps_system())
        .flush()
        .add_system(fov::fov_system())
        .flush()
        .add_system(animation::tick_animations_system())
        .add_system(animation::tick_idle_animation_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

/// Ticks animation (both glide-in-progress and idle-loop-while-standing)
/// and redraws the map (console 0) and entities (console 1) - the
/// always-every-frame half of the decorative background shown behind the
/// title and class-select screens. See build_title_background_movement_scheduler
/// for the enemy-wandering half, which is deliberately NOT run every
/// frame here (see State::background_move_timer_ms for why). No
/// fov_system here - running it would recompute the anchor's FieldOfView
/// from scratch and undo the "everything already explored" full-map
/// reveal it's set up with once at spawn time.
///
/// Animation ticking used to live in build_title_background_movement_
/// scheduler instead, alongside the actual step-taking - which meant it
/// only ever ran once every BACKGROUND_MOVE_INTERVAL_MS (400ms) and, each
/// time it did, only advanced by that one triggering frame's real elapsed
/// time (a few ms), not the ~400ms that had actually passed. Both
/// tick_animations (the walk-between-tiles glide) and
/// tick_idle_animation (the walking-in-place loop while stationary) add
/// FrameTime's value directly (see their own doc comments in
/// systems/animation.rs) - correct when ticked every real frame like
/// normal gameplay already does, but starved to a small fraction of real
/// speed when only ticked once per throttled movement step. In practice
/// this meant a background enemy's idle animation took roughly 9 real
/// seconds to advance a single frame - it read as frozen, not walking in
/// place, confirmed via a real screenshot. Moving both tick systems here
/// (run every frame regardless of the movement throttle) fixes both:
/// idle animation now advances at the same real pace the class-select
/// portrait's own hand-ticked animation already does, and an enemy's
/// glide between two tiles plays out over its real MOVE_ANIM_DURATION_MS
/// instead of creeping for hundreds of milliseconds. Only the decision
/// "take a new step" stays throttled - see
/// build_title_background_movement_scheduler below.
pub fn build_title_background_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(animation::tick_animations_system())
        .add_system(animation::tick_idle_animation_system())
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .build()
}

/// Drives ambient wandering for the decorative background enemies (see
/// State::spawn_title_background, which tags them with MovingRandomly).
/// Kept as a SEPARATE schedule from build_title_background_scheduler, and
/// only executed every BACKGROUND_MOVE_INTERVAL_MS (see
/// State::background_move_timer_ms) rather than every rendered frame -
/// real gameplay movement is naturally paced by TurnState only advancing
/// once per player input, which this decorative world has no equivalent
/// of, so without its own throttle enemies moved a full tile 30 times a
/// second (this schedule's frame rate) instead of looking like deliberate
/// steps. Deliberately uses random_move, not chasing - chasing paths
/// every enemy toward the anchor entity in spawn_title_background and
/// could still attempt a battle against it if one ever reached it,
/// whereas random_move's equivalent check just blocks that one move like
/// bumping a wall (see spawn_title_background's Invisible anchor).
///
/// Animation ticking (tick_animations/tick_idle_animation) moved OUT of
/// this schedule and into build_title_background_scheduler above, which
/// runs every real frame instead of just every throttled step - see that
/// function's own doc comment for why leaving them here starved both to
/// a small fraction of real speed.
pub fn build_title_background_movement_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(random_move::random_move_system())
        .flush()
        .add_system(movement::movement_system())
        .build()
}

/// Redraws just the dungeon map (console 0) - no entities, no HUD - so the
/// pause screen can sit over the map the player was just standing on
/// instead of a black screen. Deliberately omits entity_render/hud/
/// player_input/fov: nothing should move, animate, or otherwise change
/// while paused, this only needs to repaint what's already there since
/// every console gets cleared each tick.
pub fn build_pause_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(map_render::map_render_system())
        .build()
}
