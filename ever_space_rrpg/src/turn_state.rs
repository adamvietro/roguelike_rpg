#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TurnState {
    AwaitingInput,
    PlayerTurn,
    MonsterTurn,
    InBattle,
    GameOver,
    Victory,
    NextLevel,
}
