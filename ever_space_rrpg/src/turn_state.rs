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
    /// present (see systems/end_turn.rs). Currently a placeholder stub
    /// (see State::arena_transition_tick) - wave/boss orchestration past
    /// the starting shop hasn't been built yet.
    ArenaTransition,
    GameOver,
    Victory,
    NextLevel,
}
