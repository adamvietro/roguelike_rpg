use crate::prelude::*;

#[system]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(BattleItem)]
#[read_component(Weapon)]
#[read_component(Description)]
pub fn hud(ecs: &SubWorld, #[resource] hud_mouse_pos: &HudMousePos) {
    let mut health_query = <&Health>::query().filter(component::<Player>());
    let player_health = health_query.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(4);
    draw_batch.print_centered(1, "Explore the Dungeon. Cursor keys to move.");
    draw_batch.bar_horizontal(
        Point::zero(),
        HUD_COLS,
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
        Point::new(HUD_COLS, 1),
        format!("Dungeon Level: {}", map_level + 1), // (3)
        ColorPair::new(YELLOW, BLACK),
    );

    // Items carried, usable via number keys - left side. Draws from the
    // same usable_carried_items list player_input's use_item consumes, so
    // the displayed numbering always matches what pressing that number
    // actually activates (weapons and battle attacks are excluded - they
    // get their own panels below/right instead).
    let mut y = 3;
    for item in usable_carried_items(ecs, player) {
        if let Ok(entry) = ecs.entry_ref(item) {
            if let Ok(name) = entry.get_component::<Name>() {
                draw_batch.print(Point::new(3, y), format!("{} : {}", y - 2, &name.0));
                y += 1;
            }
        }
    }
    if y > 3 {
        draw_batch.print_color(
            Point::new(3, 2),
            "Items carried",
            ColorPair::new(YELLOW, BLACK),
        );
    }

    // Battle attacks, grouped by name with a count - right side, upper.
    // Keeps one representative entity per group so a hovered row can look
    // up that item's Description.
    let mut battle_item_query = <(Entity, &Carried, &BattleItem, &Name)>::query();
    let mut battle_item_groups: Vec<(String, i32, Entity)> = Vec::new();
    battle_item_query
        .iter(ecs)
        .filter(|(_, carried, _, _)| carried.0 == player)
        .for_each(|(entity, _, _, name)| {
            match battle_item_groups.iter_mut().find(|(n, _, _)| *n == name.0) {
                Some(entry) => entry.1 += 1,
                None => battle_item_groups.push((name.0.clone(), 1, *entity)),
            }
        });

    // Precise mouse position in this console's own coordinates (captured
    // with console 4 active - see main.rs::tick), used to detect hovering
    // a Battle Attacks row for its tooltip.
    let hud_mouse = hud_mouse_pos.0;
    let mut hovered: Option<(i32, String, String)> = None;

    let mut y2 = 3;
    for (name, count, entity) in battle_item_groups.iter() {
        draw_batch.print_color_right(
            Point::new(HUD_COLS, y2),
            format!("{} x{}", name, count),
            ColorPair::new(GREEN, BLACK),
        );
        // Loose hit-test: same row, and generally over the right-side
        // panel area (not the left item list, which shares row numbers).
        if hud_mouse.y == y2 && hud_mouse.x >= HUD_COLS - 30 {
            if let Ok(entry) = ecs.entry_ref(*entity) {
                if let Ok(desc) = entry.get_component::<Description>() {
                    hovered = Some((y2, name.clone(), desc.0.clone()));
                }
            }
        }
        y2 += 1;
    }
    if y2 > 3 {
        draw_batch.print_color_right(
            Point::new(HUD_COLS, 2),
            "Battle Attacks",
            ColorPair::new(YELLOW, BLACK),
        );
        y2 += 1; // blank row before the weapons section
    }

    // Weapons, grouped by name with a count - right side, below battle
    // attacks. Weapons apply automatically (see carried_weapon_damage),
    // so they're informational only, not number-key-usable - hence their
    // own panel rather than the left list.
    let mut weapon_query = <(&Carried, &Weapon, &Name)>::query();
    let mut weapon_counts: Vec<(String, i32)> = Vec::new();
    weapon_query
        .iter(ecs)
        .filter(|(carried, _, _)| carried.0 == player)
        .for_each(
            |(_, _, name)| match weapon_counts.iter_mut().find(|(n, _)| *n == name.0) {
                Some(entry) => entry.1 += 1,
                None => weapon_counts.push((name.0.clone(), 1)),
            },
        );
    if !weapon_counts.is_empty() {
        draw_batch.print_color_right(
            Point::new(HUD_COLS, y2),
            "Weapons",
            ColorPair::new(YELLOW, BLACK),
        );
        y2 += 1;
        for (name, count) in weapon_counts.iter() {
            draw_batch.print_color_right(
                Point::new(HUD_COLS, y2),
                format!("{} x{}", name, count),
                ColorPair::new(GREEN, BLACK),
            );
            y2 += 1;
        }
    }

    // Battle Attacks tooltip: drawn last so it sits on top, positioned to
    // the left of the panel on the same row as the hovered entry.
    if let Some((row, name, description)) = hovered {
        draw_batch.print_color(
            Point::new(25, row),
            format!("{}: {}", name, description),
            ColorPair::new(WHITE, BLACK),
        );
    }

    draw_batch.submit(10000).expect("Batch error");
}
