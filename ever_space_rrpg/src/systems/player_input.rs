use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(Enemy)]
#[read_component(Name)]
#[write_component(Health)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Weapon)]
#[read_component(BattleItem)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(AmuletOfYala)]
#[read_component(ShopStock)]
#[read_component(Price)]
#[read_component(Gold)]
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key: &Option<VirtualKeyCode>,
    #[resource] keymap: &Keymap,
    #[resource] turn_state: &mut TurnState,
    #[resource] battle: &mut Option<Battle>,
    #[resource] shopping: &Option<ShoppingActive>,
    #[resource] shop_message: &mut Option<ShopMessage>,
) {
    let mut players = <(Entity, &Point)>::query().filter(component::<Player>());
    let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());

    if let Some(key) = *key {
        // Checked first and returns immediately - Escape never falls
        // through to the movement/attack logic below, and this system
        // only ever runs during TurnState::AwaitingInput (dungeon
        // exploration), so pausing mid-battle isn't reachable: battle_tick
        // handles its own keys entirely separately and never calls this.
        // Escape stays hardcoded rather than going through Keymap - see
        // keymap.rs's module comment on why pause/Escape is deliberately
        // not rebindable.
        if key == VirtualKeyCode::Escape {
            *turn_state = TurnState::Paused;
            return;
        }

        // Cleared on EVERY keypress, then re-set below only if this
        // specific keypress is a buy attempt that fails - so a stale "Not
        // enough gold!" from a previous attempt never lingers once the
        // player's moved on to something else (including a successful
        // buy, which never re-sets it at all).
        *shop_message = None;

        // "Wait" - gathers every enemy within 1 tile (including
        // diagonals) into a single battle at once, the same way walking
        // into an already-stacked tile does, EXCEPT it doesn't require
        // them to already be stacked on one tile first - it also catches
        // several separate nearby enemies that just haven't happened to
        // converge onto the same square. Getting enemies to actually
        // stack turned out to be rare enough in practice that testing
        // multi-enemy battles at all was hard without this. If nothing's
        // close enough, this just passes the turn instead - a genuine
        // "wait a turn in place," the same as many roguelikes bind to
        // this key anyway.
        //
        // Space rather than a letter key - every letter (see
        // keymap::REBINDABLE_KEYS) can be rebound to a movement action,
        // so hardcoding one here risks silently colliding with whatever
        // the player remaps movement to. Space is never rebindable,
        // guaranteeing no such collision regardless of the player's own
        // keymap.
        if key == VirtualKeyCode::Space {
            let player = players.iter(ecs).map(|(e, pos)| (*e, *pos)).next();
            if let Some((player_entity, player_pos)) = player {
                let player_is_invisible = <(Entity, &Invisible)>::query()
                    .iter(ecs)
                    .any(|(e, _)| *e == player_entity);

                let roster: Vec<(Entity, String)> = if player_is_invisible {
                    // While Invisible, this can't start a battle either -
                    // same rule as bumping into an enemy directly (see
                    // the movement branch below) - so there's nothing to
                    // gather; this just falls through to "wait" instead.
                    Vec::new()
                } else {
                    enemies
                        .iter(ecs)
                        .filter(|(_, pos)| {
                            (pos.x - player_pos.x).abs() <= 1 && (pos.y - player_pos.y).abs() <= 1
                        })
                        .map(|(entity, _)| {
                            let name = ecs
                                .entry_ref(*entity)
                                .ok()
                                .and_then(|e| e.get_component::<Name>().ok().cloned())
                                .map(|n| n.0)
                                .unwrap_or_else(|| "the enemy".to_string());
                            (*entity, name)
                        })
                        .take(MAX_BATTLE_ENEMIES)
                        .collect()
                };

                if !roster.is_empty() {
                    // Stealth doesn't block this like Invisible does - it
                    // starts the battle anyway, as an ambush (forced
                    // first turn + 3x damage on the opening action),
                    // exactly like walking into an enemy while Stealthed
                    // already does - see Battle::sneak_attack.
                    let player_is_stealthed = <(Entity, &Stealthed)>::query()
                        .iter(ecs)
                        .any(|(e, _)| *e == player_entity);

                    let new_battle = Battle::new(player_entity, roster);
                    *battle = Some(if player_is_stealthed {
                        commands.remove_component::<Stealthed>(player_entity);
                        new_battle.as_sneak_attack()
                    } else {
                        new_battle
                    });
                    *turn_state = TurnState::InBattle;
                    return;
                }
            }

            // Nothing close enough to fight (or nothing found at all,
            // which shouldn't happen) - just pass the turn.
            *turn_state = TurnState::PlayerTurn;
            return;
        }

        // Only a genuinely turn-consuming action - movement (a real step
        // OR a wall/enemy bump, both of which cost a turn same as
        // before), using a Potion/Map/out-of-combat ability item, or a
        // successful shop purchase - actually advances the dungeon turn
        // below. Every other keypress (an unrecognized key, a number key
        // on an empty inventory slot, a failed purchase) leaves
        // turn_state untouched, so the player can just try again with no
        // penalty instead of quietly handing the monsters a free move.
        // This revives (and actually wires up) a `did_something` flag
        // that used to sit here commented out and unused - the intent
        // was already half-built, just never finished.
        let mut did_something = false;

        // Movement keys go through Keymap now instead of a hardcoded
        // VirtualKeyCode match, so a rebind made in the Options screen
        // (see screens/options.rs) takes effect immediately - the
        // mapping from key to Action lives in keymap.rs, this just asks
        // "what does the CURRENTLY bound key for this press mean."
        let delta = match keymap.action_for_key(key) {
            Some(Action::MoveUp) => Point::new(0, -1),
            Some(Action::MoveDown) => Point::new(0, 1),
            Some(Action::MoveLeft) => Point::new(-1, 0),
            Some(Action::MoveRight) => Point::new(1, 0),
            None => match key {
                VirtualKeyCode::Key1 => {
                    did_something |= use_item(0, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key2 => {
                    did_something |= use_item(1, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key3 => {
                    did_something |= use_item(2, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key4 => {
                    did_something |= use_item(3, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key5 => {
                    did_something |= use_item(4, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key6 => {
                    did_something |= use_item(5, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key7 => {
                    did_something |= use_item(6, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key8 => {
                    did_something |= use_item(7, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key9 => {
                    did_something |= use_item(8, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Key0 => {
                    did_something |= use_item(9, ecs, commands);
                    Point::zero()
                }
                VirtualKeyCode::Return if shopping.is_some() => {
                    did_something |= buy_nearby_item(ecs, commands, shop_message);
                    Point::zero()
                }
                _ => Point::new(0, 0),
            },
        };

        let (player_entity, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos + delta)))
            .unwrap();

        if delta.x != 0 || delta.y != 0 {
            // A movement key was pressed - this always consumes a turn,
            // whether it's a real step, a wall bump (movement.rs silently
            // drops an invalid destination but the turn still passes), or
            // it starts a battle (which returns early below before ever
            // reaching the did_something check at the bottom, so this
            // assignment is moot in that case - InBattle isn't a dungeon
            // turn at all).
            did_something = true;

            // Every enemy actually AT the destination tile joins the
            // battle at once, not just whichever one this find() would
            // have picked first - enemies can and do stack on the same
            // tile (confirmed - see the project instructions doc), and
            // "any enemy in the space of the entity initiating battle"
            // is exactly the rule this project settled on. Capped at
            // MAX_BATTLE_ENEMIES; any additional enemy beyond that still
            // sitting on the tile simply doesn't join THIS fight - it's
            // still there afterward for a follow-up battle.
            let roster: Vec<Entity> = enemies
                .iter(ecs)
                .filter(|(_, pos)| **pos == destination)
                .map(|(entity, _)| *entity)
                .take(MAX_BATTLE_ENEMIES)
                .collect();

            if !roster.is_empty() {
                // While Invisible (see components::Invisible / the
                // Invisible Cloak item), walking into an enemy is blocked
                // like a wall instead of starting a battle - no
                // WantsToMove is queued either, so the player doesn't step
                // onto that tile. The turn still passes via did_something
                // above, same as bumping a real wall does.
                let player_is_invisible = <(Entity, &Invisible)>::query()
                    .iter(ecs)
                    .any(|(e, _)| *e == player_entity);

                if !player_is_invisible {
                    let roster: Vec<(Entity, String)> = roster
                        .into_iter()
                        .map(|enemy_entity| {
                            let name = ecs
                                .entry_ref(enemy_entity)
                                .ok()
                                .and_then(|e| e.get_component::<Name>().ok().cloned())
                                .map(|n| n.0)
                                .unwrap_or_else(|| "the enemy".to_string());
                            (enemy_entity, name)
                        })
                        .collect();

                    // Stealth (Rogue) doesn't block the battle like
                    // Invisible does - it starts normally, but as an
                    // ambush: forced first turn + 3x damage on the
                    // opening action (see Battle::sneak_attack /
                    // screens/battle.rs's battle_tick Attack handling). Stealth
                    // breaks the instant it's used this way, same as any
                    // other consumed status.
                    let player_is_stealthed = <(Entity, &Stealthed)>::query()
                        .iter(ecs)
                        .any(|(e, _)| *e == player_entity);

                    let new_battle = Battle::new(player_entity, roster);
                    *battle = Some(if player_is_stealthed {
                        commands.remove_component::<Stealthed>(player_entity);
                        new_battle.as_sneak_attack()
                    } else {
                        new_battle
                    });
                    *turn_state = TurnState::InBattle;
                    return;
                }
            } else {
                commands.push((
                    (),
                    WantsToMove {
                        entity: player_entity,
                        destination,
                    },
                ));
            }
        };

        if did_something {
            *turn_state = TurnState::PlayerTurn;
        }
    }
}

/// Buys whichever shop item is on the player's own tile or directly
/// (orthogonally) adjacent to it - the manual counterpart to
/// movement.rs's auto-pickup, enabled only while ShoppingActive
/// suppresses that automatic version (see player_input's Return match
/// arm above). Checks the item's Price against the player's own Gold
/// before granting anything - insufficient funds sets `shop_message`
/// instead of completing the purchase, and neither the stock nor the
/// player's gold changes at all in that case. Returns true only for an
/// actual completed purchase - callers use this to decide whether the
/// dungeon turn should advance at all (see player_input's did_something),
/// since nothing really happened on a failed/no-op attempt.
///
/// Shop items are ShopStock counter markers, not real Items sitting on
/// the floor (see spawner::spawn_shop_stock_at) - the counter row is a
/// Wall tile (MapBuilder::new_arena_shop), so the player can never
/// actually stand ON one, only in the walkable row directly below it.
/// That structurally guarantees the ADJACENT check below only ever
/// matches the one item directly in front of the player, not a neighbor
/// one column over - a real bug in an earlier version of this function,
/// back when items sat on walkable floor tiles a player could stand on
/// or slip between.
fn buy_nearby_item(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    shop_message: &mut Option<ShopMessage>,
) -> bool {
    let player = <(Entity, &Point)>::query()
        .iter(ecs)
        .find_map(|(entity, pos)| Some((*entity, *pos)));
    let (player_entity, player_pos) = match player {
        Some(p) => p,
        None => return false,
    };

    const ADJACENT: [Point; 5] = [
        Point { x: 0, y: 0 },
        Point { x: 0, y: -1 },
        Point { x: 0, y: 1 },
        Point { x: -1, y: 0 },
        Point { x: 1, y: 0 },
    ];

    let found = <(Entity, &ShopStock, &Point, &Name, &Price)>::query()
        .iter(ecs)
        .filter(|(_, _, &pos, _, _)| ADJACENT.iter().any(|&d| pos == player_pos + d))
        .map(|(e, stock, _, name, price)| (*e, stock.0, name.0.clone(), price.0))
        .next();

    let (stock_entity, remaining, name, price) = match found {
        Some(f) => f,
        None => return false,
    };

    let current_gold = ecs
        .entry_ref(player_entity)
        .ok()
        .and_then(|e| e.get_component::<Gold>().ok().copied());

    let affordable = match current_gold {
        Some(Gold(amount)) => amount >= price,
        // No Gold component at all shouldn't happen while ShoppingActive
        // (only reachable in the Battle Arena, where every player has
        // one from start_arena onward), but treat it as "can't afford
        // it" rather than granting a free item if it ever did.
        None => false,
    };

    if !affordable {
        *shop_message = Some(ShopMessage("Not enough gold!".to_string()));
        return false;
    }

    if let Some(Gold(amount)) = current_gold {
        commands.add_component(player_entity, Gold(amount - price));
    }

    let granted_weapon = item_is_weapon(&name);
    spawn_named_item_via_commands(&name, player_entity, commands);

    // Same one-equipped-weapon-at-a-time rule auto-pickup enforces -
    // buying a new weapon discards whatever was previously carried.
    // Checked against components already in the world, not the copy
    // just queued above - CommandBuffer edits aren't visible until
    // flush, same deferred-command reasoning movement.rs relies on.
    if granted_weapon {
        <(Entity, &Carried, &Weapon)>::query()
            .iter(ecs)
            .filter(|(_, c, _)| c.0 == player_entity)
            .for_each(|(e, _, _)| {
                commands.remove(*e);
            });
    }

    if remaining <= 1 {
        // Last one - the counter marker disappears entirely rather
        // than sitting there advertising "0 remaining".
        commands.remove(stock_entity);
    } else {
        commands.add_component(stock_entity, ShopStock(remaining - 1));
    }

    true
}

/// Queues an ActivateItem for the item in usable_item_slots' slot `n`
/// (see that function's doc comment for the fixed-identity Potion/Map
/// layout), if one is actually there. Returns true only when a real item
/// was found and queued - callers use this to decide whether the dungeon
/// turn should advance at all (see player_input's did_something), since
/// pressing a number key over an empty slot didn't actually do anything.
fn use_item(n: usize, ecs: &mut SubWorld, commands: &mut CommandBuffer) -> bool {
    let player_entity = <(Entity, &Player)>::query()
        .iter(ecs)
        .find_map(|(entity, _player)| Some(*entity))
        .unwrap();

    // usable_item_slots reserves key 1 for the potion slot and key 2 for
    // the map slot by fixed identity - if either isn't carried, that slot
    // is None and this key does nothing, rather than the next item
    // sliding up to take its place.
    let item_entity = usable_item_slots(ecs, player_entity)
        .get(n)
        .and_then(|slot| slot.as_ref())
        .map(|(_, _, entity)| *entity);

    match item_entity {
        Some(item_entity) => {
            commands.push((
                (),
                ActivateItem {
                    used_by: player_entity,
                    item: item_entity,
                },
            ));
            true
        }
        None => false,
    }
}
