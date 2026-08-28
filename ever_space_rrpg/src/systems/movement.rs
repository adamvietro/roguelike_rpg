use crate::prelude::*;

#[system(for_each)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Item)]
#[read_component(Weapon)]
#[read_component(Carried)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
) {
    if map.can_enter_tile(want_move.destination) {
        // Snapshot wherever this entity is currently drawn FROM, before we
        // overwrite its logical Point, so entity_render can glide it from
        // there to the destination instead of popping straight there.
        if let Some(start) = ecs
            .entry_ref(want_move.entity)
            .ok()
            .and_then(|e| e.get_component::<Point>().ok().copied())
        {
            commands.add_component(
                want_move.entity,
                MovingAnimation {
                    start,
                    end: want_move.destination,
                    elapsed_ms: 0.0,
                },
            );
        }

        commands.add_component(want_move.entity, want_move.destination);

        if let Ok(entry) = ecs.entry_ref(want_move.entity) {
            if let Ok(fov) = entry.get_component::<FieldOfView>() {
                commands.add_component(want_move.entity, fov.clone_dirty());

                if entry.get_component::<Player>().is_ok()
                // (1)
                {
                    camera.on_player_move(want_move.destination);
                    fov.visible_tiles.iter().for_each(|pos| {
                        // (2)
                        map.revealed_tiles[map_idx(pos.x, pos.y)] = true;
                    });

                    // Auto-pickup: no separate pickup key anymore -
                    // walking onto an item's tile picks it up as part of
                    // landing there. Gated behind the same
                    // can_enter_tile check this whole block already sits
                    // inside, so this can never fire on a blocked move
                    // (e.g. bumping a wall) - only an actual, committed
                    // step onto the tile picks anything up. Same
                    // remove-Point/add-Carried mechanics the old G-key
                    // handler in player_input.rs used, just triggered by
                    // the move landing instead of a separate keypress.
                    let items_here: Vec<Entity> = <(Entity, &Item, &Point)>::query()
                        .iter(ecs)
                        .filter(|(_, _, &pos)| pos == want_move.destination)
                        .map(|(e, _, _)| *e)
                        .collect();
                    for item_entity in items_here {
                        commands.remove_component::<Point>(item_entity);
                        commands.add_component(item_entity, Carried(want_move.entity));

                        // A picked-up Weapon replaces (discards) any
                        // weapon already carried, same one-equipped-
                        // weapon-at-a-time rule the old handler
                        // enforced. This query still only sees
                        // PREVIOUSLY-carried weapons, not the one just
                        // queued above - CommandBuffer edits aren't
                        // visible until flush, same deferred-command
                        // reasoning the old handler relied on.
                        if let Ok(item_entry) = ecs.entry_ref(item_entity) {
                            if item_entry.get_component::<Weapon>().is_ok() {
                                <(Entity, &Carried, &Weapon)>::query()
                                    .iter(ecs)
                                    .filter(|(_, c, _)| c.0 == want_move.entity)
                                    .for_each(|(e, _, _)| {
                                        commands.remove(*e);
                                    });
                            }
                        }
                    }
                }
            }
        }
    }
    commands.remove(*entity);
}
