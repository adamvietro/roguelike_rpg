use crate::prelude::*;

#[system]
#[read_component(ActivateItem)]
#[read_component(Effect)]
#[read_component(Point)]
#[read_component(FieldOfView)]
#[read_component(Enemy)]
#[read_component(Damage)]
#[read_component(Carried)]
#[read_component(Defense)]
#[write_component(Health)]
pub fn use_items(
    ecs: &mut SubWorld,
    commands: &mut CommandBuffer,
    #[resource] map: &mut Map,
    #[resource] turn_state: &mut TurnState,
) {
    let mut healing_to_apply = Vec::<(Entity, i32)>::new();
    // Amazon's Throw Spear - collected here (during the read-only pass
    // over ActivateItem/Effect) and applied afterward, same two-phase
    // reasoning as healing_to_apply: the query iterator below holds an
    // immutable borrow of `ecs`, so any Health mutation has to wait until
    // it's dropped.
    let mut ranged_strikes_to_apply = Vec::<(Entity, i32)>::new();
    <(Entity, &ActivateItem)>::query()
        .iter(ecs)
        .for_each(|(entity, activate)| {
            // (1)

            let item = ecs.entry_ref(activate.item); // (2)
            if let Ok(item) = item {
                // (3)
                if let Ok(effect) = item.get_component::<Effect>() {
                    // (4)
                    match effect.0 {
                        ProvidesEffect::Healing(amount) => {
                            healing_to_apply.push((activate.used_by, amount)); // (5)
                        }
                        ProvidesEffect::MagicMap => {
                            map.revealed_tiles.iter_mut().for_each(|t| *t = true);
                            // (7)
                        }
                        // Invisible Cloak - grants Invisible for `moves` turns.
                        // Using another cloak while already Invisible just
                        // overwrites it with a fresh count, matching how other
                        // refreshable timed effects in this project already
                        // behave (see battle::DotState).
                        ProvidesEffect::Invisibility(moves) => {
                            commands.add_component(
                                activate.used_by,
                                Invisible {
                                    moves_remaining: moves,
                                },
                            );
                        }
                        // Stealth (Rogue) - grants Stealthed for `moves`
                        // turns. Unlike Invisibility, this does NOT block
                        // a bump-into-enemy from starting a battle - see
                        // components::Stealthed / systems/player_input.rs.
                        // Same refresh-on-reuse behavior as Invisibility.
                        ProvidesEffect::Stealth(moves) => {
                            commands.add_component(
                                activate.used_by,
                                Stealthed {
                                    moves_remaining: moves,
                                },
                            );
                        }
                        // Ice Armor - "Mages buff before battle." Applied
                        // immediately here, out of combat, as a persistent
                        // status that survives into (and across) whichever
                        // battle it's used in - see components::IceArmored and
                        // battle::resolve_enemy_attack. Using another Ice
                        // Armor while one is already active just overwrites it
                        // with a fresh count, same refresh behavior as Invisible.
                        ProvidesEffect::IceArmor {
                            defense_bonus,
                            attacks,
                        } => {
                            commands.add_component(
                                activate.used_by,
                                IceArmored {
                                    defense_bonus,
                                    attacks_remaining: attacks,
                                },
                            );
                        }
                        // Debug class only - see components::ProvidesEffect.
                        // Just sets TurnState directly; main.rs's tick
                        // dispatcher already handles all three states
                        // (victory/game_over/advance_level) exactly as it
                        // would if reached the normal way, so nothing else
                        // needs to know these were triggered by an item
                        // instead of real gameplay.
                        ProvidesEffect::DebugWin => {
                            *turn_state = TurnState::Victory;
                        }
                        ProvidesEffect::DebugLose => {
                            *turn_state = TurnState::GameOver;
                        }
                        ProvidesEffect::DebugNextLevel => {
                            *turn_state = TurnState::NextLevel;
                        }
                        // Amazon's Throw Spear - finds the nearest enemy
                        // currently within the user's FieldOfView (real
                        // line-of-sight via shadowcasting, not just
                        // distance - see systems/fov.rs), and queues it to
                        // take the user's normal attack damage plus
                        // `bonus`. Does nothing if no enemy is visible
                        // (still consumed either way, same as every other
                        // item that finds nothing to affect - e.g. a
                        // Healing Potion at full HP).
                        ProvidesEffect::RangedStrike(bonus) => {
                            if let Ok(user) = ecs.entry_ref(activate.used_by) {
                                let user_pos = user.get_component::<Point>().ok().copied();
                                let fov = user.get_component::<FieldOfView>().ok().cloned();
                                if let (Some(user_pos), Some(fov)) = (user_pos, fov) {
                                    let mut nearest: Option<(Entity, f32)> = None;
                                    <(Entity, &Point)>::query()
                                        .filter(component::<Enemy>())
                                        .iter(ecs)
                                        .for_each(|(e, pos)| {
                                            if !fov.visible_tiles.contains(pos) {
                                                return;
                                            }
                                            let dist =
                                                DistanceAlg::Pythagoras.distance2d(user_pos, *pos);
                                            if nearest.map_or(true, |(_, best)| dist < best) {
                                                nearest = Some((*e, dist));
                                            }
                                        });
                                    if let Some((target, _)) = nearest {
                                        let base = <(Entity, &Damage)>::query()
                                            .iter(ecs)
                                            .find(|(e, _)| **e == activate.used_by)
                                            .map_or(0, |(_, d)| d.0);
                                        let weapon: i32 = <(&Carried, &Damage)>::query()
                                            .iter(ecs)
                                            .filter(|(carried, _)| carried.0 == activate.used_by)
                                            .map(|(_, dmg)| dmg.0)
                                            .sum();
                                        ranged_strikes_to_apply
                                            .push((target, base + weapon + bonus));
                                    }
                                }
                            }
                        }
                        // Amazon's Trap - places a Trap entity (see
                        // components::Trap / systems/traps.rs) at the
                        // user's current position. Pure spawn via
                        // CommandBuffer, so (unlike RangedStrike) this
                        // doesn't need the deferred second pass.
                        ProvidesEffect::PlaceTrap(damage) => {
                            if let Ok(user) = ecs.entry_ref(activate.used_by) {
                                if let Ok(&pos) = user.get_component::<Point>() {
                                    commands.push((
                                        pos,
                                        Trap { damage },
                                        Render {
                                            color: ColorPair::new(RED, BLACK),
                                            glyph: to_cp437('T'),
                                        },
                                    ));
                                }
                            }
                        }
                        // Hunter's Freeze Trap - same spawn-at-current-
                        // position shape as PlaceTrap above, but a
                        // FreezeTrap entity (components::FreezeTrap)
                        // instead of a damage Trap, and colored BLUE
                        // rather than RED so the two are distinguishable
                        // at a glance on the ground even though they
                        // share the same 'T' glyph.
                        ProvidesEffect::PlaceFreezeTrap(turns) => {
                            if let Ok(user) = ecs.entry_ref(activate.used_by) {
                                if let Ok(&pos) = user.get_component::<Point>() {
                                    commands.push((
                                        pos,
                                        FreezeTrap { turns },
                                        Render {
                                            color: ColorPair::new(BLUE, BLACK),
                                            glyph: to_cp437('T'),
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
            }

            commands.remove(activate.item); // (8)
            commands.remove(*entity); // (9)
        });

    for heal in healing_to_apply.iter() {
        if let Ok(mut target) = ecs.entry_mut(heal.0) {
            // (10)
            if let Ok(health) = target.get_component_mut::<Health>() {
                // (11)
                health.current = i32::min(health.max, health.current + heal.1); // (12)
            }
        }
    }

    // Amazon's Throw Spear damage, applied here for the same reason
    // healing is: Health needs a mutable borrow the read-only query loop
    // above couldn't hold at the same time. Defense-aware, mirroring
    // battle::apply_damage's logic (which can't be called directly here -
    // it takes &mut World, not the restricted &mut SubWorld a #[system]
    // gets).
    for (target, amount) in ranged_strikes_to_apply.iter() {
        if let Ok(mut entry) = ecs.entry_mut(*target) {
            let defense = entry.get_component::<Defense>().map_or(0, |d| d.0);
            let actual = (*amount - defense).max(0);
            let died = if let Ok(health) = entry.get_component_mut::<Health>() {
                health.current -= actual;
                health.current < 1
            } else {
                false
            };
            if died {
                commands.remove(*target);
            }
        }
    }
}
