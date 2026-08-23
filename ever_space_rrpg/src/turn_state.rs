#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    TitleScreen,
    ClassSelect,
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    InBattle,
    BattleVictory,
    GameOver,
    Victory,
    NextLevel,
}
