#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
    /// Dungeon Crawl vs. Battle Arena - reached from TitleScreen (any
    /// key), leads into ClassSelect. See screens/title.rs's
    /// adventure_select and State::adventure_mode.
    AdventureSelect,
    ClassSelect,
    /// Debug-class-only, Dungeon-Crawl-only (added 2026-09-13): lets the
    /// user force which `MapTheme` (Forest/Dungeon/Sewer/Swamp, or Random for
    /// the normal per-floor roll) every floor of the upcoming run uses,
    /// instead of re-rolling randomly each floor - a testing convenience
    /// so a specific theme's Victory/Defeat art (see components::
    /// VictoryBackground/DefeatBackground) can be reached without
    /// restarting runs repeatedly. Reached from ClassSelect's hidden 'D'
    /// shortcut instead of going straight to start_game; leads into
    /// AwaitingInput via start_game once a choice is confirmed. See
    /// screens/title.rs's theme_select and components::ThemeChoice.
    ThemeSelect,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    Paused,
    /// The Item Menu (press M during dungeon exploration) - a browsable,
    /// cursor-navigable list of universal consumables (Healing Potion,
    /// Dungeon Map, any future item every class can carry). Reached from
    /// AwaitingInput, returns there on Escape (no turn consumed) or after
    /// using an item (PlayerTurn - see screens/item_menu.rs, which DOES
    /// consume a turn). Class-restricted abilities (Trap, Throw Spear,
    /// ...) live on the Ability Bar instead - see components::
    /// ability_bar_slots/systems/hud.rs - and are triggered directly by
    /// number key without ever opening this menu.
    ItemMenu,
    /// Rebindable-keys screen - reached from Paused (press O) or the
    /// title screen (press O), returns there on Escape. See
    /// screens/options.rs and keymap.rs.
    Options,
    /// Play-history screen - reached from the title screen (press H),
    /// returns there on Escape. See screens/stats_view.rs and stats.rs.
    StatsView,
    InBattle,
    BattleVictory,
    /// One-shot, like NextLevel - reached instead of NextLevel when the
    /// player steps on an Exit tile while an ArenaRun resource is
    /// present (see systems/end_turn.rs). Always means "leave the shop,
    /// start level `run.level`'s wave 1" (see State::arena_transition_tick) -
    /// distinct from ArenaWaveCleared below, which means "a wave/boss
    /// encounter already in progress just lost its last enemy."
    ArenaTransition,
    /// Reached when systems/end_turn.rs detects the last Enemy died while
    /// an Arena wave or boss encounter was active AND the kill did NOT go
    /// through the normal battle-victory screen (a Throw Spear/Shoot
    /// ranged strike, or a placed Trap - see systems/use_items.rs and
    /// systems/traps.rs). Those two kill paths just remove the enemy
    /// entity and record a stat; nothing else was watching for "was that
    /// the last one?", so without this the wave/level would silently
    /// never advance if its last enemy happened to die that way instead
    /// of through a real battle. See State::arena_wave_cleared_tick,
    /// which reuses the exact same handle_arena_kill orchestration the
    /// battle-victory path already uses - the two states are just
    /// different TRIGGERS for identical wave/boss/level logic.
    ArenaWaveCleared,
    GameOver,
    Victory,
    NextLevel,
    /// Reached by stepping on a dungeon floor's stairs tile in Dungeon
    /// Crawl mode (ArenaRun is None) while NOT already browsing the shop
    /// this leads to (ShoppingActive is None too) - see
    /// systems/end_turn.rs's Exit-tile check, which is a 3-way split now:
    /// ArenaTransition (ArenaRun present), NextLevel (ArenaRun absent but
    /// ShoppingActive present - leaving the shop this state itself
    /// built), or this. One-shot, like NextLevel/ArenaTransition -
    /// State::dungeon_shop_transition builds the shop room once then
    /// moves straight to AwaitingInput with ShoppingActive set.
    DungeonShopTransition,
    /// Reached when the player walks onto a dungeon chest (see
    /// components::Chest / systems/movement.rs) - a full-screen overlay
    /// styled exactly like Paused (see State::build_pause_scheduler's
    /// map-only redraw and State::chest_loot_tick), listing what the
    /// chest granted (Option<ChestLoot>) until dismissed with Enter, then
    /// back to AwaitingInput. Dungeon Crawl only - Battle Arena has no
    /// chests.
    ChestOpened,
}
