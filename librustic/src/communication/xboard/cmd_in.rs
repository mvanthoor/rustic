use strum_macros::Display as StrumDisplay;

#[derive(Clone, Eq, PartialEq, Debug, StrumDisplay)]
pub enum XBoardIn {
    XBoard,
    Protover(u8),
    Ping(isize),
    New,
    SetBoard(String),
    Analyze,
    Exit,
    Quit,
    Unknown(String),

    // Custom commands
    State,
    Ignore(String),
    DebugOn,
    DebugOff,
    Board,
}
