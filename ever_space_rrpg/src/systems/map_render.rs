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
    let wall_base = theme.wall_color();

    for y in camera.top_y..=camera.bottom_y {
        for x in camera.left_x..camera.right_x {
            let pt = Point::new(x, y);
            let offset = Point::new(camera.left_x, camera.top_y);
            let idx = map_idx(x, y);
            if map.in_bounds(pt)
                && (player_fov.visible_tiles.contains(&pt) | map.revealed_tiles[idx])
            {
                let visible = player_fov.visible_tiles.contains(&pt);
                let glyph = theme.tile_to_render(map.tiles[idx]);

                let color_pair = if map.tiles[idx] == TileType::Wall {
                    // Solid-filled wall: theme color as background (not
                    // just a thin foreground glyph on black, which barely
                    // shows up), a brighter shade of the same color as
                    // foreground texture, dimmed when only remembered
                    // rather than currently visible.
                    let bg = if visible {
                        wall_base
                    } else {
                        RGB::from_f32(wall_base.r * 0.35, wall_base.g * 0.35, wall_base.b * 0.35)
                    };
                    let fg = RGB::from_f32(
                        (bg.r * 1.4).min(1.0),
                        (bg.g * 1.4).min(1.0),
                        (bg.b * 1.4).min(1.0),
                    );
                    ColorPair::new(fg, bg)
                } else {
                    let tint = if visible { WHITE } else { DARK_GRAY };
                    ColorPair::new(tint, BLACK)
                };

                draw_batch.set(pt - offset, color_pair, glyph);
            }
        }
    }
    draw_batch.submit(0).expect("Batch error");
}
