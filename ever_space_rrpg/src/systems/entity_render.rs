use crate::prelude::*;

/// Below this fraction of max HP, the player's dungeon-view glyph tints
/// red - see tinted_color.
const LOW_HEALTH_THRESHOLD: f32 = 0.3;

/// Empirically-confirmed set_fancy quirk: a glyph placed via set_fancy
/// at the same (x, y) that places it correctly via a plain console's
/// set() renders exactly one full cell too far north, consistently,
/// regardless of movement direction - see the fuller writeup that used
/// to live on this same constant name (still true, just no longer
/// specific to a single console), and the matching
/// WIGGLE_CONSOLE_Y_ANCHOR_OFFSET in render_helpers.rs /
/// MAP_SCROLL_Y_ANCHOR_OFFSET in map_render.rs (three independent
/// confirmations of the same bracket-lib behavior now). Used by both
/// fancy-console paths below - GLIDE_CONSOLE (one entity gliding while
/// the camera itself sits still) and ENTITY_SCROLL_CONSOLE (every
/// entity, while the camera itself is panning) are the same underlying
/// set_fancy draw, just fed different offsets - see
/// components::camera_render_offset for where those offsets come from.
const GLIDE_CONSOLE_Y_ANCHOR_OFFSET: f32 = 1.0;

#[system]
#[read_component(Point)]
#[read_component(Render)]
#[read_component(FieldOfView)]
#[read_component(Player)]
#[read_component(Health)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(Frozen)]
#[read_component(MovingAnimation)]
#[read_component(IdleAnimation)]
#[read_component(EffectAnimation)]
pub fn entity_render(#[resource] camera: &Camera, ecs: &SubWorld) {
    let mut renderables = <(Entity, &Point, &Render)>::query();
    let mut fov = <&FieldOfView>::query().filter(component::<Player>());
    let player_fov = fov.iter(ecs).nth(0).unwrap();

    match camera_render_offset(ecs, camera) {
        None => {
            // Camera at rest. A per-entity gliding_position check
            // decides whether THIS entity draws on the sub-pixel
            // GLIDE_CONSOLE (a monster taking its own step while the
            // player stands still - the only way to reach this branch
            // with anything actually gliding) or the plain integer
            // ENTITY_CONSOLE (everything else). The player is never
            // mid-glide whenever this branch runs - see
            // camera_render_offset's doc comment - so it always lands
            // in the plain-console case here, with no special-casing
            // needed for it.
            let offset = Point::new(camera.left_x, camera.top_y);
            let mut draw_batch = DrawBatch::new();
            draw_batch.target(ENTITY_CONSOLE);
            let mut character_batch = DrawBatch::new();
            character_batch.target(CHARACTER_IDLE_CONSOLE);
            let mut enemy_batch = DrawBatch::new();
            enemy_batch.target(ENEMY_IDLE_CONSOLE);
            let mut effect_batch = DrawBatch::new();
            effect_batch.target(CHARACTER_EFFECT_CONSOLE);
            let mut glide_batch = DrawBatch::new();
            glide_batch.target(GLIDE_CONSOLE);
            let mut character_glide_batch = DrawBatch::new();
            character_glide_batch.target(CHARACTER_IDLE_GLIDE_CONSOLE);
            let mut enemy_glide_batch = DrawBatch::new();
            enemy_glide_batch.target(ENEMY_IDLE_GLIDE_CONSOLE);
            let mut effect_glide_batch = DrawBatch::new();
            effect_glide_batch.target(CHARACTER_EFFECT_GLIDE_CONSOLE);
            let mut shopkeeper_batch = DrawBatch::new();
            shopkeeper_batch.target(SHOPKEEPER_IDLE_CONSOLE);
            let mut shopkeeper_glide_batch = DrawBatch::new();
            shopkeeper_glide_batch.target(SHOPKEEPER_IDLE_GLIDE_CONSOLE);

            renderables
                .iter(ecs)
                .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
                .for_each(|(entity, pos, render)| {
                    let color = tinted_color(ecs, *entity, render.color);
                    let glyph = idle_glyph(ecs, *entity, render.glyph);
                    let sheet = idle_sheet(ecs, *entity);
                    match gliding_position(ecs, *entity) {
                        Some((fx, fy)) => {
                            let batch = match sheet {
                                IdleSpriteSheet::Dungeon => &mut glide_batch,
                                IdleSpriteSheet::CharacterIdle => &mut character_glide_batch,
                                IdleSpriteSheet::EnemyIdle => &mut enemy_glide_batch,
                                IdleSpriteSheet::CharacterEffect => &mut effect_glide_batch,
                                IdleSpriteSheet::Shopkeeper => &mut shopkeeper_glide_batch,
                            };
                            draw_glyph_fancy(
                                batch,
                                fx - offset.x as f32,
                                fy - offset.y as f32,
                                color,
                                glyph,
                            )
                        }
                        None => match sheet {
                            IdleSpriteSheet::Dungeon => {
                                draw_batch.set(*pos - offset, color, glyph);
                            }
                            IdleSpriteSheet::CharacterIdle => {
                                character_batch.set(*pos - offset, color, glyph);
                            }
                            IdleSpriteSheet::EnemyIdle => {
                                enemy_batch.set(*pos - offset, color, glyph);
                            }
                            IdleSpriteSheet::CharacterEffect => {
                                effect_batch.set(*pos - offset, color, glyph);
                            }
                            IdleSpriteSheet::Shopkeeper => {
                                shopkeeper_batch.set(*pos - offset, color, glyph);
                            }
                        },
                    }
                });

            draw_batch.submit(5000).expect("Batch error");
            character_batch.submit(5050).expect("Batch error");
            enemy_batch.submit(5075).expect("Batch error");
            effect_batch.submit(5080).expect("Batch error");
            glide_batch.submit(5100).expect("Batch error");
            character_glide_batch.submit(5150).expect("Batch error");
            enemy_glide_batch.submit(5175).expect("Batch error");
            effect_glide_batch.submit(5180).expect("Batch error");
            shopkeeper_batch.submit(5085).expect("Batch error");
            shopkeeper_glide_batch.submit(5185).expect("Batch error");
        }
        Some((ox, oy)) => {
            // The camera itself is panning - the player is mid-glide
            // (see camera_render_offset). Every entity, moving or not,
            // now needs a fractional position derived from this same
            // (ox, oy), or a stationary one would stay snapped to its
            // old integer screen cell while the map slides underneath
            // it - see ENTITY_SCROLL_CONSOLE's doc comment in main.rs.
            //
            // This includes the player, which is no longer a special
            // case: substituting the player's own gliding_position into
            // "screen pos = world pos - (ox, oy)" lands on exactly the
            // screen center for every frame of its own glide, since
            // (ox, oy) is itself defined as "the player's eased
            // position minus screen center" (see camera_render_offset).
            // That identity is what actually fixes the old jump/
            // snap-back bug this file used to work around by excluding
            // the player from its own glide entirely - it isn't a
            // special case anymore, just the same formula every other
            // entity already used.
            let mut scroll_batch = DrawBatch::new();
            scroll_batch.target(ENTITY_SCROLL_CONSOLE);
            let mut character_scroll_batch = DrawBatch::new();
            character_scroll_batch.target(CHARACTER_IDLE_SCROLL_CONSOLE);
            let mut enemy_scroll_batch = DrawBatch::new();
            enemy_scroll_batch.target(ENEMY_IDLE_SCROLL_CONSOLE);
            let mut effect_scroll_batch = DrawBatch::new();
            effect_scroll_batch.target(CHARACTER_EFFECT_SCROLL_CONSOLE);
            let mut shopkeeper_scroll_batch = DrawBatch::new();
            shopkeeper_scroll_batch.target(SHOPKEEPER_IDLE_SCROLL_CONSOLE);

            renderables
                .iter(ecs)
                .filter(|(_, pos, _)| player_fov.visible_tiles.contains(pos))
                .for_each(|(entity, pos, render)| {
                    let color = tinted_color(ecs, *entity, render.color);
                    let glyph = idle_glyph(ecs, *entity, render.glyph);
                    let (fx, fy) =
                        gliding_position(ecs, *entity).unwrap_or((pos.x as f32, pos.y as f32));
                    let batch = match idle_sheet(ecs, *entity) {
                        IdleSpriteSheet::Dungeon => &mut scroll_batch,
                        IdleSpriteSheet::CharacterIdle => &mut character_scroll_batch,
                        IdleSpriteSheet::EnemyIdle => &mut enemy_scroll_batch,
                        IdleSpriteSheet::CharacterEffect => &mut effect_scroll_batch,
                        IdleSpriteSheet::Shopkeeper => &mut shopkeeper_scroll_batch,
                    };
                    draw_glyph_fancy(batch, fx - ox, fy - oy, color, glyph);
                });

            scroll_batch.submit(5000).expect("Batch error");
            character_scroll_batch.submit(5050).expect("Batch error");
            enemy_scroll_batch.submit(5075).expect("Batch error");
            effect_scroll_batch.submit(5080).expect("Batch error");
            shopkeeper_scroll_batch.submit(5085).expect("Batch error");
        }
    }
}

/// Shared by both branches above: draws one glyph onto a fancy console
/// (already `.target()`ed by the caller) with a fully transparent
/// background and the Y-anchor correction applied - the same proven
/// trick END_SCREEN_FALLEN_CONSOLE's fallen-hero portrait needed,
/// without which a fancy console's normally-opaque background quad
/// would paint a visible box sliding over the map every time something
/// moved. `sx`/`sy` are already camera-relative (world position minus
/// whichever offset is active this frame) - callers do that subtraction
/// themselves, since the two branches above get their offset from
/// different places (a fixed integer Point vs. a fractional (f32, f32)
/// pair).
fn draw_glyph_fancy(
    batch: &mut DrawBatch,
    sx: f32,
    sy: f32,
    color: ColorPair,
    glyph: FontCharType,
) {
    let bg_transparent = RGBA::from_f32(0.0, 0.0, 0.0, 0.0);
    batch.set_fancy(
        PointF::new(sx, sy + GLIDE_CONSOLE_Y_ANCHOR_OFFSET),
        0,
        Degrees::new(0.0),
        PointF::new(1.0, 1.0),
        ColorPair::new(color.fg, bg_transparent),
        glyph,
    );
}

/// The glyph to actually draw for this entity this frame: an in-flight
/// EffectAnimation's current glyph if it has one (an out-of-combat
/// ability's brief animation override - see that component's own doc
/// comment), else its current IdleAnimation frame if it has one, else
/// just `base` unchanged. Items/scenery with no IdleAnimation component
/// at all (weapons, potions, etc.) always take the final fallback.
fn idle_glyph(ecs: &SubWorld, entity: Entity, base: FontCharType) -> FontCharType {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return base,
    };
    if let Ok(effect) = entry.get_component::<EffectAnimation>() {
        return effect.0.current_glyph();
    }
    entry
        .get_component::<IdleAnimation>()
        .ok()
        .map(IdleAnimation::current_glyph)
        .unwrap_or(base)
}

/// Which console `idle_glyph`'s returned glyph should actually be drawn
/// on - see IdleSpriteSheet's own doc comment in components.rs. An
/// in-flight EffectAnimation always wins (CharacterEffect), checked
/// before IdleAnimation for the same reason idle_glyph checks it first -
/// both need to agree on which entity/frame source is authoritative this
/// frame. Defaults to Dungeon for anything with neither component at all
/// (items/scenery), matching idle_glyph's own fallback-to-`base`
/// behavior for the same entities - `base` is always a dungeonfont glyph
/// for those.
fn idle_sheet(ecs: &SubWorld, entity: Entity) -> IdleSpriteSheet {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return IdleSpriteSheet::Dungeon,
    };
    if entry.get_component::<EffectAnimation>().is_ok() {
        return IdleSpriteSheet::CharacterEffect;
    }
    entry
        .get_component::<IdleAnimation>()
        .ok()
        .map(|i| i.sheet)
        .unwrap_or(IdleSpriteSheet::Dungeon)
}

/// Overrides a dungeon-view entity's color for a few status indicators.
/// Frozen (Hunter's Freeze Trap) is checked first and applies to ANY
/// entity, not just the player - see components::Frozen. Everything
/// after that is player-only, as before: Invisible (stealth) takes
/// priority over low health, since a stealthed player being visually
/// flagged as "in danger" would undercut the point of being hidden.
fn tinted_color(ecs: &SubWorld, entity: Entity, base: ColorPair) -> ColorPair {
    let entry = match ecs.entry_ref(entity) {
        Ok(e) => e,
        Err(_) => return base,
    };
    if entry.get_component::<Frozen>().is_ok() {
        return ColorPair::new(BLUE, BLACK);
    }
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
