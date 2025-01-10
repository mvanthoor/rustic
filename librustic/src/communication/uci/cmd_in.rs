use crate::search::defs::GameTime;

#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    // UCI specification
    Uci,
    IsReady,
    UciNewGame,
    Position(String, Vec<String>),
    GoInfinite,
    GoDepth(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    DebugOff,
    DebugOn,
    Quit,
    Unknown(String),

    // Custom
    Board,
}
