use crate::prelude::*;
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
