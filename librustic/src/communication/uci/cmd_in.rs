#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    // UCI specification
    Uci,
    IsReady,
    UciNewGame,
    Quit,
    Unknown(String),

    // Custom
    Board,
}
