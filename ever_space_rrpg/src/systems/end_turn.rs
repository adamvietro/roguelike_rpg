use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(AmuletOfYala)]
#[read_component(Invisible)]
pub fn end_turn(
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
    #[resource] turn_state: &mut TurnState,
    #[resource] map: &Map,
) {
    let mut player_hp = <(&Health, &Point)>::query().filter(component::<Player>());
    let mut amulet = <&Point>::query().filter(component::<AmuletOfYala>());
    let current_state = turn_state.clone();
    let mut new_state = match current_state {
        TurnState::AwaitingInput => return,
        TurnState::PlayerTurn => TurnState::MonsterTurn,
        TurnState::MonsterTurn => TurnState::AwaitingInput,
        _ => current_state,
    };

    // A player action just completed - tick any active Invisible Cloak
    // effect down by one of its moves, dropping it once it runs out.
    if current_state == TurnState::PlayerTurn {
        let mut invisible = <(Entity, &Invisible)>::query().filter(component::<Player>());
        if let Some((player, status)) = invisible.iter(ecs).nth(0) {
            if status.moves_remaining <= 1 {
                commands.remove_component::<Invisible>(*player);
            } else {
                commands.add_component(
                    *player,
                    Invisible {
                        moves_remaining: status.moves_remaining - 1,
                    },
                );
            }
        }
    }

    let amulet_default = Point::new(-1, -1);
    let amulet_pos = amulet.iter(ecs).nth(0).unwrap_or(&amulet_default);

    player_hp.iter(ecs).for_each(|(hp, pos)| {
        if hp.current < 1 {
            new_state = TurnState::GameOver;
        }
        if pos == amulet_pos {
            new_state = TurnState::Victory;
        }
        let idx = map.point2d_to_index(*pos);
        if map.tiles[idx] == TileType::Exit {
            new_state = TurnState::NextLevel;
        }
    });

    *turn_state = new_state;
}
