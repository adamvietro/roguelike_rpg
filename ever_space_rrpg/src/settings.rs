use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// --- Battle Speed --------------------------------------------------------
//
// A global pacing knob for ATB gauge fills (see battle::atb_fill_rate) -
// purely a player-comfort setting. It scales every combatant's fill rate
// by the same multiplier, so it never changes who's faster than whom
// (that's still Speed's job) - it just stretches or compresses how long
// battles feel to sit through, independent of gameplay balance. Adjusted
// from the Options screen (see screens/options.rs) and persisted to disk
// the same way Keymap is, so a chosen speed survives between sessions.

/// How fast ATB gauges fill during battle, as a multiplier applied on top
/// of each combatant's own Speed-derived rate - see
/// battle::atb_fill_rate. Slow/Fast are named for how long a bar takes to
/// fill, which is the INVERSE of the rate multiplier (taking 1.5x as long
/// means filling at 1/1.5 the rate).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BattleSpeed {
    /// Gauges take 1.5x as long to fill as Normal (rate x 1/1.5).
    Slow,
    /// Today's speed, unscaled - the default, and what
    /// battle::atb_fill_rate's own doc comment was tuned against.
    Normal,
    /// Gauges take half as long to fill as Normal (rate x 2).
    Fast,
}

/// Where the chosen BattleSpeed is persisted - see BattleSpeed::load/save.
/// Lives under saves/ alongside keymap.ron/stats.ron, for the same reason
/// Keymap's own path doc comment gives: this is "options," not "save
/// game," but it's the same kind of small local config either way.
const BATTLE_SPEED_PATH: &str = "saves/battle_speed.ron";

impl BattleSpeed {
    /// Every choice, in the order the Options screen lists/cycles them.
    pub const ALL: [BattleSpeed; 3] = [BattleSpeed::Slow, BattleSpeed::Normal, BattleSpeed::Fast];

    pub fn label(self) -> &'static str {
        match self {
            BattleSpeed::Slow => "Slow",
            BattleSpeed::Normal => "Normal",
            BattleSpeed::Fast => "Fast",
        }
    }

    /// The multiplier applied on top of atb_fill_rate's own per-entity
    /// number - see that function and this type's own doc comment for
    /// why Slow divides and Fast multiplies rather than the reverse.
    pub fn rate_multiplier(self) -> f32 {
        match self {
            BattleSpeed::Slow => 1.0 / 1.5,
            BattleSpeed::Normal => 1.0,
            BattleSpeed::Fast => 2.0,
        }
    }

    /// Cycles Slow -> Normal -> Fast -> Slow. Only 3 fixed choices, so
    /// the Options screen just cycles on keypress rather than needing a
    /// Keymap-style "capture the next input" sub-mode.
    pub fn next(self) -> Self {
        match self {
            BattleSpeed::Slow => BattleSpeed::Normal,
            BattleSpeed::Normal => BattleSpeed::Fast,
            BattleSpeed::Fast => BattleSpeed::Slow,
        }
    }

    /// Loads the saved choice from BATTLE_SPEED_PATH, falling back to
    /// Normal if the file doesn't exist, fails to parse, or names
    /// something unrecognized. Never panics on a missing/corrupt save
    /// file, same as Keymap::load.
    pub fn load() -> Self {
        if let Ok(text) = fs::read_to_string(BATTLE_SPEED_PATH) {
            if let Ok(name) = ron::de::from_str::<String>(&text) {
                if let Some(speed) = Self::ALL.iter().copied().find(|s| s.label() == name) {
                    return speed;
                }
            }
        }
        BattleSpeed::Normal
    }

    /// Writes this choice to BATTLE_SPEED_PATH as its plain label text,
    /// creating the saves/ directory first if needed. Silently does
    /// nothing on a write failure, same reasoning as Keymap::save.
    pub fn save(self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) =
            ron::ser::to_string_pretty(&self.label().to_string(), ron::ser::PrettyConfig::default())
        {
            let _ = fs::write(BATTLE_SPEED_PATH, text);
        }
    }
}

// --- ATB Mode ------------------------------------------------------------
//
// FFVII itself shipped three ATB modes (Wait/Active/Semi) - this project
// only offers the two extremes, named to match what the project
// instructions/design chat actually called them: "Wait" (today's
// default) and "True ATB" (FFVII's "Active"). See screens/battle.rs's
// battle_tick for exactly which gauges tick in which BattleTurn state
// under each mode - the short version:
// - Wait: every gauge freezes solid the instant ANYONE has something "in
//   progress" (the player's menu is open, or either side's action result
//   is being shown) - nothing can interrupt a menu or a result screen.
// - Active (True ATB): the enemy's gauge keeps filling even while the
//   PLAYER's menu is open, and can interrupt it (forcing an attack
//   before the player finishes choosing) - "select fast or take the
//   hit" is the entire point. The one exception, kept identical to Wait
//   mode on purpose: while the PLAYER's own action is actually resolving
//   (BattleTurn::ActionResult(Combatant::Player)), everything still
//   freezes, the same "wait while attacking" pause Wait mode always has
//   - only the ENEMY's own action result (ActionResult(Combatant::Enemy(_)))
//   lets gauges keep moving underneath it in Active mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtbMode {
    Wait,
    Active,
}

/// Where the chosen AtbMode is persisted - see AtbMode::load/save. Same
/// saves/ treatment as BATTLE_SPEED_PATH/KEYMAP_PATH.
const ATB_MODE_PATH: &str = "saves/atb_mode.ron";

impl AtbMode {
    pub const ALL: [AtbMode; 2] = [AtbMode::Wait, AtbMode::Active];

    pub fn label(self) -> &'static str {
        match self {
            AtbMode::Wait => "Wait",
            AtbMode::Active => "True ATB",
        }
    }

    /// Only two choices, so "cycle" is just a toggle.
    pub fn next(self) -> Self {
        match self {
            AtbMode::Wait => AtbMode::Active,
            AtbMode::Active => AtbMode::Wait,
        }
    }

    /// Same load contract as BattleSpeed::load - falls back to Wait
    /// (today's long-standing default behavior) on anything missing,
    /// corrupt, or unrecognized.
    pub fn load() -> Self {
        if let Ok(text) = fs::read_to_string(ATB_MODE_PATH) {
            if let Ok(name) = ron::de::from_str::<String>(&text) {
                if let Some(mode) = Self::ALL.iter().copied().find(|m| m.label() == name) {
                    return mode;
                }
            }
        }
        AtbMode::Wait
    }

    /// Same save contract as BattleSpeed::save.
    pub fn save(self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) =
            ron::ser::to_string_pretty(&self.label().to_string(), ron::ser::PrettyConfig::default())
        {
            let _ = fs::write(ATB_MODE_PATH, text);
        }
    }
}

// --- Battle Menu Memory ---------------------------------------------------
//
// Optional: remembers which action (Attack/Defend/Flee, or a specific
// technique by name) was last chosen in a class's battle menu, so a
// FRESH battle's cursor can start right there instead of always at
// Attack - see screens/battle.rs's battle_tick, which seeds the cursor
// from this exactly once per battle (the first time that battle's own
// PlayerMenu is shown). Deliberately two separate persisted pieces
// rather than one:
// - MenuMemory: the on/off toggle itself (Options screen, cycles like
//   AtbMode).
// - LastBattleAction: the actual per-class remembered choices. Kept
//   separate from the toggle so switching the setting off and back on
//   doesn't throw away what was already remembered - it just stops
//   being READ (and stops being WRITTEN to) while off.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuMemory {
    Off,
    On,
}

/// Where the MenuMemory toggle is persisted - see MenuMemory::load/save.
/// Same saves/ treatment as BATTLE_SPEED_PATH/ATB_MODE_PATH.
const MENU_MEMORY_PATH: &str = "saves/menu_memory.ron";

impl MenuMemory {
    pub const ALL: [MenuMemory; 2] = [MenuMemory::Off, MenuMemory::On];

    pub fn label(self) -> &'static str {
        match self {
            MenuMemory::Off => "Off",
            MenuMemory::On => "On",
        }
    }

    /// Only two choices, so "cycle" is a toggle - same shape as
    /// AtbMode::next.
    pub fn next(self) -> Self {
        match self {
            MenuMemory::Off => MenuMemory::On,
            MenuMemory::On => MenuMemory::Off,
        }
    }

    /// Same load contract as AtbMode::load - falls back to Off (memory
    /// disabled) on anything missing, corrupt, or unrecognized.
    pub fn load() -> Self {
        if let Ok(text) = fs::read_to_string(MENU_MEMORY_PATH) {
            if let Ok(name) = ron::de::from_str::<String>(&text) {
                if let Some(v) = Self::ALL.iter().copied().find(|m| m.label() == name) {
                    return v;
                }
            }
        }
        MenuMemory::Off
    }

    /// Same save contract as AtbMode::save.
    pub fn save(self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) =
            ron::ser::to_string_pretty(&self.label().to_string(), ron::ser::PrettyConfig::default())
        {
            let _ = fs::write(MENU_MEMORY_PATH, text);
        }
    }
}

// --- Fullscreen ------------------------------------------------------------
//
// Whether the game window should launch fullscreen - Options screen item
// 12 (docs/ideas.md). bracket-terminal 0.8.7's `BTermBuilder::with_
// fullscreen` is a real, public, supported API - but `platform_hints.
// fullscreen` is only ever read ONCE, at window-creation time inside
// `main()`'s own builder chain (confirmed straight from bracket-terminal's
// real `hal/native/init.rs` source, not guessed) - there's no live/
// runtime toggle without reaching into an internal, not-officially-public
// global (`bracket_terminal::prelude::BACKEND`), which this project
// deliberately avoids for a cosmetic setting (see docs/ideas.md's own
// note on that). So this setting changes what NEXT launch does, not the
// current session - the Options screen says so next to the row.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FullscreenSetting {
    Off,
    On,
}

/// Where the Fullscreen toggle is persisted - see FullscreenSetting::
/// load/save. Same saves/ treatment as MENU_MEMORY_PATH/ATB_MODE_PATH.
const FULLSCREEN_PATH: &str = "saves/fullscreen.ron";

impl FullscreenSetting {
    pub const ALL: [FullscreenSetting; 2] = [FullscreenSetting::Off, FullscreenSetting::On];

    pub fn label(self) -> &'static str {
        match self {
            FullscreenSetting::Off => "Off",
            FullscreenSetting::On => "On",
        }
    }

    /// Only two choices, so "cycle" is a toggle - same shape as
    /// MenuMemory::next.
    pub fn next(self) -> Self {
        match self {
            FullscreenSetting::Off => FullscreenSetting::On,
            FullscreenSetting::On => FullscreenSetting::Off,
        }
    }

    pub fn is_on(self) -> bool {
        self == FullscreenSetting::On
    }

    /// Same load contract as MenuMemory::load - falls back to Off (today's
    /// long-standing default, a windowed launch) on anything missing,
    /// corrupt, or unrecognized. Called both from `main()` directly
    /// (before the ECS/resources exist, to feed `BTermBuilder::with_
    /// fullscreen`) and via the usual resource-insertion sites so the
    /// Options screen has something to read/toggle.
    pub fn load() -> Self {
        if let Ok(text) = fs::read_to_string(FULLSCREEN_PATH) {
            if let Ok(name) = ron::de::from_str::<String>(&text) {
                if let Some(v) = Self::ALL.iter().copied().find(|s| s.label() == name) {
                    return v;
                }
            }
        }
        FullscreenSetting::Off
    }

    /// Same save contract as MenuMemory::save.
    pub fn save(self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) =
            ron::ser::to_string_pretty(&self.label().to_string(), ron::ser::PrettyConfig::default())
        {
            let _ = fs::write(FULLSCREEN_PATH, text);
        }
    }
}

/// Per-class "last battle action chosen," read/written only while
/// MenuMemory::On - see that type's own doc comment. Stores the action's
/// NAME (e.g. "Attack" or "Deathblow"), not a raw (column, row)
/// position: a position is meaningless once you're in a different fight,
/// but a name can be re-found in whatever grid shape the CURRENT fight's
/// class roster happens to have (see screens/battle.rs's
/// seed_menu_cursor_from_memory) - which matters since a technique you
/// didn't own last time might be unlocked (or a previously-owned one
/// used up) by the time this is read again. Every field here is a plain
/// serde-friendly type, so - like Stats - this derives
/// Serialize/Deserialize directly and round-trips through RON as a whole
/// struct, no manual name round-trip needed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LastBattleAction {
    per_class: HashMap<String, String>,
}

/// Where LastBattleAction is persisted - see its own load/save.
const LAST_BATTLE_ACTION_PATH: &str = "saves/last_battle_action.ron";

impl LastBattleAction {
    /// Same load contract as Stats::load - a fresh, empty map if the file
    /// doesn't exist yet or fails to parse, never a panic.
    pub fn load() -> Self {
        fs::read_to_string(LAST_BATTLE_ACTION_PATH)
            .ok()
            .and_then(|text| ron::de::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Same save contract as Stats::save.
    pub fn save(&self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = fs::write(LAST_BATTLE_ACTION_PATH, text);
        }
    }

    /// Remembers `action_name` as the last one chosen for `class`,
    /// overwriting whatever was remembered before. Does NOT save to disk
    /// itself - callers decide when to persist (see
    /// screens/battle.rs::resolve_player_action, which saves right after
    /// recording, same "write immediately" policy every other saved
    /// setting in this project already follows).
    pub fn record(&mut self, class: &str, action_name: &str) {
        self.per_class
            .insert(class.to_string(), action_name.to_string());
    }

    /// The last action name remembered for `class`, if any.
    pub fn get(&self, class: &str) -> Option<&str> {
        self.per_class.get(class).map(String::as_str)
    }
}
