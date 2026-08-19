use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(BattleItem)]
pub fn hud(ecs: &SubWorld) {
    let mut health_query = <&Health>::query().filter(component::<Player>());
    let player_health = health_query.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(2);
    draw_batch.print_centered(1, "Explore the Dungeon. Cursor keys to move.");
    draw_batch.bar_horizontal(
        Point::zero(),
        SCREEN_WIDTH * 2,
        player_health.current,
        player_health.max,
        ColorPair::new(RED, BLACK),
    );
    draw_batch.print_color_centered(
        0,
        format!(
            " Health: {} / {} ",
            player_health.current, player_health.max
        ),
        ColorPair::new(WHITE, RED),
    );

    let (player, map_level) = <(Entity, &Player)>::query() // (1)
        .iter(ecs)
        .find_map(|(entity, player)| Some((*entity, player.map_level)))
        .unwrap();

    draw_batch.print_color_right(
        // (2)
        Point::new(SCREEN_WIDTH * 2, 1),
        format!("Dungeon Level: {}", map_level + 1), // (3)
        ColorPair::new(YELLOW, BLACK),
    );

    // Battle attacks (Deathblow, Garrote, etc.) get their own count so we
    // can both exclude them from the regular item list below and list them
    // separately, grouped by name with a count, on the right.
    let battle_item_ids: Vec<Entity> = <(Entity, &BattleItem)>::query()
        .iter(ecs)
        .map(|(e, _)| *e)
        .collect();

    let mut item_query = <(Entity, &Item, &Name, &Carried)>::query();
    let mut y = 3;
    item_query
        .iter(ecs)
        .filter(|(_, _, _, carried)| carried.0 == player)
        .filter(|(entity, _, _, _)| !battle_item_ids.contains(entity))
        .for_each(|(_, _, name, _)| {
            draw_batch.print(Point::new(3, y), format!("{} : {}", y - 2, &name.0));
            y += 1;
        });
    if y > 3 {
        draw_batch.print_color(
            Point::new(3, 2),
            "Items carried",
            ColorPair::new(YELLOW, BLACK),
        );
    }

    let mut battle_item_query = <(&Carried, &BattleItem, &Name)>::query();
    let mut battle_item_counts: Vec<(String, i32)> = Vec::new();
    battle_item_query
        .iter(ecs)
        .filter(|(carried, _, _)| carried.0 == player)
        .for_each(
            |(_, _, name)| match battle_item_counts.iter_mut().find(|(n, _)| *n == name.0) {
                Some(entry) => entry.1 += 1,
                None => battle_item_counts.push((name.0.clone(), 1)),
            },
        );
    let mut y2 = 3;
    for (name, count) in battle_item_counts.iter() {
        draw_batch.print_color_right(
            Point::new(SCREEN_WIDTH * 2, y2),
            format!("{} x{}", name, count),
            ColorPair::new(GREEN, BLACK),
        );
        y2 += 1;
    }
    if y2 > 3 {
        draw_batch.print_color_right(
            Point::new(SCREEN_WIDTH * 2, 2),
            "Battle Attacks",
            ColorPair::new(YELLOW, BLACK),
        );
    }

    draw_batch.submit(10000).expect("Batch error");
}
