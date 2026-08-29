#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
    ClassSelect,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    Paused,
    /// Rebindable-keys screen - reached from Paused (press O), returns to
    /// Paused on Escape. See screens/options.rs and keymap.rs.
    Options,
    InBattle,
    BattleVictory,
    GameOver,
    Victory,
    NextLevel,
}
