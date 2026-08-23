use crate::prelude::*;

/// Below this fraction of max HP, the player's dungeon-view glyph tints
/// red - see tinted_color.
const LOW_HEALTH_THRESHOLD: f32 = 0.3;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Health)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
pub fn entity_render(#[resource] camera: &Camera, ecs: &SubWorld) {
    let mut renderables = <(Entity, &Point, &Render)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let mut draw_batch = DrawBatch::new();
    draw_batch.target(1);
    let offset = Point::new(camera.left_x, camera.top_y);

    let player_fov = fov.iter(ecs).nth(0).unwrap();

    renderables
        .iter(ecs)
        .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
        .for_each(|(entity, pos, render)| {
            let color = tinted_color(ecs, *entity, render.color);
            draw_batch.set(*pos - offset, color, render.glyph);
        });

    draw_batch.submit(5000).expect("Batch error");
}

/// Overrides a dungeon-view entity's color for two player-only status
/// indicators - non-player entities (enemies, items) always render with
/// their normal Render.color unchanged. Invisible (stealth) takes
/// priority over low health, since a stealthed player being visually
/// flagged as "in danger" would undercut the point of being hidden.
fn tinted_color(ecs: &SubWorld, entity: Entity, base: ColorPair) -> ColorPair {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return base,
    };
    if entry.get_component::<Player>().is_err() {
        return base;
    }
    if entry.get_component::<Invisible>().is_ok() || entry.get_component::<Stealthed>().is_ok() {
        return ColorPair::new(GRAY, BLACK);
    }
    if let Ok(health) = entry.get_component::<Health>() {
        if health.max > 0 && (health.current as f32 / health.max as f32) <= LOW_HEALTH_THRESHOLD {
            return ColorPair::new(RED, BLACK);
        }
    }
    base
}
