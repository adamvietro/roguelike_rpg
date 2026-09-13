use crate::prelude::*;

/// Every Carried+Item entity belonging to `wielder` that's actually
/// usable via a number-key press - i.e. NOT a Weapon (equipped/applied
/// automatically, see carried_weapon_damage in battle.rs) and NOT a
/// BattleItem (used from the battle menu instead, not the dungeon-view
/// item keys). This is the single source of truth both usable_menu_items
/// and usable_ability_items below split further - using the same
/// underlying list in both keeps them from ever double-counting or
/// missing an item type.
pub fn usable_carried_items<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    let mut items: Vec<Entity> = <(Entity, &Item, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, carried)| carried.0 == wielder)
        .map(|(e, _, _)| *e)
        .filter(|e| {
            let entry = ecs.entry_ref(*e).unwrap();
            entry.get_component::<Weapon>().is_err() && entry.get_component::<BattleItem>().is_err()
        })
        .collect();

    // Healing Potion first, Dungeon Map second (when carried), everything
    // else after - just a stable, sensible default ordering for the Item
    // Menu list now (see usable_menu_items) rather than a fixed-identity
    // hotkey requirement the way it used to be, back when 1/2 were
    // hardcoded to these specific items.
    items.sort_by_key(|e| item_hotkey_priority(ecs, *e));
    items
}

/// The Class name on an item entity, if it has one - the signal that
/// splits usable_carried_items into the Item Menu's universal
/// consumables (usable_menu_items, no Class at all) versus the Ability
/// Bar's class-restricted abilities (usable_ability_items, Class ==
/// wielder's own class). Every out-of-combat item template already
/// either omits `class:` entirely (Healing Potion, Dungeon Map) or tags
/// it with a specific class (Trap, Throw Spear, Invisible Cloak, ...) -
/// this reuses that existing data rather than needing any new field.
fn item_class<T: EntityStore>(ecs: &T, item: Entity) -> Option<String> {
    ecs.entry_ref(item)
        .ok()
        .and_then(|entry| entry.get_component::<Class>().ok().map(|c| c.0.clone()))
}

/// The Item Menu's contents (press M - see screens/item_menu.rs): every
/// usable_carried_items entry with NO Class restriction at all -
/// Healing Potion, Dungeon Map, and any future item every class can
/// carry. Class-restricted items are deliberately excluded here - see
/// usable_ability_items, their home on the Ability Bar instead.
pub fn usable_menu_items<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    usable_carried_items(ecs, wielder)
        .into_iter()
        .filter(|e| item_class(ecs, *e).is_none())
        .collect()
}

/// The Ability Bar's contents (systems/hud.rs) - every
/// usable_carried_items entry whose Class matches `wielder_class`.
/// Filtered to the wielder's own class defensively, though in practice a
/// carried item's Class should already always match the wielder's -
/// items are only ever granted class-matched in the first place.
pub fn usable_ability_items<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    wielder_class: &str,
) -> Vec<Entity> {
    usable_carried_items(ecs, wielder)
        .into_iter()
        .filter(|e| item_class(ecs, *e).as_deref() == Some(wielder_class))
        .collect()
}

/// Groups a list of item entities by display Name, stacking identical
/// copies (e.g. two Healing Potions become one "Healing Potion" entry
/// with a count of 2) instead of one line/hotkey slot per physical copy.
/// Keeps one representative entity per group (the first copy
/// encountered) - using that group's slot consumes just that one entity,
/// so a stack of 2 correctly becomes a stack of 1 after using one, not
/// both. Shared by the Item Menu (grouping usable_menu_items) and the
/// Ability Bar (grouping usable_ability_items) - previously each had its
/// own near-identical copy of this loop before the item/ability split.
pub fn group_items<T: EntityStore>(ecs: &T, items: Vec<Entity>) -> Vec<(String, i32, Entity)> {
    let mut groups: Vec<(String, i32, Entity)> = Vec::new();
    for item in items {
        let name = match ecs
            .entry_ref(item)
            .ok()
            .and_then(|entry| entry.get_component::<Name>().ok().map(|n| n.0.clone()))
        {
            Some(n) => n,
            None => continue,
        };
        match groups.iter_mut().find(|(existing, _, _)| *existing == name) {
            Some(group) => group.1 += 1,
            None => groups.push((name, 1, item)),
        }
    }
    groups
}

/// The Item Menu's grouped, displayable rows (see screens/item_menu.rs) -
/// usable_menu_items, grouped by name. Since it's a real scrollable/
/// cursor-navigable list now rather than fixed hotkey slots, there's no
/// need to preserve gaps for an uncarried Potion/Map the way the old
/// fixed-identity usable_item_slots had to - group_items' natural
/// "only what's actually carried" output is exactly what a real menu
/// wants.
pub fn usable_item_groups<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<(String, i32, Entity)> {
    group_items(ecs, usable_menu_items(ecs, wielder))
}

/// One Ability Bar slot - see ability_bar_slots. `owned` is None for an
/// ability the class roster includes but the player doesn't currently
/// have any copies of (shown greyed out, per the always-show-the-full-
/// roster convention the battle menu already established for
/// techniques).
#[derive(Clone, Debug, PartialEq)]
pub struct AbilityBarSlot {
    pub name: String,
    /// (count, a representative entity to consume when used) - None if
    /// not currently carried at all.
    pub owned: Option<(i32, Entity)>,
}

/// Matches a fixed `roster` of names against whatever's actually owned
/// (`owned_groups`, from group_items) - the shared "always show the full
/// roster, grey out what's unowned" shape behind ability_bar_slots,
/// battle_bar_slots, and item_bar_slots, pulled out once they'd otherwise
/// be three near-identical copies of this same loop (the same reasoning
/// group_items itself was already extracted for).
fn build_roster_slots(
    roster: &[String],
    owned_groups: &[(String, i32, Entity)],
) -> Vec<AbilityBarSlot> {
    roster
        .iter()
        .map(|name| AbilityBarSlot {
            name: name.clone(),
            owned: owned_groups
                .iter()
                .find(|(n, _, _)| n == name)
                .map(|(_, count, entity)| (*count, *entity)),
        })
        .collect()
}

/// The Ability Bar's full row of slots, in FIXED roster order (from
/// `roster` - see spawner::effect_names_for_class, which reads that
/// order straight from template.ron) - every slot the class could ever
/// have shows up here every time, greyed out (owned: None) when not
/// currently carried, rather than the list compacting around whatever's
/// actually owned right now. This is what makes "press 3" always mean
/// "this class's 3rd roster ability," in battle or out, even during a
/// run where that ability hasn't dropped yet.
pub fn ability_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    wielder_class: &str,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, usable_ability_items(ecs, wielder, wielder_class));
    build_roster_slots(roster, &owned_groups)
}

/// Every BattleItem entity `wielder` currently carries - the Battle Bar's
/// contents (systems/hud.rs's second icon bar, next to the out-of-combat
/// Ability Bar), as opposed to usable_carried_items' EXCLUSION of
/// BattleItem (those are used from the battle menu, not out here).
pub fn battle_items_carried<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<Entity> {
    <(Entity, &Item, &BattleItem, &Carried)>::query()
        .iter(ecs)
        .filter(|(_, _, _, carried)| carried.0 == wielder)
        .map(|(e, _, _, _)| *e)
        .collect()
}

/// The Battle Bar's full row of slots (systems/hud.rs) - same shape and
/// "always show the full roster, grey out what's unowned" convention as
/// ability_bar_slots, but for in-battle Techniques (battle_items_carried)
/// rather than out-of-combat Effects. `roster` should be
/// spawner::class_technique_names(class) - the same roster the battle
/// menu itself already builds its technique list from.
pub fn battle_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, battle_items_carried(ecs, wielder));
    build_roster_slots(roster, &owned_groups)
}

/// The Item Bar's full row of slots (systems/hud.rs's third icon bar,
/// left of the Ability Bar) - same shape and "always show the full
/// roster, grey out what's unowned" convention as ability_bar_slots/
/// battle_bar_slots, but for universal (no `class:` tag) consumables
/// rather than a specific class's abilities. `roster` should be
/// spawner::universal_item_names() - Healing Potion, Dungeon Map, and any
/// future item every class can carry, in template.ron's own file order.
pub fn item_bar_slots<T: EntityStore>(
    ecs: &T,
    wielder: Entity,
    roster: &[String],
) -> Vec<AbilityBarSlot> {
    let owned_groups = group_items(ecs, usable_menu_items(ecs, wielder));
    build_roster_slots(roster, &owned_groups)
}

/// The Item Menu's "Equipped Items" section (screens/item_menu.rs) - just
/// the currently-carried Weapon for now (armor/trinkets don't exist as
/// components yet - see docs/ideas.md). Unlike ability_bar_slots/
/// battle_bar_slots/item_bar_slots, there's no fixed "roster of every
/// weapon this class could ever equip" to show greyed-out placeholders
/// for - only one weapon is ever carried at a time (auto-pickup discards
/// whatever was carried before), so this is just whatever's actually
/// equipped right now, or empty if nothing is.
pub fn equipped_weapon_slots<T: EntityStore>(ecs: &T, wielder: Entity) -> Vec<AbilityBarSlot> {
    <(Entity, &Carried, &Weapon, &Name)>::query()
        .iter(ecs)
        .filter(|(_, carried, _, _)| carried.0 == wielder)
        .map(|(entity, _, _, name)| AbilityBarSlot {
            name: name.0.clone(),
            // The real weapon entity, even though Equipped Items is
            // browse/description-only today (see screens/item_menu.rs)
            // and never actions this via Enter - the real entity costs
            // nothing extra to thread through and is a lot less
            // confusing than a placeholder that would need explaining.
            owned: Some((1, *entity)),
        })
        .collect()
}

/// How many columns of breathing room sit between two adjacent bar groups
/// on ABILITY_BAR_CONSOLE (Item | gap | Ability | gap | Battle) - shared
/// by battle_bar_start_col and item_bar_start_col so the two gaps stay
/// visually identical.
pub const BAR_GROUP_GAP_COLS: i32 = 2;

/// Which ABILITY_BAR_CONSOLE row every bar's icons sit on - one full
/// icon-height above the console's very bottom row, so the bars aren't
/// flush against the physical screen edge. Shared by systems/hud.rs
/// (rendering) and systems/player_input.rs (hit-testing an Item Bar
/// click) - both need the exact same row, not two independently-computed
/// copies that could drift out of sync.
pub fn ability_bar_row() -> i32 {
    ABILITY_BAR_ROWS - 2
}

/// The leftmost column `n` icons should start at to appear centered as a
/// group on ABILITY_BAR_CONSOLE - e.g. a 2-ability class's icons sit
/// centered in the middle of the screen, not pinned to the left edge the
/// way a longer roster's would naturally reach toward anyway. Integer
/// division rounds a genuinely-odd remainder toward the left rather than
/// perfectly splitting a half-column, which isn't expressible on a
/// whole-cell grid regardless. Shared with player_input.rs - see
/// ability_bar_row's own doc comment on why this lives here now instead
/// of only in systems/hud.rs.
pub fn ability_bar_start_col(n: i32) -> i32 {
    (ABILITY_BAR_COLS - n) / 2
}

/// The Battle Bar's own starting column - immediately to the right of the
/// out-of-combat Ability Bar's icons, plus a small gap, so the two boxes
/// read as clearly separate groups rather than touching.
pub fn battle_bar_start_col(ability_bar_start_col: i32, ability_bar_n: i32) -> i32 {
    ability_bar_start_col + ability_bar_n + BAR_GROUP_GAP_COLS
}

/// The Item Bar's own starting column - immediately to the LEFT of the
/// out-of-combat Ability Bar's icons (mirroring battle_bar_start_col's
/// gap on the right), so the row reads as Item | gap | Ability | gap |
/// Battle. Subtracts rather than adds since this group grows leftward
/// from the Ability Bar's own left edge.
pub fn item_bar_start_col(ability_bar_start_col: i32, item_bar_n: i32) -> i32 {
    ability_bar_start_col - BAR_GROUP_GAP_COLS - item_bar_n
}

/// Sort key used by usable_carried_items - see its comment for the slot
/// assignment. Falls back to the "other items" bucket if the entity has
/// no Name for some reason, rather than panicking.
fn item_hotkey_priority<T: EntityStore>(ecs: &T, item: Entity) -> i32 {
    let name = ecs
        .entry_ref(item)
        .ok()
        .and_then(|entry| entry.get_component::<Name>().ok().map(|n| n.0.clone()));
    match name.as_deref() {
        Some("Healing Potion") => 0,
        Some("Dungeon Map") => 1,
        _ => 2,
    }
}

