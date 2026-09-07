use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Point)]
#[read_component(Player)]
#[read_component(AmuletOfYala)]
#[read_component(Invisible)]
#[read_component(Stealthed)]
#[read_component(Frozen)]
#[read_component(Enemy)]
pub fn end_turn(
    ecs: &SubWorld,
    commands: &mut CommandBuffer,
    #[resource] turn_state: &mut TurnState,
    #[resource] map: &Map,
    #[resource] arena_run: &Option<ArenaRun>,
    #[resource] shopping: &Option<ShoppingActive>,
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

        // Same countdown for Stealth (Rogue) - see components::Stealthed.
        let mut stealthed = <(Entity, &Stealthed)>::query().filter(component::<Player>());
        if let Some((player, status)) = stealthed.iter(ecs).nth(0) {
            if status.moves_remaining <= 1 {
                commands.remove_component::<Stealthed>(*player);
            } else {
                commands.add_component(
                    *player,
                    Stealthed {
                        moves_remaining: status.moves_remaining - 1,
                    },
                );
            }
        }

        // Same countdown for Frozen (Hunter's Freeze Trap) on however
        // many enemies currently carry it - not filtered to the player,
        // unlike Invisible/Stealthed above, since this is an enemy-side
        // status. See components::Frozen / systems/chasing.rs.
        let frozen_updates: Vec<(Entity, i32)> = <(Entity, &Frozen)>::query()
            .iter(ecs)
            .map(|(e, status)| (*e, status.turns_remaining))
            .collect();
        for (entity, turns_remaining) in frozen_updates {
            if turns_remaining <= 1 {
                commands.remove_component::<Frozen>(entity);
            } else {
                commands.add_component(
                    entity,
                    Frozen {
                        turns_remaining: turns_remaining - 1,
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
            // Three-way split, not two: a Dungeon Crawl floor's OWN
            // stairs (ArenaRun absent, not yet ShoppingActive) leads into
            // the between-floor shop; that shop's own stairs (ArenaRun
            // still absent, but ShoppingActive is now Some - see
            // State::dungeon_shop_transition) is what actually generates
            // the next floor. Arena's stairs (inside its own shop) still
            // always mean "begin wave 1", same as before.
            new_state = if arena_run.is_some() {
                TurnState::ArenaTransition
            } else if shopping.is_some() {
                TurnState::NextLevel
            } else {
                TurnState::DungeonShopTransition
            };
        }
    });

    // Catches an Arena kill that happened OUTSIDE the normal battle-
    // victory flow - a ranged strike (Throw Spear/Shoot) or a Trap can
    // kill the map's last enemy without ever opening a BattleVictory
    // screen, so nothing else would notice the wave/boss encounter was
    // actually just cleared. Only checked when an Arena wave/boss fight
    // could genuinely be in progress: ShoppingActive being present means
    // we're browsing a shop instead (zero enemies there is normal, not a
    // "wave cleared" signal), and this deliberately doesn't clobber a
    // more urgent outcome (GameOver/Victory/a real Exit tile) that the
    // checks above may have already set this same tick - new_state is
    // only eligible here if it's still one of the ordinary "keep playing"
    // states.
    if arena_run.is_some()
        && shopping.is_none()
        && matches!(
            new_state,
            TurnState::AwaitingInput | TurnState::PlayerTurn | TurnState::MonsterTurn
        )
        && <&Enemy>::query().iter(ecs).count() == 0
    {
        new_state = TurnState::ArenaWaveCleared;
    }

    *turn_state = new_state;
}
