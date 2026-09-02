use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(ChasingPlayer)]
#[read_component(FieldOfView)]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Enemy)]
#[read_component(Name)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(Frozen)]
pub fn chasing(
    #[resource] map: &Map,
    #[resource] turn_state: &mut TurnState,
    #[resource] battle: &mut Option<Battle>,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    // While Invisible OR Stealthed, no enemy can "see" the player at all -
    // they don't chase, don't reposition toward the player's last known
    // tile, and (further down) can't ambush the player by wandering into
    // their tile either. This is a full early-out, not just skipping the
    // eventual battle trigger, since "can they see me" is the actual ask -
    // a monster that can't see you shouldn't move toward you either.
    // Stealth and Invisible differ elsewhere (the PLAYER walking into an
    // enemy while Stealthed still starts a battle, as an ambush - see
    // systems/player_input.rs) but they're identical here: an enemy that
    // can't detect the player can't initiate anything regardless of which
    // status is active.
    let player_is_hidden = <(Entity, Option<&Invisible>, Option<&Stealthed>)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .map_or(false, |(_, invisible, stealthed)| {
            invisible.is_some() || stealthed.is_some()
        });
    if player_is_hidden {
        return;
    }

    let mut movers = <(
        Entity,
        &Point,
        &ChasingPlayer,
        &FieldOfView,
        Option<&Frozen>,
    )>::query();
    let mut positions = <(Entity, &Point, &Health)>::query();
    let mut enemies_at = <(Entity, &Point)>::query().filter(component::<Enemy>());
    let mut player = <(&Point, &Player)>::query();
    let player_pos = player.iter(ecs).nth(0).unwrap().0;
    let player_idx = map_idx(player_pos.x, player_pos.y);

    let search_targets = vec![player_idx];
    let dijkstra_map = DijkstraMap::new(SCREEN_WIDTH, SCREEN_HEIGHT, &search_targets, map, 1024.0);

    movers.iter(ecs).for_each(|(entity, pos, _, fov, frozen)| {
        // Frozen (Hunter's Freeze Trap - see components::Frozen) is a
        // full early-out, same shape as player_is_hidden above: a
        // frozen enemy doesn't reposition toward the player AND can't
        // bump into them to start a battle either, since both go
        // through this same movement path.
        if frozen.is_some() {
            return;
        }
        if !fov.visible_tiles.contains(&player_pos) {
            return;
        }
        // Once a battle has been queued this turn, let the rest of this
        // turn's movers wait - they'll get another chance next monster turn.
        if battle.is_some() {
            return;
        }
        let idx = map_idx(pos.x, pos.y);
        if let Some(destination) = DijkstraMap::find_lowest_exit(&dijkstra_map, idx, map) {
            let distance = DistanceAlg::Pythagoras.distance2d(*pos, *player_pos);
            let destination = if distance > 1.2 {
                map.index_to_point2d(destination)
            } else {
                *player_pos
            };

            let mut attacked = false;
            positions
                .iter(ecs)
                .filter(|(_, target_pos, _)| **target_pos == destination)
                .for_each(|(victim, _, _)| {
                    if ecs
                        .entry_ref(*victim)
                        .unwrap()
                        .get_component::<Player>()
                        .is_ok()
                    {
                        // Every enemy already AT the player's tile joins
                        // too (not just this mover) - see
                        // player_input.rs's own roster-gathering comment
                        // for the same rule applied from the other
                        // direction. `entity` (this mover) hasn't
                        // actually moved onto `destination` in the ECS
                        // yet, so it has to be included explicitly
                        // rather than found by the position query below.
                        let mut roster: Vec<(Entity, String)> = vec![(
                            *entity,
                            ecs.entry_ref(*entity)
                                .ok()
                                .and_then(|e| e.get_component::<Name>().ok().cloned())
                                .map(|n| n.0)
                                .unwrap_or_else(|| "the enemy".to_string()),
                        )];
                        roster.extend(
                            enemies_at
                                .iter(ecs)
                                .filter(|(e, pos)| *e != entity && **pos == destination)
                                .map(|(e, _)| {
                                    let name = ecs
                                        .entry_ref(*e)
                                        .ok()
                                        .and_then(|er| er.get_component::<Name>().ok().cloned())
                                        .map(|n| n.0)
                                        .unwrap_or_else(|| "the enemy".to_string());
                                    (*e, name)
                                }),
                        );
                        roster.truncate(MAX_BATTLE_ENEMIES);

                        *battle = Some(Battle::new(*victim, roster));
                        *turn_state = TurnState::InBattle;
                    }
                    attacked = true;
                });

            if !attacked {
                commands.push((
                    (),
                    WantsToMove {
                        entity: *entity,
                        destination,
                    },
                ));
            }
        }
    });
}
