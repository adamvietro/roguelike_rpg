use crate::prelude::*;

#[system]
#[read_component(FieldOfView)]
#[read_component(Player)]
pub fn map_render(
    #[resource] map: &Map,
    #[resource] camera: &Camera,
    #[resource] theme: &Box<dyn MapTheme>,
    ecs: &SubWorld,
) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(0);

    let player_fov = fov.iter(ecs).nth(0).unwrap();
    let offset = Point::new(camera.left_x, camera.top_y);

    for y in camera.top_y..=camera.bottom_y {
        for x in camera.left_x..camera.right_x {
            let pt = Point::new(x, y);
            if let Some((color_pair, glyph)) =
                tile_render_at(map, theme.as_ref(), &player_fov.visible_tiles, pt)
            {
                draw_batch.set(pt - offset, color_pair, glyph);
            }
        }
    }
    draw_batch.submit(0).expect("Batch error");
}
