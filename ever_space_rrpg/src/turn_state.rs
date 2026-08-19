#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    InBattle,
    BattleVictory,
    GameOver,
    Victory,
    NextLevel,
}
