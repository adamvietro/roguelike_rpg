use crate::prelude::*;

/// Checks every enemy's current position against every placed Trap (see
/// components::Trap / ProvidesEffect::PlaceTrap in systems/use_items.rs) -
/// if an enemy is standing on a trap's tile after this turn's movement, it
/// takes the trap's damage and the trap is consumed (single-use). Runs
/// once per monster turn, right after movement_system, so it sees each
/// enemy's freshly-updated Point - see systems/mod.rs's
/// build_monster_scheduler. The player is never checked against traps -
/// only entities with an Enemy component.
///
/// Damage here is Defense-aware, mirroring battle::apply_damage's logic -
/// that function can't be called directly from a #[system], since it
/// takes &mut World rather than the restricted &mut SubWorld a system
/// gets, so the same "amount minus Defense, floored at 0" calculation is
/// duplicated here rather than shared.
#[system]
#[read_component(Point)]
#[read_component(Enemy)]
#[read_component(Trap)]
#[read_component(Defense)]
#[write_component(Health)]
pub fn traps(ecs: &mut SubWorld, commands: &mut CommandBuffer) {
    let trap_positions: Vec<(Entity, Point, i32)> = <(Entity, &Point, &Trap)>::query()
        .iter(ecs)
        .map(|(e, pos, trap)| (*e, *pos, trap.damage))
        .collect();
    if trap_positions.is_empty() {
        return;
    }

    let enemy_positions: Vec<(Entity, Point)> = <(Entity, &Point)>::query()
        .filter(component::<Enemy>())
        .iter(ecs)
        .map(|(e, pos)| (*e, *pos))
        .collect();

    for (enemy, epos) in enemy_positions {
        if let Some((trap_entity, _, damage)) =
            trap_positions.iter().find(|(_, tpos, _)| *tpos == epos)
        {
            if let Ok(mut entry) = ecs.entry_mut(enemy) {
                let defense = entry.get_component::<Defense>().map_or(0, |d| d.0);
                let actual = (*damage - defense).max(0);
                let died = if let Ok(health) = entry.get_component_mut::<Health>() {
                    health.current -= actual;
                    health.current < 1
                } else {
                    false
                };
                if died {
                    commands.remove(enemy);
                }
            }
            // Single-use, whether or not the enemy actually had Health to
            // damage (shouldn't happen in practice, but a trap that failed
            // to find a target shouldn't linger and double-trigger later).
            commands.remove(*trap_entity);
        }
    }
}
