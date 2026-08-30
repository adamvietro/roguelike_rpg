use crate::prelude::*;

#[system(for_each)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(FieldOfView)]
#[read_component(Item)]
#[read_component(Weapon)]
#[read_component(Carried)]
#[read_component(AmuletOfYala)]
pub fn movement(
    entity: &Entity,
    want_move: &WantsToMove,
    #[resource] map: &mut Map,
    #[resource] camera: &mut Camera,
    #[resource] shopping: &Option<ShoppingActive>,
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
                // Normally every move marks FieldOfView dirty so
                // fov_system recomputes real shadowcasting from the new
                // position next frame. Skipped entirely while
                // ShoppingActive - State::start_arena deliberately
                // freezes the shop's FieldOfView as "everything visible,
                // forever" (is_dirty = false, visible_tiles = the whole
                // map), specifically so the shopkeeper stays visible
                // behind the counter (a Counter tile, opaque like a
                // Wall, which real shadowcasting would otherwise never
                // see past). Without this guard, the player's very
                // first step re-dirtied that frozen FOV, fov_system
                // recomputed a normal radius-limited view on the very
                // next frame, and the shopkeeper (and anything past the
                // counter) vanished the instant you moved.
                if shopping.is_none() {
                    commands.add_component(want_move.entity, fov.clone_dirty());
                }

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
                    //
                    // Suppressed entirely while ShoppingActive - an
                    // arena shop's items are a deliberate choice (see
                    // player_input.rs's new buy key), not something
                    // walking past should sweep up.
                    //
                    // The Amulet of Yala is deliberately EXCLUDED here,
                    // even though it has an Item component - it was never
                    // meant to be carried/inventory-managed, its entire
                    // mechanic is "stand on this tile, you win" (see
                    // systems/end_turn.rs, which checks the player's
                    // position against the amulet entity's own Point).
                    // Auto-pickup runs earlier in this same schedule than
                    // end_turn's victory check - if it swept the amulet
                    // up like a normal item, the amulet would lose its
                    // Point (and gain Carried) before end_turn ever got
                    // to compare positions, and Victory would silently
                    // never trigger. The old G-key system never hit this,
                    // since it needed a SEPARATE keypress on a LATER
                    // turn - one that was never reachable, because
                    // stepping onto the amulet already triggered Victory
                    // that same turn, before the player could press G at
                    // all.
                    if shopping.is_none() {
                        let items_here: Vec<Entity> = <(Entity, &Item, &Point)>::query()
                            .iter(ecs)
                            .filter(|(_, _, &pos)| pos == want_move.destination)
                            .filter(|(e, _, _)| {
                                ecs.entry_ref(**e)
                                    .map(|entry| entry.get_component::<AmuletOfYala>().is_err())
                                    .unwrap_or(true)
                            })
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
    }
    commands.remove(*entity);
}
