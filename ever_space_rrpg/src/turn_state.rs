#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
    ClassSelect,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    Paused,
    InBattle,
    BattleVictory,
    GameOver,
    Victory,
    NextLevel,
}
