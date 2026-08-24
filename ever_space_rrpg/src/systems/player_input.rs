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
pub fn player_input(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] key: &Option<VirtualKeyCode>,
    #[resource] turn_state: &mut TurnState,
    #[resource] battle: &mut Option<Battle>,
) {
    let mut players = <(Entity, &Point)>::query().filter(component::<Player>());
    let mut enemies = <(Entity, &Point)>::query().filter(component::<Enemy>());

    if let Some(key) = *key {
        // Checked first and returns immediately - Escape never falls
        // through to the movement/attack logic below, and this system
        // only ever runs during TurnState::AwaitingInput (dungeon
        // exploration), so pausing mid-battle isn't reachable: battle_tick
        // handles its own keys entirely separately and never calls this.
        if key == VirtualKeyCode::Escape {
            *turn_state = TurnState::Paused;
            return;
        }

        let delta = match key {
            VirtualKeyCode::Left => Point::new(-1, 0),
            VirtualKeyCode::Right => Point::new(1, 0),
            VirtualKeyCode::Up => Point::new(0, -1),
            VirtualKeyCode::Down => Point::new(0, 1),
            VirtualKeyCode::G => {
                let (player, player_pos) = players
                    .iter(ecs)
                    .find_map(|(entity, pos)| Some((*entity, *pos)))
                    .unwrap();

                let mut items = <(Entity, &Item, &Point)>::query();
                items
                    .iter(ecs)
                    .filter(|(_entity, _item, &item_pos)| item_pos == player_pos)
                    .for_each(|(entity, _item, _item_pos)| {
                        commands.remove_component::<Point>(*entity);
                        commands.add_component(*entity, Carried(player));

                        if let Ok(e) = ecs.entry_ref(*entity) {
                            if e.get_component::<Weapon>().is_ok() {
                                // (1)
                                <(Entity, &Carried, &Weapon)>::query()
                                    .iter(ecs)
                                    .filter(|(_, c, _)| c.0 == player)
                                    .for_each(|(e, _c, _w)| {
                                        commands.remove(*e); // (2)
                                    })
                            }
                        }
                    });
                Point::new(0, 0)
            }
            VirtualKeyCode::Key1 => use_item(0, ecs, commands),
            VirtualKeyCode::Key2 => use_item(1, ecs, commands),
            VirtualKeyCode::Key3 => use_item(2, ecs, commands),
            VirtualKeyCode::Key4 => use_item(3, ecs, commands),
            VirtualKeyCode::Key5 => use_item(4, ecs, commands),
            VirtualKeyCode::Key6 => use_item(5, ecs, commands),
            VirtualKeyCode::Key7 => use_item(6, ecs, commands),
            VirtualKeyCode::Key8 => use_item(7, ecs, commands),
            VirtualKeyCode::Key9 => use_item(8, ecs, commands),
            VirtualKeyCode::Key0 => use_item(9, ecs, commands),
            _ => Point::new(0, 0),
        };

        let (player_entity, destination) = players
            .iter(ecs)
            .find_map(|(entity, pos)| Some((*entity, *pos + delta)))
            .unwrap();

        // let mut did_something = false;
        if delta.x != 0 || delta.y != 0 {
            let attacked_enemy = enemies
                .iter(ecs)
                .find(|(_, pos)| **pos == destination)
                .map(|(entity, _)| *entity);

            if let Some(enemy_entity) = attacked_enemy {
                // While Invisible (see components::Invisible / the
                // Invisible Cloak item), walking into an enemy is blocked
                // like a wall instead of starting a battle - no
                // WantsToMove is queued either, so the player doesn't step
                // onto that tile. The turn still passes via the
                // unconditional TurnState::PlayerTurn below, same as
                // bumping a real wall does (movement.rs silently drops an
                // invalid destination but still consumes the move).
                let player_is_invisible = <(Entity, &Invisible)>::query()
                    .iter(ecs)
                    .any(|(e, _)| *e == player_entity);

                if !player_is_invisible {
                    let enemy_name = ecs
                        .entry_ref(enemy_entity)
                        .ok()
                        .and_then(|e| e.get_component::<Name>().ok().cloned())
                        .map(|n| n.0)
                        .unwrap_or_else(|| "the enemy".to_string());

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

                    let new_battle = Battle::new(player_entity, enemy_entity, enemy_name);
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
                // did_something = true;
                commands.push((
                    (),
                    WantsToMove {
                        entity: player_entity,
                        destination,
                    },
                ));
            }
        };
        *turn_state = TurnState::PlayerTurn;
    }
}

fn use_item(n: usize, ecs: &mut SubWorld, commands: &mut CommandBuffer) -> Point {
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

    if let Some(item_entity) = item_entity {
        commands.push((
            (),
            ActivateItem {
                used_by: player_entity,
                item: item_entity,
            },
        ));
    }

    Point::zero()
}
