use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
// --- Battle action capability components -----------------------------------
//
// Each of these is a marker component an entity can carry to say "I can do
// this in battle." The battle menu is built at runtime from whichever of
// these the acting entity actually has, rather than a hardcoded list - so a
// future class can mix and match (e.g. a Mage might get CanAttack + CanFlee
// but not CanDefend, or later a CanCastSpell component of its own) without
// touching the menu code at all.

pub struct CanAttack;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanDefend;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanFlee;

/// One battle menu option. The always-available capability actions
/// (Attack/Defend/Flee) come from CanXxx components above; Technique wraps
/// a carried item entity whose mechanical effect (TechniqueEffect, see
/// components.rs) is class/content data rather than a fixed enum variant -
/// see `available_actions` and `apply_player_technique`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleAction {
    Attack,
    Defend,
    Flee,
    /// Points at one representative entity from a group of same-named
    /// carried technique items (see `grouped_carried_techniques`) -
    /// resolving it consumes one item from that group.
    Technique(Entity),
}

/// One rendered battle-menu row: the action it triggers, its display
/// label, and a remaining-use count. `action` is None for a class
/// technique you don't currently own a copy of - it's still shown (greyed
/// out, see main.rs) so the menu always reflects the class's full
/// technique roster rather than only whatever you happen to be carrying,
/// but there's no Entity to reference for it and it can't be selected.
/// Built fresh each menu render, so using an item immediately updates the
/// count.
#[derive(Clone, Debug, PartialEq)]
pub struct BattleMenuEntry {
    pub action: Option<BattleAction>,
    pub label: String,
    pub count: Option<i32>,
}

/// The display name a chosen BattleAction should be remembered as - see
/// settings::LastBattleAction. Attack/Defend/Flee use fixed names
/// matching their own BattleMenuEntry::label exactly, so the same string
/// can be searched for again in a future battle's grid (see
/// screens/battle.rs's cursor-memory seed) regardless of which action
/// type it actually is. Must be called BEFORE the technique item is
/// removed from the ECS (see apply_player_technique) - same ordering
/// requirement Stats::record_ability_used's own call site already
/// follows, for the same reason.
pub fn action_name(ecs: &World, action: BattleAction) -> String {
    match action {
        BattleAction::Attack => "Attack".to_string(),
        BattleAction::Defend => "Defend".to_string(),
        BattleAction::Flee => "Flee".to_string(),
        BattleAction::Technique(item) => entity_name(ecs, item),
    }
}

fn has_can_attack(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanAttack)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

fn has_can_defend(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanDefend)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

fn has_can_flee(ecs: &World, entity: Entity) -> bool {
    <(Entity, &CanFlee)>::query()
        .iter(ecs)
        .any(|(e, _)| *e == entity)
}

/// The ordered list of battle-menu rows this entity currently has
/// available. Always shows the acting class's full technique roster (see
/// class_technique_names) - not just what's currently carried - so the
/// menu stays a stable reference of "what this class can eventually do";
/// entries for techniques not currently owned get `action: None` (count
/// Some(0)) so main.rs can grey them out and skip them on selection. Class
/// filtering happens once here - callers don't need to know or pass the
/// wielder's class at all.
pub fn available_actions(ecs: &World, entity: Entity) -> Vec<BattleMenuEntry> {
    let mut actions = Vec::new();
    if has_can_attack(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Attack),
            label: "Attack".to_string(),
            count: None,
        });
    }
    if has_can_defend(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Defend),
            label: "Defend".to_string(),
            count: None,
        });
    }

    let class = entity_class(ecs, entity).unwrap_or_default();
    let owned = grouped_carried_techniques(ecs, entity, &class);
    for name in class_technique_names(&class) {
        match owned.iter().find(|(n, _)| *n == name) {
            Some((_, entities)) => actions.push(BattleMenuEntry {
                action: Some(BattleAction::Technique(entities[0])),
                label: name,
                count: Some(entities.len() as i32),
            }),
            None => actions.push(BattleMenuEntry {
                action: None,
                label: name,
                count: Some(0),
            }),
        }
    }

    if has_can_flee(ecs, entity) {
        actions.push(BattleMenuEntry {
            action: Some(BattleAction::Flee),
            label: "Flee".to_string(),
            count: None,
        });
    }
    actions
}

/// Maps the number-row keys to a 0-based menu index, matching the existing
/// item-use UX (Key1..Key9) elsewhere in the game.
pub fn number_key_index(key: VirtualKeyCode) -> Option<usize> {
    match key {
        VirtualKeyCode::Key1 => Some(0),
        VirtualKeyCode::Key2 => Some(1),
        VirtualKeyCode::Key3 => Some(2),
        VirtualKeyCode::Key4 => Some(3),
        VirtualKeyCode::Key5 => Some(4),
        VirtualKeyCode::Key6 => Some(5),
        VirtualKeyCode::Key7 => Some(6),
        VirtualKeyCode::Key8 => Some(7),
        VirtualKeyCode::Key9 => Some(8),
        _ => None,
    }
}

/// The battle menu's cursor position - which of the two columns (0 = the
/// fixed Attack/Defend/Flee capability column, 1 = the class's technique
/// roster column) and which row within it. Lives on Battle so it
/// persists for the whole fight; a fresh Battle always starts a fresh
/// cursor at (0, 0) - see Battle::new/menu_cursor_seeded for how
/// cross-battle memory (MenuMemory) can then move it once, before the
/// first PlayerMenu is ever shown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuCursor {
    pub col: usize,
    pub row: usize,
    /// The row remembered for each column the last time the cursor left
    /// it (index 0/1 matches `col` above) - so switching Left/Right and
    /// back returns you to where you were, rather than always landing on
    /// row 0 of whichever column you switch into. Seeded to match
    /// wherever cross-battle memory places the initial cursor too (see
    /// screens/battle.rs), so switching columns right after a memory-
    /// seeded start still has something sensible to fall back to for the
    /// OTHER column.
    remembered_row: [usize; 2],
}

impl MenuCursor {
    pub fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            remembered_row: [0, 0],
        }
    }

    /// Moves the cursor up/down within its CURRENT column, wrapping at
    /// either end - same convention render_helpers::menu_nav already
    /// uses for every top-menu screen, so arrow-key behavior feels
    /// consistent across the whole game. `col_len` is however many rows
    /// the current column actually has right now (0 is a safe no-op -
    /// nothing to move within an empty column).
    pub fn move_vertical(&mut self, delta: i32, col_len: usize) {
        if col_len == 0 {
            return;
        }
        let len = col_len as i32;
        self.row = (((self.row as i32 + delta) % len + len) % len) as usize;
        self.remembered_row[self.col] = self.row;
    }

    /// Switches to `new_col`, restoring whichever row was last visited
    /// there (see `remembered_row`), clamped to `new_col_len` in case
    /// that column has fewer rows now than it did the last time the
    /// cursor was in it (e.g. a shorter technique list than remembered -
    /// not possible today since a class's roster size never changes
    /// mid-battle, but harmless to guard against regardless). A no-op if
    /// the destination column has zero rows (nothing to land on there) or
    /// is already the current column.
    pub fn move_horizontal(&mut self, new_col: usize, new_col_len: usize) {
        if new_col_len == 0 || new_col == self.col {
            return;
        }
        self.remembered_row[self.col] = self.row;
        self.col = new_col;
        self.row = self.remembered_row[new_col].min(new_col_len - 1);
    }
}

