#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    // UCI specification
    Uci,
    IsReady,
    UciNewGame,
    DebugOff,
    DebugOn,
    Quit,
    Unknown(String),

    // Custom
    Board,
}
