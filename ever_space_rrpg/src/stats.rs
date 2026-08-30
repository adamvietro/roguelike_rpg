use crate::arena::AdventureMode;
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
    /// raised, never lowered - see record_deepest_level. Dungeon Crawl
    /// only - see arena_furthest_level/arena_furthest_wave for the Arena
    /// equivalent, deliberately kept as a separate number rather than
    /// merged into this one.
    pub deepest_level: u32,
    /// Furthest Battle Arena level *reached* with this class, 1..=3 (0 =
    /// never reached any Arena wave with this class yet). "Reached" means
    /// the player was dropped into that wave, not that they cleared it -
    /// see record_arena_progress. Always used together with
    /// arena_furthest_wave; the pair (level, wave) is the unit of
    /// progress, not this field alone.
    #[serde(default)]
    pub arena_furthest_level: u8,
    /// Furthest wave *reached* within arena_furthest_level, 1..=3.
    /// Deliberately ignores whether a boss was up at the time (per the
    /// user's own call: wave granularity only, don't track boss state) -
    /// reaching wave 3 while its boss is mid-fight still just reads as
    /// "Level N, Wave 3". Meaningless (0) when arena_furthest_level is 0.
    #[serde(default)]
    pub arena_furthest_wave: u8,
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
    /// Total Dungeon Crawl runs started, any class - the "y" in the
    /// History screen's "Dungeon Crawl: x of y" line. Battle Arena runs
    /// no longer feed this (see record_game_started) - previously they
    /// did, silently inflating this number; a stats.ron saved before this
    /// fix will have that inflation baked into its history and won't be
    /// retroactively corrected.
    pub games_played: u32,
    /// Total Dungeon Crawl runs that ended in victory, any class - the
    /// "x" in "x of y". See games_played's note on the same Arena/Dungeon
    /// split.
    pub games_won: u32,
    /// Battle Arena counterpart to games_played/games_won - a genuinely
    /// separate counter, not blended with the Dungeon Crawl numbers
    /// above. Drives the History screen's parallel "Battle Arena: x of y"
    /// line.
    #[serde(default)]
    pub arena_games_played: u32,
    #[serde(default)]
    pub arena_games_won: u32,
    pub enemies_killed: u32,
    /// Per-class win/loss counts here are deliberately left blended
    /// across both modes (same as today) - only the top-level counters
    /// above are split. Ability usage is likewise unified/shared across
    /// modes on purpose - see ClassStats::ability_uses.
    pub per_class: HashMap<String, ClassStats>,
    /// How many times each UNRESTRICTED item has been used (no Class
    /// component - Healing Potion, Dungeon Map, and any future universal
    /// consumable). Deliberately top-level, not per-class: these items
    /// aren't gated to any one class in the first place, so splitting
    /// them per-class the way ability_uses is would just duplicate the
    /// same numbers under whichever class happened to be played most.
    /// Keyed by the item's own Name, same convention as ability_uses. See
    /// record_item_used / the History screen's Items Used sub-view.
    #[serde(default)]
    pub item_uses: HashMap<String, u32>,
}

/// Which sub-view the History screen (screens/stats_view.rs) is currently
/// showing - replaces the old bare `Option<String>` (`stats_selected_class`)
/// now that there are two different drill-downs from the overview instead
/// of just one. Kept as a plain `State` field (like `adventure_mode`), not
/// a resource - it's screen-navigation state, not something any gameplay
/// system needs to see.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatsViewMode {
    /// The table of all classes plus the two Dungeon/Arena summary lines.
    Overview,
    /// Drill-down into one class's ability_uses, reached by pressing that
    /// class's row number from Overview.
    ClassAbilities(String),
    /// Drill-down into the global item_uses list, reached by pressing I
    /// from Overview.
    ItemUsage,
}

impl Default for StatsViewMode {
    fn default() -> Self {
        StatsViewMode::Overview
    }
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

    /// Call once when a run actually begins (see State::start_game /
    /// State::start_arena) - counts as "this class was chosen" no matter
    /// how the run later ends (won, lost, or abandoned via
    /// quit-to-title). `mode` decides which TOP-LEVEL counter this feeds
    /// (Dungeon Crawl's games_played vs Arena's arena_games_played) - the
    /// per-class counter below is intentionally blended across both
    /// modes regardless of `mode`, matching how ability_uses is already
    /// unified.
    pub fn record_game_started(&mut self, class: &str, mode: AdventureMode) {
        match mode {
            AdventureMode::DungeonCrawl => self.games_played += 1,
            AdventureMode::BattleArena => self.arena_games_played += 1,
        }
        self.per_class
            .entry(class.to_string())
            .or_default()
            .games_played += 1;
        self.save();
    }

    /// Call once when a run ends in victory (see State::return_to_title).
    /// Same Dungeon/Arena top-level split as record_game_started, same
    /// deliberately-blended per-class counter.
    pub fn record_win(&mut self, class: &str, mode: AdventureMode) {
        match mode {
            AdventureMode::DungeonCrawl => self.games_won += 1,
            AdventureMode::BattleArena => self.arena_games_won += 1,
        }
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

    /// Call whenever an UNRESTRICTED item (no Class component - Healing
    /// Potion, Dungeon Map) is actually consumed - see systems/use_items.rs,
    /// the counterpart branch to record_ability_used for items that don't
    /// belong to any one class. Top-level only; see item_uses's own doc
    /// comment for why this isn't split per-class.
    pub fn record_item_used(&mut self, item_name: &str) {
        *self.item_uses.entry(item_name.to_string()).or_insert(0) += 1;
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

    /// Arena counterpart to record_deepest_level - call once when a
    /// Battle Arena run ends, however it ends (see
    /// State::return_to_title), with the run's live `level`/`wave` at
    /// that moment. Records "reached", not "cleared" - a run that quits
    /// or ends mid-wave 2 still counts wave 2 as reached, per the user's
    /// call to track by wave and ignore completion. Deliberately ignores
    /// boss_active entirely: reaching wave 3 while its boss is up still
    /// just reads as "Level N, Wave 3", exactly like reaching wave 3
    /// before the boss spawns. Only ever raises the stored (level, wave)
    /// pair, compared as a single level-major ordinal (level * 4 + wave -
    /// the multiplier is 4 rather than the "expected" 3 so wave's max
    /// value of 3 can never collide with the next level's own baseline,
    /// even though the caller is only ever expected to pass wave >= 1) so
    /// a later level always outranks an earlier one regardless of wave,
    /// and a later wave within the same level outranks an earlier wave. A
    /// `wave` of 0 (still in a level's shop, no wave started yet) is
    /// never passed in by the caller - see the caller's own check.
    pub fn record_arena_progress(&mut self, class: &str, level: u8, wave: u8) {
        let entry = self.per_class.entry(class.to_string()).or_default();
        let new_ordinal = level as u32 * 4 + wave as u32;
        let old_ordinal = entry.arena_furthest_level as u32 * 4 + entry.arena_furthest_wave as u32;
        if new_ordinal > old_ordinal {
            entry.arena_furthest_level = level;
            entry.arena_furthest_wave = wave;
            self.save();
        }
    }
}
