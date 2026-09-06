use crate::prelude::*;

/// The hotkey label for Ability Bar slot `i` - matches
/// player_input.rs::use_ability's key order exactly (1-9, then 0 for the
/// 10th slot), NOT just "i + 1", which would read "10" for the 10th slot
/// - not a key that exists.
fn ability_bar_key_label(i: usize) -> &'static str {
    const LABELS: [&str; ABILITY_BAR_MAX_SLOTS] =
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
    LABELS.get(i).copied().unwrap_or("")
}

/// Which ABILITY_BAR_CONSOLE row the icons sit on - one full icon-height
/// above the console's very bottom row, so the bar isn't flush against
/// the physical screen edge.
fn ability_bar_row() -> i32 {
    ABILITY_BAR_ROWS - 2
}

/// The leftmost column `n` icons should start at to appear centered as a
/// group on ABILITY_BAR_CONSOLE - e.g. a 2-ability class's icons sit
/// centered in the middle of the screen, not pinned to the left edge the
/// way a longer roster's would naturally reach toward anyway. Integer
/// division rounds a genuinely-odd remainder toward the left rather than
/// perfectly splitting a half-column, which isn't expressible on a
/// whole-cell grid regardless.
fn ability_bar_start_col(n: i32) -> i32 {
    (ABILITY_BAR_COLS - n) / 2
}

/// Converts Ability Bar column `col`'s icon position into a (col, row) on
/// HUD_CONSOLE for its number label - both consoles share the same
/// physical 1280x800 window, so this is a ratio of cell counts, same
/// technique class_select's headline/description split already uses
/// between BIG_TEXT_CONSOLE and HUD_CONSOLE. The label sits one
/// HUD_CONSOLE row above the bar's own top pixel edge (just above each
/// icon, not overlapping it) and aligned to that icon's own LEFT pixel
/// edge (not centered - a single digit is narrow enough that left-
/// aligned still reads as "belonging to" the icon immediately to its
/// right, and it avoids needing to also know the label's own rendered
/// width to center it).
fn ability_bar_label_position(col: i32) -> (i32, i32) {
    let bar_top_px = ability_bar_row() * (800 / ABILITY_BAR_ROWS);
    let label_row = (bar_top_px * HUD_ROWS / 800) - 1;

    let icon_left_px = col * (1280 / ABILITY_BAR_COLS);
    let label_col = icon_left_px * HUD_COLS / 1280;

    (label_col, label_row)
}

/// The (x, y, width, height) box - in HUD_CONSOLE cell terms, for
/// render_helpers::draw_ascii_box, the same ASCII box style the battle
/// menu already uses - that should enclose the whole Ability Bar as one
/// group (icons AND their number labels), for the single surrounding
/// border. `n` is however many slots are actually showing right now.
/// A 1-cell pad on every side keeps the border from touching the icons/
/// labels themselves.
fn ability_bar_box_bounds(n: i32) -> (i32, i32, i32, i32) {
    let start_col = ability_bar_start_col(n);
    let bar_row = ability_bar_row();

    let icons_left_px = start_col * (1280 / ABILITY_BAR_COLS);
    let icons_right_px = (start_col + n) * (1280 / ABILITY_BAR_COLS);
    let icons_bottom_px = (bar_row + 1) * (800 / ABILITY_BAR_ROWS);

    let (_, label_row) = ability_bar_label_position(start_col);

    let left = (icons_left_px * HUD_COLS / 1280) - 1;
    let right = (icons_right_px * HUD_COLS / 1280) + 1;
    let bottom = (icons_bottom_px * HUD_ROWS / 800) + 1;
    let top = label_row - 1;

    (left, top, right - left, bottom - top)
}

/// The HUD_CONSOLE row a wrapped tooltip's FIRST line should start on,
/// given how many lines it wrapped to and the Ability Bar's own box top
/// row (`box_y` - see ability_bar_box_bounds). Anchors the tooltip's
/// LAST line just above the box (row `box_y - 1`) and grows upward from
/// there, so a longer description never collides with the box/icons
/// below it regardless of how many lines it wraps to - a fixed row
/// (what this used to be) works fine for a short description but runs
/// the risk of a long one overlapping the bar itself.
fn ability_bar_tooltip_start_row(box_y: i32, line_count: i32) -> i32 {
    let bottom_row = box_y - 1;
    bottom_row - (line_count - 1)
}

#[system]
#[read_component(Health)]
#[read_component(Player)]
#[read_component(Item)]
#[read_component(Carried)]
#[read_component(Name)]
#[read_component(BattleItem)]
#[read_component(Weapon)]
#[read_component(Description)]
#[read_component(Gold)]
#[read_component(ShopStock)]
#[read_component(Price)]
#[read_component(Point)]
#[read_component(Class)]
pub fn hud(
    ecs: &SubWorld,
    #[resource] hud_mouse_pos: &HudMousePos,
    #[resource] ability_bar_mouse_pos: &AbilityBarMousePos,
    #[resource] shopping: &Option<ShoppingActive>,
    #[resource] arena_run: &Option<ArenaRun>,
    #[resource] shop_message: &Option<ShopMessage>,
) {
    let mut health_query = <&Health>::query().filter(component::<Player>());
    let player_health = health_query.iter(ecs).nth(0).unwrap();

    let mut draw_batch = DrawBatch::new();
    draw_batch.target(HUD_CONSOLE);
    if let Some(ShopMessage(text)) = shop_message {
        draw_batch.print_color_centered(1, text, ColorPair::new(RED, BLACK));
    } else if shopping.is_some() {
        draw_batch.print_centered(1, "Stand next to an item, press ENTER to buy it.");
    } else {
        draw_batch.print_centered(1, "Explore the Dungeon. Cursor keys to move. M for Items.");
    }
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

    // Battle Arena shows Gold in this corner instead of Dungeon Level -
    // map_level is a dungeon-crawl-only concept Arena code never touches
    // (it stays stuck at whatever it was spawned with), so showing it
    // during an Arena run would just be a stale, meaningless number.
    if arena_run.is_some() {
        let gold = <&Gold>::query()
            .filter(component::<Player>())
            .iter(ecs)
            .nth(0)
            .map_or(0, |g| g.0);
        draw_batch.print_color_right(
            Point::new(HUD_COLS, 1),
            format!("Gold: {}", gold),
            ColorPair::new(YELLOW, BLACK),
        );
    } else {
        draw_batch.print_color_right(
            // (2)
            Point::new(HUD_COLS, 1),
            format!("Dungeon Level: {}", map_level + 1), // (3)
            ColorPair::new(YELLOW, BLACK),
        );
    }

    // Out-of-combat class abilities now live on the Ability Bar - a row
    // of icons along the bottom of the dungeon view - rather than a text
    // list here. See below (after this draw_batch is submitted) for the
    // actual bar; this system still needs `player`/`player_class` for
    // both that and the Battle Attacks/Weapons panels further down.
    let player_class = entity_class(ecs, player);
    let row = 3;
    // ShoppingActive. Every ShopStock entity currently on the map (a real
    // Point + Name + ShopStock + Price - see spawner::spawn_shop_stock_at),
    // not filtered by proximity, so the player can see the whole counter's
    // prices at a glance rather than only whatever they're standing next
    // to.
    if shopping.is_some() {
        let mut shop_row = row + 1;
        let mut stock_query = <(&ShopStock, &Name, &Price)>::query();
        let mut any_stock = false;
        for (stock, name, price) in stock_query.iter(ecs) {
            any_stock = true;
            draw_batch.print_color(
                Point::new(3, shop_row),
                format!("{} x{} - {}g", name.0, stock.0, price.0),
                ColorPair::new(GREEN, BLACK),
            );
            shop_row += 1;
        }
        if any_stock {
            draw_batch.print_color(Point::new(3, row), "Shop", ColorPair::new(YELLOW, BLACK));
        }
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

    // The Ability Bar - a row of icons along the bottom of the dungeon
    // view, one per out-of-combat ability the current class could ever
    // have (components::ability_bar_slots), greyed out when not
    // currently carried. Its own DrawBatch/console (ABILITY_BAR_CONSOLE)
    // since it uses the coarse big-glyph dungeonfont grid, not
    // HUD_CONSOLE's fine text one - see that console's own doc comment
    // in main.rs for the exact cell geometry. Submitted as its own batch,
    // targeting a HIGHER z-order than the panel batch above, so the bar
    // reliably paints over the dungeon view beneath it regardless of
    // draw order here.
    if let Some(class) = &player_class {
        let roster = class_effect_names(class);
        if !roster.is_empty() {
            let mut bar_batch = DrawBatch::new();
            bar_batch.target(ABILITY_BAR_CONSOLE);

            let slots = ability_bar_slots(ecs, player, class, &roster);
            let n = (slots.len() as i32).min(ABILITY_BAR_MAX_SLOTS as i32);
            let start_col = ability_bar_start_col(n);
            let bar_row = ability_bar_row();
            let bar_mouse = ability_bar_mouse_pos.0;
            let mut hovered_ability: Option<&str> = None;

            // Label positions/text - see ability_bar_key_label and
            // ability_bar_label_position for the actual math and
            // reasoning (pulled out to module level so they're testable
            // without a real ECS/DrawBatch).
            let mut label_batch = DrawBatch::new();
            label_batch.target(HUD_CONSOLE);

            for (i, slot) in slots.iter().enumerate().take(n as usize) {
                let col = start_col + i as i32;
                let owned = slot.owned.is_some();
                let glyph = glyph_for_item_name(&slot.name).unwrap_or('?');
                draw_portrait(
                    &mut bar_batch,
                    col,
                    bar_row,
                    Render {
                        color: ColorPair::new(if owned { WHITE } else { DARK_GRAY }, BLACK),
                        glyph: to_cp437(glyph),
                    },
                );
                if bar_mouse.y == bar_row && bar_mouse.x == col {
                    hovered_ability = Some(&slot.name);
                }

                let (label_col, label_row) = ability_bar_label_position(col);
                label_batch.print_color(
                    Point::new(label_col, label_row),
                    ability_bar_key_label(i),
                    ColorPair::new(if owned { YELLOW } else { GRAY }, BLACK),
                );
            }
            // A single box around the whole bar (icons + labels
            // together), not one per icon - drawn on HUD_CONSOLE, same
            // ASCII box style (render_helpers::draw_ascii_box) the
            // battle menu's own action box already uses, just in RED
            // here to stand out as a distinct HUD element.
            let (box_x, box_y, box_w, box_h) = ability_bar_box_bounds(n);
            draw_ascii_box(
                &mut label_batch,
                box_x,
                box_y,
                box_w,
                box_h,
                ColorPair::new(RED, BLACK),
            );
            bar_batch.submit(10001).expect("Batch error");
            label_batch.submit(10002).expect("Batch error");

            // The hovered slot's tooltip - drawn on HUD_CONSOLE (fine
            // text) rather than the bar's own coarse console, which has
            // no room for readable prose. Centered rather than aligned
            // under the specific hovered icon, for the same reasoning
            // the label positions above needed real pixel-ratio math to
            // get right - a full sentence of prose is far more sensitive
            // to being a few columns off than a single digit is.
            if let Some(name) = hovered_ability {
                let mut tooltip_batch = DrawBatch::new();
                tooltip_batch.target(HUD_CONSOLE);
                let description = description_for_item_name(name)
                    .unwrap_or_else(|| "No description.".to_string());
                // Wrapped across multiple lines rather than one long
                // print_color_centered call - some real descriptions
                // (Freeze Trap's, for one) are long enough to run off
                // both edges of the screen on a single line. Anchored to
                // grow UPWARD from just above the bar's own red box
                // (box_y, computed above) rather than a fixed row, so a
                // longer description never collides with the box/icons
                // below it regardless of how many lines it wraps to.
                let lines = wrap_text(&format!("{}: {}", name, description), 70);
                let start_row = ability_bar_tooltip_start_row(box_y, lines.len() as i32);
                for (i, line) in lines.iter().enumerate() {
                    tooltip_batch.print_color_centered(
                        start_row + i as i32,
                        line,
                        ColorPair::new(WHITE, BLACK),
                    );
                }
                tooltip_batch.submit(10003).expect("Batch error");
            }
        }
    }
}
