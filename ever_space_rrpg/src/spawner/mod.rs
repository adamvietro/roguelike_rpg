use crate::prelude::*;
mod template;
use template::Templates;

pub fn spawn_player(ecs: &mut World, pos: Point) {
    let mut commands = legion::systems::CommandBuffer::new(ecs);
    let player = commands.push((
        Player { map_level: 0 },
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('@'),
        },
        Health {
            current: 10,
            max: 10,
        },
        FieldOfView::new(8),
        Damage(1),
    ));
    commands.add_component(player, CanAttack);
    commands.add_component(player, CanDefend);
    commands.add_component(player, CanFlee);
    commands.add_component(player, Speed(6));
    commands.add_component(player, Class("Barbarian".to_string()));
    commands.flush(ecs);
}

pub fn spawn_level(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    level: usize,
    spawn_points: &[Point],
) {
    let template = Templates::load();
    template.spawn_entities(ecs, rng, level, spawn_points);
}

/// Rolls a chance to grant the player a random one-time battle item after
/// a battle victory, filtered to items matching the player's own Class.
/// Returns the granted item's display name, if any.
pub fn grant_random_battle_loot(
    ecs: &mut World,
    rng: &mut RandomNumberGenerator,
    player: Entity,
) -> Option<String> {
    let player_class = entity_class(ecs, player)?;
    let template = Templates::load();
    template.grant_random_battle_loot(ecs, rng, player, &player_class)
}

pub fn spawn_amulet_of_yala(ecs: &mut World, pos: Point) {
    ecs.push((
        Item,
        AmuletOfYala,
        pos,
        Render {
            color: ColorPair::new(WHITE, BLACK),
            glyph: to_cp437('|'),
        },
        Name("Amulet of Yala".to_string()),
    ));
}
