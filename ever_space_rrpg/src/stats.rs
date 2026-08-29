use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// --- Play history ------------------------------------------------------
//
// Persistent across every run on this install - NOT reset when a run
// starts/ends, unlike everything else Resources::default() wipes at
// State::start_game/return_to_title. Same "reload from disk at every
// reset point" pattern as Keymap (see keymap.rs), for the same reason:
// there's no single long-lived in-memory copy on State itself, just
// Stats::load() called fresh each time and re-inserted as a resource.
//
// Unlike Keymap, every field here is a plain serde-friendly type
// (String/u32/HashMap), so this derives Serialize/Deserialize directly
// and round-trips through RON as a whole struct - no manual name
// round-trip needed, since nothing here is an external, non-serde type
// like VirtualKeyCode.

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClassStats {
    /// Number of runs started as this class (see record_game_started) -
    /// the "m" in "n of m" class success.
    pub games_played: u32,
    /// Number of those runs that ended in victory - the "n" in "n of m".
    pub games_won: u32,
    pub enemies_killed: u32,
    /// Deepest dungeon level reached with this class, 0-indexed (matches
    /// Player::map_level) - display code adds 1 for a human "Level N"
    /// label, same as advance_level's own level numbering. Only ever
    /// raised, never lowered - see record_deepest_level.
    pub deepest_level: u32,
    /// How many times each of this class's abilities has been used -
    /// keyed by the ability's own Name (e.g. "Deathblow", "Throw
    /// Spear"), covering both in-battle techniques and out-of-combat
    /// effects. Only class-gated abilities are recorded here (see
    /// record_ability_used) - a universal item like a Healing Potion
    /// isn't any one class's "ability" and doesn't appear in any class's
    /// map.
    pub ability_uses: HashMap<String, u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    /// Total runs started, any class - the "y" in overall "x of y".
    pub games_played: u32,
    /// Total runs that ended in victory, any class - the "x" in overall
    /// "x of y".
    pub games_won: u32,
    pub enemies_killed: u32,
    pub per_class: HashMap<String, ClassStats>,
}

/// Where play history is persisted - see Stats::load/save. Lives
/// alongside keymap.ron under saves/ (already gitignored).
const STATS_PATH: &str = "saves/stats.ron";

impl Stats {
    /// Loads saved history from STATS_PATH, or a fresh all-zero Stats if
    /// the file doesn't exist yet or fails to parse. Never panics on a
    /// missing/corrupt save file - worst case, history starts over from
    /// zero rather than crashing the game.
    pub fn load() -> Self {
        fs::read_to_string(STATS_PATH)
            .ok()
            .and_then(|text| ron::de::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Writes the current history to STATS_PATH, creating saves/ first if
    /// needed. Silently does nothing on a write failure rather than
    /// panicking - same reasoning as Keymap::save. Called at the end of
    /// every record_* method below rather than left to callers to
    /// remember, so a stat is never recorded in memory only to be lost if
    /// the game closes before some later explicit save.
    fn save(&self) {
        let _ = fs::create_dir_all("saves");
        if let Ok(text) = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()) {
            let _ = fs::write(STATS_PATH, text);
        }
    }

    /// Call once when a run actually begins (see State::start_game) -
    /// counts as "this class was chosen" no matter how the run later
    /// ends (won, lost, or abandoned via quit-to-title).
    pub fn record_game_started(&mut self, class: &str) {
        self.games_played += 1;
        self.per_class
            .entry(class.to_string())
            .or_default()
            .games_played += 1;
        self.save();
    }

    /// Call once when a run ends in victory (see State::return_to_title).
    pub fn record_win(&mut self, class: &str) {
        self.games_won += 1;
        self.per_class
            .entry(class.to_string())
            .or_default()
            .games_won += 1;
        self.save();
    }

    /// Call whenever an enemy actually dies, wherever that happens to
    /// occur - a battle victory, a damage trap, or a ranged strike (see
    /// screens/battle.rs, systems/traps.rs, systems/use_items.rs).
    pub fn record_enemy_killed(&mut self, class: &str) {
        self.enemies_killed += 1;
        self.per_class
            .entry(class.to_string())
            .or_default()
            .enemies_killed += 1;
        self.save();
    }

    /// Call whenever a class-gated ability is actually used - a consumed
    /// battle Technique or an activated out-of-combat Effect that has a
    /// Class restriction. `class` should be the ABILITY's own class (from
    /// its Class component/template.ron `class:` field), not just
    /// whatever class the current player happens to be - the two always
    /// agree today (class-gated items can only be used by a matching
    /// player anyway), but keying off the item's own class is what makes
    /// that an invariant this function doesn't have to assume.
    pub fn record_ability_used(&mut self, class: &str, ability_name: &str) {
        let entry = self.per_class.entry(class.to_string()).or_default();
        *entry
            .ability_uses
            .entry(ability_name.to_string())
            .or_insert(0) += 1;
        self.save();
    }

    /// Call once when a run ends, however it ends - see
    /// State::return_to_title, the one place every run-ending path
    /// (victory, game over, or an early quit) funnels through. Only ever
    /// raises a class's recorded deepest_level, never lowers it, and
    /// skips the save() every other record_* does when the level isn't
    /// actually a new best - most runs don't set a new record, so this
    /// avoids a write on every single one of them.
    pub fn record_deepest_level(&mut self, class: &str, level: u32) {
        let entry = self.per_class.entry(class.to_string()).or_default();
        if level > entry.deepest_level {
            entry.deepest_level = level;
            self.save();
        }
    }
}
