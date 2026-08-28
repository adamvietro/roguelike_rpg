use crate::prelude::*;

/// Same set_fancy anchoring quirk documented in detail on
/// entity_render's GLIDE_CONSOLE_Y_ANCHOR_OFFSET and render_helpers'
/// WIGGLE_CONSOLE_Y_ANCHOR_OFFSET: a glyph placed via set_fancy renders
/// one full cell too far north compared to the same position placed via
/// a plain console's set(). Applied here for MAP_SCROLL_CONSOLE, the
/// third console this has now shown up on - not re-measured
/// independently for this specific console, carried over as an
/// assumption the same way the wiggle console's own offset was, since
/// all three share the same underlying dungeonfont.png/32x32px
/// fancy-console setup and the effect has been consistent every time so
/// far. First thing to check if MAP_SCROLL_CONSOLE ever renders visibly
/// high.
const MAP_SCROLL_Y_ANCHOR_OFFSET: f32 = 1.0;

#[system]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Point)]
#[read_component(MovingAnimation)]
pub fn map_render(
    #[resource] map: &Map,
    #[resource] camera: &Camera,
    #[resource] theme: &Box<dyn MapTheme>,
    ecs: &SubWorld,
) {
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    match camera_render_offset(ecs) {
        None => {
            // Camera at rest - unchanged from before this session. The
            // common case by far: every frame except the ~220ms the
            // player is actually mid-step.
            let mut draw_batch = DrawBatch::new();
            draw_batch.target(0);
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
        Some((ox, oy)) => {
            // The camera itself is panning (see camera_render_offset).
            // A plain console like console 0 can only ever be drawn to
            // at integer cell positions, so every visible tile has to
            // be redrawn via set_fancy at a fractional position on
            // MAP_SCROLL_CONSOLE instead - see that console's doc
            // comment in main.rs for why it needs to be a whole separate
            // console rather than just changing the offset passed to
            // draw_batch.set on console 0.
            let mut draw_batch = DrawBatch::new();
            draw_batch.target(MAP_SCROLL_CONSOLE);

            // One tile of padding on every side: camera.left_x/right_x/
            // top_y/bottom_y are already snapped to the destination tile
            // (Camera::on_player_move fires the instant a move commits,
            // well before the glide finishes playing out - see
            // systems/movement.rs), but (ox, oy) can still be lagging up
            // to one full tile behind that, on the source side, while
            // the glide plays out. Without this padding, the column/row
            // that's supposed to be sliding into view from off-screen
            // would be clipped instead of drawn.
            for y in (camera.top_y - 1)..=(camera.bottom_y + 1) {
                for x in (camera.left_x - 1)..(camera.right_x + 1) {
                    let pt = Point::new(x, y);
                    if let Some((color_pair, glyph)) =
                        tile_render_at(map, theme.as_ref(), &player_fov.visible_tiles, pt)
                    {
                        draw_batch.set_fancy(
                            PointF::new(
                                pt.x as f32 - ox,
                                pt.y as f32 - oy + MAP_SCROLL_Y_ANCHOR_OFFSET,
                            ),
                            0,
                            Degrees::new(0.0),
                            PointF::new(1.0, 1.0),
                            color_pair,
                            glyph,
                        );
                    }
                }
            }
            draw_batch.submit(0).expect("Batch error");
        }
    }
}
