#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
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
    GameOver,
    Victory,
    NextLevel,
}
