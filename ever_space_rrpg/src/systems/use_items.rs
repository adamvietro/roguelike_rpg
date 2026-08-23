use crate::prelude::*;

#[system]
#[read_component(ActivateItem)]
#[read_component(Effect)]
#[write_component(Health)]
pub fn use_items(ecs: &mut SubWorld, commands: &mut CommandBuffer, #[resource] map: &mut Map) {
    let mut healing_to_apply = Vec::<(Entity, i32)>::new();
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
}
