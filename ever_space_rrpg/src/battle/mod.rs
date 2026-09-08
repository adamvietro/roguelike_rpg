use crate::prelude::*;

// --- Attack-type category modules -------------------------------------------
//
// Each module below owns one mechanical category of battle effect - the
// same split TechniqueEffect's variants already fell into, just given a
// home instead of being interpreted inline in one giant match. A
// technique that reuses an existing category (e.g. a new class's flat-
// damage attack) is a template.ron entry only, same as before this
// refactor. A genuinely new category is one new module plus one new
// TechniqueEffect variant and match arm in apply_player_technique below -
// it never needs a new field on Battle, since every category (except
// pure damage, which has no lingering state) stores its active state in
// the same generic StatusSet (see battle::status) rather than inventing
// its own.
pub mod buff;
pub mod counter;
pub mod damage;
pub mod dot;
pub mod heal;
pub mod status;
pub mod stun;

pub use buff::{BuffKind, Magnitude};
pub use damage::HitQueue;
pub use status::{ActiveStatus, StatusKind, StatusSet};

// --- Battle action capability components -----------------------------------
//
// Each of these is a marker component an entity can carry to say "I can do
// this in battle." The battle menu is built at runtime from whichever of
// these the acting entity actually has, rather than a hardcoded list - so a
// future class can mix and match (e.g. a Mage might get CanAttack + CanFlee
// but not CanDefend, or later a CanCastSpell component of its own) without
// touching the menu code at all.

#[derive(Clone, Copy, Debug, PartialEq)]
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

// --- Carried battle-item lookups --------------------------------------------
//
// Each returns every copy of that item `wielder` is currently carrying AND
// can actually use (class-unrestricted items, or items matching
// `wielder_class`), so callers can both count them (for the menu) and
// consume one (removing the first entity in the list) when used.

/// The class name on an entity's Class component, if it has one. Generic
/// over EntityStore so it works both from plain `&World` contexts
/// (screens/battle.rs, screens/title.rs) and from inside a `#[system]`'s
/// `&SubWorld` (systems/player_input.rs's use_ability).
pub fn entity_class<T: EntityStore>(ecs: &T, entity: Entity) -> Option<String> {
    <(Entity, &Class)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, c)| c.0.clone())
}

/// True if an item entity has no class restriction, or its Class matches
/// `wielder_class`.
fn item_usable_by_class(ecs: &World, item: Entity, wielder_class: &str) -> bool {
    entity_class(ecs, item)
        .map(|item_class| item_class == wielder_class)
        .unwrap_or(true)
}

/// Every Carried+Technique item `wielder` can currently use (unrestricted,
/// or matching `wielder_class`), grouped by display Name with all matching
/// entities kept together (so the menu can show "Deathblow x2" and consume
/// one at a time). Replaces the old one-function-per-technique-type
/// approach - the mechanical difference between techniques is data
/// (TechniqueEffect) now, not a distinct Rust component type, so one
/// generic lookup covers every class's techniques.
pub fn grouped_carried_techniques(
    ecs: &World,
    wielder: Entity,
    wielder_class: &str,
) -> Vec<(String, Vec<Entity>)> {
    let mut groups: Vec<(String, Vec<Entity>)> = Vec::new();
    <(Entity, &Carried, &Technique, &Name)>::query()
        .iter(ecs)
        .filter(|(_, carried, _, _)| carried.0 == wielder)
        .filter(|(e, _, _, _)| item_usable_by_class(ecs, **e, wielder_class))
        .for_each(
            |(e, _, _, name)| match groups.iter_mut().find(|(n, _)| *n == name.0) {
                Some((_, entities)) => entities.push(*e),
                None => groups.push((name.0.clone(), vec![*e])),
            },
        );
    groups
}

/// The TechniqueEffect a carried item's Technique component holds, if it
/// has one.
pub fn technique_effect(ecs: &World, item: Entity) -> Option<TechniqueEffect> {
    <(Entity, &Technique)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == item)
        .map(|(_, t)| t.0)
}

/// An entity's Name text, or a generic fallback if it has none.
pub fn entity_name(ecs: &World, entity: Entity) -> String {
    <(Entity, &Name)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, n)| n.0.clone())
        .unwrap_or_else(|| "technique".to_string())
}

// --- Battle state ------------------------------------------------------

/// Up to this many enemies can be in one battle at once - see
/// systems/player_input.rs (and random_move.rs/chasing.rs) for where a
/// battle's roster is actually gathered: every enemy sharing the tile
/// that triggered the fight, capped here. Drives both the ATB race (one
/// gauge per enemy - see EnemyCombatant) and the battle screen's layout
/// (a stacked column of up to this many portraits - see screens/battle.rs).
pub const MAX_BATTLE_ENEMIES: usize = 4;

/// One enemy currently in the battle - bundles everything that used to
/// be a single flat field directly on Battle (gauge, statuses, flash,
/// damage popup) back when there was only ever exactly one enemy. Now
/// that a battle can hold up to MAX_BATTLE_ENEMIES of these, each needs
/// its own independent copy of all of it - you can poison one enemy
/// while stunning another, and each fills its own ATB gauge at its own
/// Speed-derived rate, completely independently of its neighbors.
#[derive(Clone, Debug, PartialEq)]
pub struct EnemyCombatant {
    pub entity: Entity,
    pub name: String,
    /// See Battle::player_gauge's own doc comment - identical mechanics,
    /// just one of these per enemy instead of a single shared field.
    pub gauge: f32,
    pub statuses: StatusSet,
    pub flash: Option<(FlashKind, f32)>,
    pub damage_popup: Option<DamagePopup>,
}

/// Which combatant is acting - Player, or a specific enemy (there can be
/// up to MAX_BATTLE_ENEMIES of them at once, so "Enemy" alone is no
/// longer enough to say which).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combatant {
    Player,
    Enemy(Entity),
}

/// ATB (Active Time Battle, FFVII-style) turn structure. There is no more
/// fixed "round" where both sides act in a fixed order - each combatant
/// has its own gauge (Battle::player_gauge/enemy_gauge) that fills
/// continuously from its Speed stat (see atb_fill_rate), and whichever
/// gauge reaches ATB_GAUGE_MAX first gets to act, independent of the
/// other. This project uses "Wait" ATB (one of FFVII's own three ATB
/// modes): both gauges freeze the instant either one is full, and stay
/// frozen for the whole PlayerMenu/ActionResult exchange - only ticking
/// again once back in Filling. This sidesteps any "what if the OTHER
/// gauge also fills while I'm still picking a menu item" race entirely,
/// while still being a real continuous-time gauge race rather than a
/// fixed turn order - and it keeps the design simple to extend to
/// multiple enemies later (each with its own gauge; whichever fills
/// first still just wins the race, no different in kind from 1v1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BattleTurn {
    /// Both gauges are filling. Every battle_tick frame while in this
    /// state advances player_gauge/enemy_gauge by that combatant's own
    /// atb_fill_rate * elapsed time. The moment either reaches
    /// ATB_GAUGE_MAX, this state ends: to PlayerMenu if the player got
    /// there (checked first - see battle_tick - so a same-frame tie
    /// always favors the player), or straight into an automatic enemy
    /// attack (then ActionResult(Enemy)) otherwise.
    Filling,
    /// The player's gauge is full and frozen at max - waiting for a menu
    /// choice. Was previously shown "either at the start of a round (player
    /// faster) or after the enemy's opening move" under the old fixed-round
    /// system; now it's simply "whenever the player's own gauge fills."
    PlayerMenu,
    /// Result of whichever single combatant just acted (indicated by the
    /// payload) - replaces the old FirstResult/SecondResult pair, since
    /// actions are no longer paired into rounds. Auto-advances after
    /// RESULT_AUTO_ADVANCE_MS (a keypress skips ahead sooner) back to
    /// Filling, with that combatant's gauge reset to 0 - see
    /// Battle::enter_result.
    ActionResult(Combatant),
}

/// Resource describing an in-progress battle. Lives in `Resources` as
/// `Option<Battle>` - `None` when no battle is happening, `Some(..)` while
/// `TurnState::InBattle` is active.
#[derive(Clone, Debug, PartialEq)]
pub struct Battle {
    pub player: Entity,
    /// Every enemy currently in this fight, up to MAX_BATTLE_ENEMIES -
    /// see EnemyCombatant. Shrinks as enemies die (see
    /// screens/battle.rs's record_enemy_kill); the battle itself ends
    /// (BattleVictory) the instant this becomes empty.
    pub enemies: Vec<EnemyCombatant>,
    pub turn: BattleTurn,
    /// Current fill level of the player's ATB gauge, 0.0..=ATB_GAUGE_MAX.
    /// Advances every frame while `turn == Filling` (see battle_tick and
    /// atb_fill_rate); frozen otherwise. Reset to 0.0 the instant the
    /// player's action resolves (see Battle::enter_result) - every
    /// enemy's own gauge (EnemyCombatant::gauge) works the same way,
    /// independently.
    pub player_gauge: f32,
    pub player_defending: bool,
    /// Every currently-active status affecting the PLAYER (Buff,
    /// Counter) - see battle::status::StatusSet. Replaces the old
    /// bespoke `dodge_bonus`/`war_cry`/`countering` fields.
    pub player_statuses: StatusSet,
    pub fled: bool,
    /// Scrolling battle log, most recent entry last - replaces the old
    /// single `message: String` so earlier lines (e.g. a DoT tick right
    /// before an attack) stay readable instead of being overwritten the
    /// instant the next action resolves. Bounded to MAX_LOG_LINES by
    /// push_log; main.rs renders the tail of this Vec every battle_tick
    /// frame instead of a single centered line.
    pub log: Vec<String>,
    pub player_flash: Option<(FlashKind, f32)>,
    /// A briefly-shown floating damage number over the player's portrait
    /// - set by show_player_damage right after apply_damage returns the
    /// real (post-Defense) amount, and ticked down each frame in
    /// battle_tick. Each enemy has its own equivalent on EnemyCombatant.
    pub player_damage_popup: Option<DamagePopup>,
    /// Counts down while `turn` is ActionResult - once it hits zero,
    /// battle_tick advances automatically instead of waiting for a
    /// keypress (a keypress still skips ahead immediately, it just isn't
    /// required anymore). Set via enter_result whenever the turn changes
    /// to that state.
    pub result_timer_ms: f32,
    /// True for a battle that started as a Stealth ambush (see
    /// systems/player_input.rs) - triples the damage of whichever action
    /// the player picks first. Cleared (one-shot) the instant that first
    /// PlayerMenu action resolves, whatever it was - see
    /// screens/battle.rs's BattleTurn::PlayerMenu handling. Under the old
    /// fixed-round system this also forced "player goes first regardless
    /// of Speed"; under ATB, as_sneak_attack achieves the same guarantee
    /// more directly, by starting player_gauge already at max (see
    /// as_sneak_attack) rather than overriding a turn-order decision that
    /// no longer exists.
    pub sneak_attack: bool,
    /// True ATB (AtbMode::Active) only: a player action chosen while the
    /// player couldn't yet actually act - specifically, while some
    /// enemy's own ActionResult was still playing (see
    /// screens/battle.rs's BattleTurn::ActionResult(Combatant::Enemy(_))
    /// handling) - held here to resolve the INSTANT it's safe to
    /// (battle_tick's Filling handling checks this before anything
    /// else), rather than forcing the player to wait for a fresh
    /// PlayerMenu prompt that might not come again for a while. Without
    /// this, an enemy interrupt used to just discard the player's chance
    /// to act entirely until the next race - which, under Wait's normal
    /// enemy-goes-first framing plus True ATB's "enemy never stops"
    /// framing together, made it very hard to ever land a hit at all.
    /// Always None in Wait mode - nothing ever writes to it there. Only
    /// ever holds ONE action regardless of how many enemies act while
    /// it's queued - not a stack of several moves lined up in advance.
    pub queued_player_action: Option<BattleAction>,
    /// Gold accumulated so far THIS fight, across every enemy killed in
    /// it (Battle Arena only - see record_enemy_kill). A multi-enemy
    /// fight can end with several kills' worth of gold; this is what
    /// gets shown as one combined total on the eventual BattleVictory
    /// screen rather than just the last kill's own reward.
    pub gold_earned: i32,
    /// Loot found so far this fight, one entry per kill that happened to
    /// drop something (Dungeon Crawl only - Battle Arena kills grant
    /// gold instead, never loot, same as before this was a Vec). Shown
    /// as a list on BattleVictory instead of a single item.
    pub loot_found: Vec<String>,
    /// One name per enemy defeated so far this fight, in kill order -
    /// needed because by the time the whole battle ends (enemies is
    /// empty), there's nothing left in `enemies` itself to read names
    /// from. See screens/battle.rs's record_enemy_kill/finish_battle.
    pub defeated_names: Vec<String>,
    /// A multi-hit technique's still-pending hits, landing one at a time
    /// - see battle::damage::HitQueue/tick_hit_queue. None whenever no
    /// multi-hit sequence is currently playing out (which is most of the
    /// time - only MultiHit/AoeMultiHit ever populate this).
    pub hit_queue: Option<HitQueue>,
    /// The battle menu's arrow-key cursor position - see MenuCursor.
    pub menu_cursor: MenuCursor,
    /// Whether cross-battle cursor memory (MenuMemory) has already been
    /// applied to menu_cursor for THIS battle - set true the first time
    /// screens/battle.rs's battle_tick sees this false, so memory only
    /// ever moves the cursor once per fight (right at its very start),
    /// never overriding a choice the player actually made mid-battle.
    pub menu_cursor_seeded: bool,
    /// Which frame of the player's own battle-idle loop
    /// (`resources/character_battle.png`, via components::
    /// character_battle_glyph) is currently showing, and how long it's
    /// been showing it - ticked every frame in battle_tick, the same
    /// place player_flash/player_damage_popup already tick down. Not an
    /// ECS `IdleAnimation` component - the battle portrait isn't a
    /// dungeon-view entity, and `Battle` already persists for exactly
    /// the lifetime this animation needs to (fresh at 0/0.0 every new
    /// fight, same as every other per-battle field here).
    pub player_idle_frame: usize,
    pub player_idle_elapsed_ms: f32,
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

/// Which color a portrait's brief post-action flash should use - see
/// Battle::enemy_flash/player_flash.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlashKind {
    /// This combatant just landed a hit - a bright, energetic flash.
    Attacking,
    /// This combatant just took damage - a red "ouch" flash.
    Hit,
}

/// Upper bound for each combatant's RANDOM starting ATB gauge value (see
/// Battle::new) - a fraction of ATB_GAUGE_MAX rather than the full range,
/// so a battle's opening instant always shows at least a little bit of
/// gauge-filling before anyone can act, rather than an occasional
/// zero-warning instant attack the moment the screen appears. Raise this
/// toward ATB_GAUGE_MAX for more frequent/dramatic "caught you off
/// guard" opens (at 100%, either side could occasionally act the instant
/// battle starts); lower it toward 0 to make Speed the dominant factor
/// in who acts first again, same as before this was added.
pub const ATB_RANDOM_START_MAX: f32 = ATB_GAUGE_MAX * 0.8;

impl Battle {
    /// `enemies` is every (Entity, Name) pair joining this fight - see
    /// systems/player_input.rs (and random_move.rs/chasing.rs) for where
    /// that roster is actually gathered (every enemy sharing the tile
    /// that triggered the battle) and capped to MAX_BATTLE_ENEMIES.
    pub fn new(player: Entity, enemies: Vec<(Entity, String)>) -> Self {
        // Each gauge - the player's AND every enemy's own - starts at an
        // independent random value instead of a flat 0.0 - without this,
        // the higher-Speed combatant would reach ATB_GAUGE_MAX first in
        // literally every battle (Speed alone would fully determine who
        // acts first, every time), with no randomness at all in who
        // opens a fight. A random head start means a slower combatant
        // can occasionally begin close enough to full that it reaches
        // ATB_GAUGE_MAX before a faster one does, regardless of either
        // one's actual fill rate - see ATB_RANDOM_START_MAX for how large
        // that head start can be.
        let mut rng = RandomNumberGenerator::new();
        let enemies = enemies
            .into_iter()
            .map(|(entity, name)| EnemyCombatant {
                entity,
                name,
                gauge: rng.range(0.0, ATB_RANDOM_START_MAX),
                statuses: StatusSet::default(),
                flash: None,
                damage_popup: None,
            })
            .collect();
        Self {
            player,
            enemies,
            turn: BattleTurn::Filling,
            player_gauge: rng.range(0.0, ATB_RANDOM_START_MAX),
            player_defending: false,
            player_statuses: StatusSet::default(),
            fled: false,
            log: Vec::new(),
            player_flash: None,
            player_damage_popup: None,
            result_timer_ms: 0.0,
            sneak_attack: false,
            queued_player_action: None,
            gold_earned: 0,
            loot_found: Vec::new(),
            defeated_names: Vec::new(),
            hit_queue: None,
            menu_cursor: MenuCursor::new(),
            menu_cursor_seeded: false,
            player_idle_frame: 0,
            player_idle_elapsed_ms: 0.0,
        }
    }

    /// Marks this battle as a Stealth ambush - see Battle::sneak_attack.
    /// Also starts the player's ATB gauge already full (and jumps
    /// straight to PlayerMenu, skipping Filling for this one opening
    /// beat) rather than leaving every gauge to race from zero - an
    /// ambush should mean "you act immediately," not merely "you have a
    /// speed edge this race." Every enemy's gauge is left at whatever
    /// Battle::new already randomized it to. Chainable so
    /// player_input.rs can set it right after Battle::new without an
    /// extra statement.
    pub fn as_sneak_attack(mut self) -> Self {
        self.sneak_attack = true;
        self.player_gauge = ATB_GAUGE_MAX;
        self.turn = BattleTurn::PlayerMenu;
        self
    }

    /// The enemy in this battle with entity == `target`, if it's still
    /// alive/present (it won't be, right after it dies - see
    /// screens/battle.rs's record_enemy_kill, which removes it from
    /// `enemies` immediately).
    pub fn enemy(&self, target: Entity) -> Option<&EnemyCombatant> {
        self.enemies.iter().find(|e| e.entity == target)
    }

    /// Mutable version of `enemy` - see that method's doc comment.
    pub fn enemy_mut(&mut self, target: Entity) -> Option<&mut EnemyCombatant> {
        self.enemies.iter_mut().find(|e| e.entity == target)
    }

    /// Which enemy the player's own single-target actions (plain Attack,
    /// or a single-target Technique) hit, since the player doesn't
    /// choose a target directly with more than one enemy present -
    /// always whichever living enemy currently has the highest Speed,
    /// ties broken by whoever's earlier in `enemies` (a stable, always-
    /// reproducible order - the same two enemies with equal Speed always
    /// resolve the tie the same way). Recomputed fresh every time it's
    /// called rather than cached, so it automatically shifts onto the
    /// next-fastest survivor the instant the current target dies -
    /// there's deliberately no "locked-in" target that would need
    /// separate invalidation logic. Returns None only if `enemies` is
    /// empty, which shouldn't be reachable in practice: the battle ends
    /// (see record_enemy_kill) the instant that happens, before
    /// anything would need a target again.
    pub fn primary_target(&self, ecs: &World) -> Option<Entity> {
        self.enemies
            .iter()
            .max_by_key(|e| entity_speed(ecs, e.entity))
            .map(|e| e.entity)
    }

    /// Switches to ActionResult(acting) and (re)arms the auto-advance
    /// timer, resetting `acting`'s own gauge back to 0.0 - every OTHER
    /// combatant's gauge is left untouched (see the field doc comments
    /// on player_gauge/EnemyCombatant::gauge). Use this instead of
    /// assigning `self.turn`/gauges directly, so the timer and the gauge
    /// reset can never be left stale or forgotten.
    pub fn enter_result(&mut self, acting: Combatant) {
        match acting {
            Combatant::Player => self.player_gauge = 0.0,
            Combatant::Enemy(target) => {
                if let Some(enemy) = self.enemy_mut(target) {
                    enemy.gauge = 0.0;
                }
            }
        }
        self.turn = BattleTurn::ActionResult(acting);
        self.result_timer_ms = RESULT_AUTO_ADVANCE_MS;
    }

    /// Appends a line to the battle log, dropping the oldest line once past
    /// MAX_LOG_LINES. Skips genuinely empty strings so a DoT tick that had
    /// nothing to report (see tick_dot's None case) doesn't leave a blank
    /// entry in the log.
    pub fn push_log(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        self.log.push(line);
        if self.log.len() > MAX_LOG_LINES {
            self.log.remove(0);
        }
    }

    /// Arms a floating damage number over `target`'s portrait - call with
    /// the real (post-Defense) amount apply_damage returned. A no-op if
    /// `target` isn't (or is no longer) in this battle.
    pub fn show_enemy_damage(&mut self, target: Entity, amount: i32) {
        if let Some(enemy) = self.enemy_mut(target) {
            enemy.damage_popup = Some(DamagePopup {
                amount,
                remaining_ms: DAMAGE_POPUP_DURATION_MS,
            });
        }
    }

    /// Arms a floating damage number over the player's portrait - call with
    /// the real (post-Defense) amount apply_damage returned.
    pub fn show_player_damage(&mut self, amount: i32) {
        self.player_damage_popup = Some(DamagePopup {
            amount,
            remaining_ms: DAMAGE_POPUP_DURATION_MS,
        });
    }

    /// Sets `target`'s post-action flash - see EnemyCombatant::flash. A
    /// no-op if `target` isn't (or is no longer) in this battle.
    pub fn set_enemy_flash(&mut self, target: Entity, kind: FlashKind) {
        if let Some(enemy) = self.enemy_mut(target) {
            enemy.flash = Some((kind, PORTRAIT_FLASH_DURATION_MS));
        }
    }
}

/// How long a portrait's post-action color flash lasts, in milliseconds.
/// See Battle::enemy_flash/player_flash and flash_tint in render_helpers.rs.
pub const PORTRAIT_FLASH_DURATION_MS: f32 = 150.0;

/// A floating damage number shown briefly over a portrait - see
/// Battle::enemy_damage_popup/player_damage_popup.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamagePopup {
    pub amount: i32,
    pub remaining_ms: f32,
}

/// How long a floating damage number stays on screen, in milliseconds.
pub const DAMAGE_POPUP_DURATION_MS: f32 = 700.0;

/// Battle::log is trimmed to this many most-recent lines - see push_log.
pub const MAX_LOG_LINES: usize = 4;

/// How long an ActionResult sits on screen before battle_tick advances
/// automatically - see Battle::result_timer_ms/enter_result. A keypress
/// still skips ahead immediately; this is just the natural pace when the
/// player doesn't bother pressing anything.
pub const RESULT_AUTO_ADVANCE_MS: f32 = 1100.0;

/// The value an ATB gauge counts up to before that combatant is ready to
/// act - see Battle::player_gauge/enemy_gauge and atb_fill_rate. An
/// arbitrary round number, not tied to any other unit; only the RATIO
/// between two combatants' fill rates (i.e. their relative Speed) affects
/// who acts more often, not this constant's absolute value.
pub const ATB_GAUGE_MAX: f32 = 100.0;

/// How many ATB gauge points a combatant with a given Speed gains per
/// millisecond of real time while `turn == BattleTurn::Filling` - i.e.
/// this entity's Speed stat times a shared per-point rate. At the
/// project's typical Speed range (roughly 2-10, see template.ron), this
/// puts full-gauge times in the ballpark of 1.5-8 real seconds: fast
/// (Speed 10) acts roughly every ~1.7s, slow (Speed 2) roughly every
/// ~8.3s, average (Speed 6) roughly every ~2.8s. Tune this one constant
/// to make every battle in the game faster/slower at once, since it's
/// the only place the real-time-to-fill relationship is defined.
pub const ATB_GAUGE_PER_MS_PER_SPEED: f32 = 0.006;

/// How fast `entity`'s ATB gauge fills, in gauge points per millisecond -
/// see ATB_GAUGE_PER_MS_PER_SPEED. Speed 0 or below would never fill at
/// all, which would softlock a battle, so this floors the effective
/// Speed used for the rate at 1 (entity_speed's own "no Speed component"
/// default is already 5, well above this floor - this only guards
/// against a template that explicitly sets speed: Some(0) or a negative
/// value).
pub fn atb_fill_rate(ecs: &World, entity: Entity) -> f32 {
    let speed = entity_speed(ecs, entity).max(1) as f32;
    speed * ATB_GAUGE_PER_MS_PER_SPEED
}

/// Defend used to be a guaranteed 50% reduction on the next hit taken. Now
/// it's a gamble: this is the percent chance that reduction actually
/// triggers at all (see resolve_enemy_attack) - on a miss, Defend does
/// nothing this turn beyond having been selected.
pub const DEFEND_SUCCESS_CHANCE_PERCENT: i32 = 30;

/// What to show on the post-battle victory screen (TurnState::BattleVictory)
/// - accumulated across every kill in the fight (see
/// screens/battle.rs's record_enemy_kill), shown once the whole battle
/// ends, then cleared when the player dismisses it. `player` stays valid
/// since the player entity is never removed, so its Render is looked up
/// live - every enemy is gone by this point and isn't shown.
#[derive(Clone, Debug, PartialEq)]
pub struct BattleVictory {
    pub player: Entity,
    /// One name per enemy defeated this fight, in kill order - a single-
    /// enemy fight (the common case) just has one entry, same message as
    /// before this was a Vec.
    pub enemy_names: Vec<String>,
    /// One entry per kill that happened to drop something - empty if
    /// nothing dropped at all (Dungeon Crawl), always empty in the
    /// Battle Arena (kills grant gold there instead, never loot).
    pub loot: Vec<String>,
    /// Total gold earned across every kill this fight, Battle Arena only
    /// - None for a Dungeon Crawl fight (gold doesn't exist there at
    /// all). See screens/battle.rs's record_enemy_kill, the only place
    /// this is ever accumulated.
    pub gold_earned: Option<i32>,
}

// --- Shared lookups/helpers used by the battle screen -----------------------

/// An entity's own base Damage component value, or 0 if it doesn't have one.
pub fn entity_damage(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Damage)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, d)| d.0)
        .unwrap_or(0)
}

/// An entity's own Speed component value, or a neutral default (5) if it
/// doesn't have one. Higher acts first in battle - see battle_tick.
pub fn entity_speed(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Speed)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, s)| s.0)
        .unwrap_or(5)
}

/// An entity's own Evasion component value, or 0 (no innate dodge chance)
/// if it doesn't have one - most classes/enemies today.
pub fn entity_evasion(ecs: &World, entity: Entity) -> i32 {
    <(Entity, &Evasion)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, ev)| ev.0)
        .unwrap_or(0)
}

/// Sum of Damage on anything Carried by `wielder` (i.e. equipped weapons).
/// Mirrors the weapon-damage lookup the old combat system used.
pub fn carried_weapon_damage(ecs: &World, wielder: Entity) -> i32 {
    <(&Carried, &Damage)>::query()
        .iter(ecs)
        .filter(|(carried, _)| carried.0 == wielder)
        .map(|(_, dmg)| dmg.0)
        .sum()
}

/// Current/max HP for an entity, or (0, 0) if it has no Health component.
pub fn entity_health(ecs: &World, entity: Entity) -> (i32, i32) {
    <(Entity, &Health)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, h)| (h.current, h.max))
        .unwrap_or((0, 0))
}

/// Subtract `amount` from an entity's current Health, reduced by the
/// target's Defense (if any). Can go below zero; callers check for death
/// via entity_health and clamp for display. Returns the actual damage
/// dealt (post-Defense) - callers should use this return value, not the
/// `amount` they passed in, when building a message: the two can diverge
/// for any entity with nonzero Defense (e.g. Mage's -1).
pub fn apply_damage(ecs: &mut World, entity: Entity, amount: i32) -> i32 {
    let mut actual_damage = 0;
    <(Entity, &mut Health, Option<&Defense>)>::query()
        .iter_mut(ecs)
        .filter(|(e, _, _)| **e == entity)
        .for_each(|(_, hp, defense)| {
            let defense = defense.map_or(0, |d| d.0);
            let damage = (amount - defense).max(0);
            hp.current -= damage;
            actual_damage = damage;
        });
    actual_damage
}

/// The player's normal attack damage: base Damage plus any equipped weapon.
pub fn player_attack_damage(ecs: &World, player: Entity) -> i32 {
    entity_damage(ecs, player) + carried_weapon_damage(ecs, player)
}

/// Restores `amount` HP to an entity, clamped to its max. Shared by
/// battle::heal (the Heal technique effect).
pub fn heal_entity(ecs: &mut World, entity: Entity, amount: i32) {
    <(Entity, &mut Health)>::query()
        .iter_mut(ecs)
        .filter(|(e, _)| **e == entity)
        .for_each(|(_, hp)| hp.current = (hp.current + amount).min(hp.max));
}

/// An enemy automatically attacks the player. Enemies currently only ever
/// know Attack (see CanAttack / available_actions), so this is a simple
/// hardcoded action - a natural place for smarter enemy AI to hook in
/// later.
///
/// This is the single place every enemy attack against the player passes
/// through, so it's also the one place that has to check every status
/// that can intercept an attack, in order: Stun (skips the attack
/// entirely) -> Evasion (skips the damage entirely) -> Defend/Ice
/// Armor/War Cry (each reduce the damage that lands) -> Counter (reacts
/// to a landed hit). Each of those checks now delegates to its own
/// category module instead of being inlined here - this function is the
/// ORDER they happen in, not their individual mechanics. `attacker` is
/// which specific enemy is swinging this time - Stun is checked/ticked
/// on THAT enemy specifically (a different enemy in the same battle could
/// be perfectly free to act), and a landed Counter reflects damage back
/// at THAT enemy, not just "the enemy" generically.
pub fn resolve_enemy_attack(ecs: &mut World, battle: &mut Battle, attacker: Entity) {
    // A stunned enemy (Hunter's Stun or Feint) doesn't attack at all this
    // turn - checked ahead of even the Evasion check below, since this is
    // "the enemy never swings" rather than "the enemy swings and misses."
    // No Defend/Ice Armor/Counter gets consumed either, same reasoning as
    // the full-dodge early return further down.
    if stun::check_and_tick(battle, attacker) {
        battle.push_log("The enemy is stunned and can't act.".to_string());
        battle.player_defending = false;
        return;
    }

    // Evasion check (base Evasion stat + any active Dodge-technique bonus,
    // additive). A full dodge skips the entire rest of this function: no
    // Defend or Ice Armor gets consumed and no Counter triggers, since
    // nothing actually landed to defend against or counter.
    let dodge_chance =
        entity_evasion(ecs, battle.player) + buff::flat_value(battle, BuffKind::Evasion);
    let mut rng = RandomNumberGenerator::new();
    let evaded = dodge_chance > 0 && rng.range(0, 100) < dodge_chance;

    // The Evasion buff's duration ticks down once per enemy attack faced,
    // regardless of whether this particular attack was the one that got
    // evaded.
    buff::tick_on_attack_faced(battle, BuffKind::Evasion);

    if evaded {
        battle.set_enemy_flash(attacker, FlashKind::Attacking);
        battle.push_log("Dodge attack.".to_string());
        battle.player_defending = false;
        return;
    }

    let mut dmg = entity_damage(ecs, attacker);
    if battle.player_defending && dmg > 0 {
        // Gamble, not a guarantee: roll separately from the dodge check
        // above (Defend and Evasion are different mechanics and shouldn't
        // share a roll), and on success halve the damage with no floor -
        // a successful Defend can now reduce a small hit all the way to 0.
        if rng.range(0, 100) < DEFEND_SUCCESS_CHANCE_PERCENT {
            dmg /= 2;
        }
    }

    // Ice Armor is applied out-of-combat (via an Invisible-Cloak-style
    // item - "Mages buff before battle") and persists as a status on the
    // player rather than per-Battle state, so it's looked up here instead
    // of read from `battle` directly - see components::IceArmored.
    let ice_armor = entity_ice_armor(ecs, battle.player);
    if let Some(armor) = &ice_armor {
        dmg = (dmg - armor.defense_bonus).max(0);
    }

    // War Cry (Amazon) - a Buff of kind DamageReduction: a random
    // reduction per hit, on top of any Ice Armor already subtracted
    // above. Ticks down (and clears once exhausted) only here, past the
    // evaded-early-return above - a dodge shouldn't spend a charge, since
    // no damage landed to reduce.
    dmg = buff::tick_and_reduce(battle, BuffKind::DamageReduction, dmg);

    let dmg = damage::strike_player(ecs, battle, attacker, dmg);
    battle.push_log(damage::take_message(dmg));
    battle.player_defending = false;

    // Ice Armor wears down by one attack actually absorbed, same "attacks"
    // semantics the old in-battle Shield technique used - just persistent
    // across turns and battles now instead of scoped to a single fight.
    if let Some(armor) = ice_armor {
        let mut cb = CommandBuffer::new(ecs);
        if armor.attacks_remaining <= 1 {
            cb.remove_component::<IceArmored>(battle.player);
        } else {
            cb.add_component(
                battle.player,
                IceArmored {
                    defense_bonus: armor.defense_bonus,
                    attacks_remaining: armor.attacks_remaining - 1,
                },
            );
        }
        cb.flush(ecs);
    }

    counter::resolve_on_hit(ecs, battle, attacker);
}

/// An entity's active IceArmored bonus, if any - see resolve_enemy_attack.
/// Public so main.rs can also show it as an active-status line in battle.
pub fn entity_ice_armor(ecs: &World, entity: Entity) -> Option<IceArmored> {
    <(Entity, &IceArmored)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, armor)| *armor)
}

/// If a damage-over-time effect is active on `target`, ticks it down by
/// one and applies its damage. Called once per enemy, right before that
/// enemy would act - see screens/battle.rs's battle_tick. Thin wrapper
/// over battle::dot::tick - kept as a top-level name since
/// screens/battle.rs already calls it that way.
pub fn tick_dot(ecs: &mut World, battle: &mut Battle, target: Entity) -> Option<String> {
    dot::tick(ecs, battle, target)
}

/// Applies a chosen technique's effect on behalf of the player against
/// `target` (the auto-selected enemy - see Battle::primary_target;
/// ignored entirely by the self-buff effects, Heal/Evade/WarCry/Counter,
/// which have no target), consuming one copy of `item` first. This is
/// the single place a technique's mechanical effect is dispatched -
/// main.rs no longer needs one match arm per technique, and neither does
/// this function anymore: each variant's actual mechanics live in its
/// category module (battle::damage, battle::dot, battle::stun, etc.)
/// above. Adding a new class's technique that reuses an existing
/// TechniqueEffect shape needs zero code changes here (just a
/// template.ron entry); a genuinely new mechanic needs one new module
/// (or one new function in an existing one) plus one new match arm here,
/// not a new component/BattleAction variant/main.rs block like before
/// this refactor.
pub fn apply_player_technique(
    ecs: &mut World,
    battle: &mut Battle,
    item: Entity,
    target: Entity,
) -> String {
    let effect = match technique_effect(ecs, item) {
        Some(e) => e,
        None => return String::new(),
    };
    let name = entity_name(ecs, item);

    let mut cb = CommandBuffer::new(ecs);
    cb.remove(item);
    cb.flush(ecs);

    match effect {
        TechniqueEffect::DamageMultiplier(multiplier) => {
            damage::damage_multiplier(ecs, battle, target, multiplier)
        }
        TechniqueEffect::FlatDamage(amount) => damage::flat_damage(ecs, battle, target, amount),
        TechniqueEffect::MultiHit(hits) => damage::multi_hit(ecs, battle, target, hits),
        TechniqueEffect::Counter {
            chance_percent,
            multiplier,
        } => counter::apply(battle, chance_percent, multiplier),
        TechniqueEffect::DamageOverTime { damage, turns } => {
            dot::apply(battle, target, damage, turns, name.to_lowercase());
            "Apply bleed.".to_string()
        }
        TechniqueEffect::Heal { amount } => heal::apply(ecs, battle, amount),
        TechniqueEffect::Evade {
            chance_percent,
            turns,
        } => {
            buff::apply(
                battle,
                BuffKind::Evasion,
                Magnitude::Flat(chance_percent),
                turns,
            );
            "Boost evasion.".to_string()
        }
        TechniqueEffect::WarCry {
            min_reduction,
            max_reduction,
            attacks,
        } => {
            buff::apply(
                battle,
                BuffKind::DamageReduction,
                Magnitude::Random {
                    min: min_reduction,
                    max: max_reduction,
                },
                attacks,
            );
            "Rally your courage.".to_string()
        }
        TechniqueEffect::PoisonStrike {
            initial,
            dot_damage,
            dot_turns,
        } => {
            let total = initial + carried_weapon_damage(ecs, battle.player);
            let dmg = damage::strike_enemy(ecs, battle, target, total);
            dot::apply(battle, target, dot_damage, dot_turns, name.to_lowercase());
            if dmg == 0 {
                "Dodge attack. Poison lingers.".to_string()
            } else {
                format!("Deal {} damage. Poison lingers.", dmg)
            }
        }
        TechniqueEffect::Stun {
            chance_percent,
            turns,
        } => stun::roll(battle, target, chance_percent, turns),
        TechniqueEffect::Feint => stun::feint(battle, target),
        TechniqueEffect::AoeMultiHit { hits, bonus_damage } => {
            damage::aoe_multi_hit(ecs, battle, hits, bonus_damage)
        }
    }
}

/// An entity's Render component (color + glyph), if it has one. Used to draw
/// the scaled-up battle portraits using the same glyph the entity uses on
/// the dungeon map.
pub fn entity_render_component(ecs: &World, entity: Entity) -> Option<Render> {
    <(Entity, &Render)>::query()
        .iter(ecs)
        .find(|(e, _)| **e == entity)
        .map(|(_, r)| *r)
}

/// A simple bracket-style text health bar, e.g. "[######----]".
pub fn hp_bar_string(current: i32, max: i32, width: usize) -> String {
    if max <= 0 {
        return format!("[{}]", "-".repeat(width));
    }
    let ratio = (current.max(0) as f32 / max as f32).min(1.0);
    let filled = ((ratio * width as f32).round() as usize).min(width);
    let empty = width - filled;
    format!("[{}{}]", "#".repeat(filled), "-".repeat(empty))
}
