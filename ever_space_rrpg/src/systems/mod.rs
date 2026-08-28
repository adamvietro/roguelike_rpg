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
        .flush()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
        .add_system(hud::hud_system())
        .add_system(end_turn::end_turn_system())
        .build()
}

/// Redraws the map (console 0) and entities (console 1) - the
/// always-every-frame half of the decorative background shown behind the
/// title and class-select screens. See build_title_background_movement_scheduler
/// for the enemy-wandering half, which is deliberately NOT run every
/// frame here (see State::background_move_timer_ms for why). No
/// fov_system here - running it would recompute the anchor's FieldOfView
/// from scratch and undo the "everything already explored" full-map
/// reveal it's set up with once at spawn time.
pub fn build_title_background_scheduler() -> Schedule {
    Schedule::builder()
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
pub fn build_title_background_movement_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(random_move::random_move_system())
        .flush()
        .add_system(movement::movement_system())
        .flush()
        .add_system(animation::tick_animations_system())
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
