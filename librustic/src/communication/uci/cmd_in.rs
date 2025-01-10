#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    // UCI specification
    Uci,
    IsReady,
    UciNewGame,
    Position(String, Vec<String>),
    DebugOff,
    DebugOn,
    Quit,
    Unknown(String),

    // Custom
    Board,
}
