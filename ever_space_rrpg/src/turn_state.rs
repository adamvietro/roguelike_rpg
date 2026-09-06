#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
    /// Dungeon Crawl vs. Battle Arena - reached from TitleScreen (any
    /// key), leads into ClassSelect. See screens/title.rs's
    /// adventure_select and State::adventure_mode.
    AdventureSelect,
    ClassSelect,
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
}
