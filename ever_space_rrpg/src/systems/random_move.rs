use crate::prelude::*;

#[system]
#[read_component(Point)]
#[read_component(MovingRandomly)]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Name)]
#[read_component(Invisible)]
pub fn random_move(
    #[resource] turn_state: &mut TurnState,
    #[resource] battle: &mut Option<Battle>,
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
) {
    // While Invisible, a wandering monster that happens to step onto the
    // player's tile can't start a battle either - see chasing.rs for the
    // (more common) chase-based case. Movement onto that tile is still
    // blocked below (attacked stays true), same as bumping anything else.
    let player_is_invisible = <(Entity, &Invisible)>::query()
        .filter(component::<Player>())
        .iter(ecs)
        .nth(0)
        .is_some();

    let mut movers = <(Entity, &Point, &MovingRandomly)>::query();
    let mut positions = <(Entity, &Point, &Health)>::query();
    movers.iter(ecs).for_each(|(entity, pos, _)| {
        // Once a battle has been queued this turn, let the rest of this
        // turn's movers wait - they'll get another chance next monster turn.
        if battle.is_some() {
            return;
        }
        let mut rng = RandomNumberGenerator::new();
        let destination = match rng.range(0, 4) {
            0 => Point::new(-1, 0),
            1 => Point::new(1, 0),
            2 => Point::new(0, -1),
            _ => Point::new(0, 1),
        } + *pos;

        let mut attacked = false;
        positions
            .iter(ecs)
            .filter(|(_, target_pos, _)| **target_pos == destination)
            .for_each(|(victim, _, _)| {
                if !player_is_invisible
                    && ecs
                        .entry_ref(*victim)
                        .unwrap()
                        .get_component::<Player>()
                        .is_ok()
                {
                    let enemy_name = ecs
                        .entry_ref(*entity)
                        .ok()
                        .and_then(|e| e.get_component::<Name>().ok().cloned())
                        .map(|n| n.0)
                        .unwrap_or_else(|| "the enemy".to_string());

                    *battle = Some(Battle::new(*victim, *entity, enemy_name));
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
    });
}
