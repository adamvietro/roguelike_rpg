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

/// Redraws the map (console 0) and entities (console 1) with no other
/// systems running - used for the decorative, fully-revealed background
/// world shown behind the title and class-select screens. Unlike
/// build_pause_scheduler, entities ARE drawn here (the point is to show
/// "a map with everything explored and enemies inside" as backdrop) - see
/// State::spawn_title_background for how that world's fake "player"
/// FieldOfView is pre-filled with the whole map so nothing is hidden by
/// normal fog-of-war rules.
pub fn build_title_background_scheduler() -> Schedule {
    Schedule::builder()
        .add_system(map_render::map_render_system())
        .add_system(entity_render::entity_render_system())
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
