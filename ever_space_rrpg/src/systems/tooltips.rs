use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Name)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(MovingAnimation)]
pub fn tooltips(ecs: &SubWorld, #[resource] mouse_pos: &Point, #[resource] camera: &Camera) {
    let mut positions = <(Entity, &Point, &Name)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    // Smooth camera_render_offset while the player is mid-glide (see that
    // function's own doc comment), falling back to Camera's own resting
    // integer left_x/top_y otherwise - the same two-offset pattern
    // map_render/entity_render already use for actual drawing. Without
    // this, hovering an entity during the ~150ms the camera is visibly
    // panning could point at the wrong tile (or none at all), since the
    // stale integer offset briefly disagrees with what's actually on
    // screen.
    let map_pos = match camera_render_offset(ecs) {
        Some((fx, fy)) => Point::new(
            (mouse_pos.x as f32 + fx).round() as i32,
            (mouse_pos.y as f32 + fy).round() as i32,
        ),
        None => *mouse_pos + Point::new(camera.left_x, camera.top_y),
    };
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    // Wrapped in a real render_helpers::PanelBox 2026-09-14 (direct
    // request) - Swamp, matching every other border on this HUD, sized
    // dynamically to the text's own length (this tooltip's content
    // varies - just a name, or "name : N hp" - unlike the shop
    // tooltip's own fixed SHOP_TOOLTIP_WIDTH, which never needs to fit
    // more than one specific line shape). Same dx=2/dy=2 text inset the
    // shop tooltip already uses.
    positions
        .iter(ecs)
        .filter(|(_, pos, _)| **pos == map_pos && player_fov.visible_tiles.contains(&pos))
        .for_each(|(entity, _, name)| {
            // mouse_pos is in console 0's cell coordinates (32px tiles);
            // convert to the HUD console's coordinate space.
            let screen_pos = mouse_to_hud(*mouse_pos);
            let display =
                if let Ok(health) = ecs.entry_ref(*entity).unwrap().get_component::<Health>() {
                    format!("{} : {} hp", &name.0, health.current)
                } else {
                    name.0.clone()
                };
            let width = display.chars().count() as i32 + 4;
            let mut tooltip_box = PanelBox::new(
                screen_pos.x,
                screen_pos.y,
                width,
                4,
                UiPanelTheme::Swamp,
                PIXEL_BOX_TILE_SCALE_COMPACT,
            );
            tooltip_box.text_color(0, 0, WHITE, BLACK, display);
            tooltip_box.submit();
        });
}
