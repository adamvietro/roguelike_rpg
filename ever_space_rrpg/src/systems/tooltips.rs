use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Name)]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn tooltips(ecs: &SubWorld, #[resource] mouse_pos: &Point, #[resource] camera: &Camera) {
    let mut positions = <(Entity, &Point, &Name)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let offset = Point::new(camera.left_x, camera.top_y);
    let map_pos = *mouse_pos + offset;
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(4);
    let player_fov = fov.iter(ecs).nth(0).unwrap();
    positions
        .iter(ecs)
        .filter(|(_, pos, _)| **pos == map_pos && player_fov.visible_tiles.contains(&pos))
        .for_each(|(entity, _, name)| {
            // mouse_pos is in console 0's cell coordinates (32px tiles).
            // Convert to the HUD console's coordinate space by the ratio
            // of cell counts - both consoles span the same physical
            // window, so this works regardless of either console's exact
            // cell size.
            let screen_pos = Point::new(
                (mouse_pos.x as f32 * HUD_COLS as f32 / DISPLAY_WIDTH as f32) as i32,
                (mouse_pos.y as f32 * HUD_ROWS as f32 / DISPLAY_HEIGHT as f32) as i32,
            );
            let display =
                if let Ok(health) = ecs.entry_ref(*entity).unwrap().get_component::<Health>() {
                    format!("{} : {} hp", &name.0, health.current)
                } else {
                    name.0.clone()
                };
            draw_batch.print(screen_pos, &display);
        });
    draw_batch.submit(10100).expect("Batch error");
}
