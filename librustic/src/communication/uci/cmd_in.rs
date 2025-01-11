use crate::communication::uci::defs::{FenString, Moves, Name, Value};
use crate::search::defs::GameTime;

#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    // UCI specification
    Uci,
    IsReady,
    UciNewGame,
    Position(FenString, Moves),
    SetOption(Name, Value),
    GoInfinite,
    GoDepth(i8),
    GoMoveTime(u128),
    GoNodes(usize),
    GoGameTime(GameTime),
    DebugOff,
    DebugOn,
    Stop,
    Quit,
    Unknown(String),
}
