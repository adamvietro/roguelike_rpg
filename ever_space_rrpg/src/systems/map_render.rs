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

    // (ox, oy): the fractional camera-relative offset every fancy-console
    // draw below is positioned against. While the camera is genuinely
    // panning, this is the real lerped in-between position (see
    // camera_render_offset). At rest, there's no lerp in progress, so it
    // simply falls back to the camera's own resting integer position -
    // the same value `offset` used to be for the old plain-console path
    // below, just as an (f32, f32) instead of a Point.
    let render_offset = camera_render_offset(ecs, camera);
    let is_panning = render_offset.is_some();
    let (ox, oy) = render_offset.unwrap_or((camera.left_x as f32, camera.top_y as f32));

    // Padded by 1 tile on every side: needed while panning so the column/
    // row sliding into view from off-screen isn't clipped (camera.left_x/
    // right_x/top_y/bottom_y are already snapped to the destination tile,
    // but (ox, oy) can still be lagging up to one full tile behind that,
    // on the source side, while the glide plays out - see
    // camera_render_offset's own doc comment). Harmless overdraw at rest
    // (bracket-lib silently no-ops an out-of-range `set`/`set_fancy`) so
    // there's no need for a second, unpadded range there.
    let x_range = (camera.left_x - 1)..(camera.right_x + 1);
    let y_range = (camera.top_y - 1)..=(camera.bottom_y + 1);

    let mut tile_draw_batch = DrawBatch::new();
    tile_draw_batch.target(MAP_TILE_CONSOLE);
    let mut tile_scroll_batch = DrawBatch::new();
    tile_scroll_batch.target(MAP_TILE_SCROLL_CONSOLE);
    // TileSpriteSheet::Dungeon (Exit/Counter/Water - see map_tile_glyph's
    // own doc comment for why these three specifically never get a real
    // per-theme texture) always renders through the fancy console, at
    // rest or panning alike - found 2026-09-13, confirmed with a real
    // recording: this exact glyph/color/font renders correctly every
    // single time it goes through MAP_SCROLL_CONSOLE's set_fancy, and
    // renders solid black every single time it goes through console 0's
    // plain set() while the camera is at rest, despite both paths
    // computing the identical ColorPair/glyph (verified via debug
    // logging) and console 0's own with_bg shader logic looking correct
    // on paper (traced against bracket-terminal's actual .wgsl source).
    // Root cause not pinned down after extensive tracing (shader source,
    // vertex-buffer building, FontScaler UV math - all identical for both
    // paths); this sidesteps it entirely by only ever using the path
    // that's actually been confirmed to work. Floor/Wall's real-texture
    // path (MAP_TILE_CONSOLE/MAP_TILE_SCROLL_CONSOLE below) isn't
    // reported broken, so it keeps its original plain/fancy split rather
    // than being switched over speculatively.
    let mut dungeon_scroll_batch = DrawBatch::new();
    dungeon_scroll_batch.target(MAP_SCROLL_CONSOLE);

    for y in y_range {
        for x in x_range.clone() {
            let pt = Point::new(x, y);
            if let Some((color_pair, glyph, sheet)) =
                tile_render_at(map, theme.as_ref(), &player_fov.visible_tiles, pt)
            {
                let fx = pt.x as f32 - ox;
                let fy = pt.y as f32 - oy + MAP_SCROLL_Y_ANCHOR_OFFSET;
                match sheet {
                    TileSpriteSheet::Dungeon => {
                        dungeon_scroll_batch.set_fancy(
                            PointF::new(fx, fy),
                            0,
                            Degrees::new(0.0),
                            PointF::new(1.0, 1.0),
                            color_pair,
                            glyph,
                        );
                    }
                    TileSpriteSheet::MapTiles if is_panning => {
                        tile_scroll_batch.set_fancy(
                            PointF::new(fx, fy),
                            0,
                            Degrees::new(0.0),
                            PointF::new(1.0, 1.0),
                            color_pair,
                            glyph,
                        );
                    }
                    TileSpriteSheet::MapTiles => {
                        let offset = Point::new(camera.left_x, camera.top_y);
                        tile_draw_batch.set(pt - offset, color_pair, glyph);
                    }
                }
            }
        }
    }
    dungeon_scroll_batch.submit(0).expect("Batch error");
    tile_draw_batch.submit(1).expect("Batch error");
    tile_scroll_batch.submit(1).expect("Batch error");
}
