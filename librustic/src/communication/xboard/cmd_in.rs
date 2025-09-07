#[derive(Clone, Eq, PartialEq)]
pub enum XBoardIn {
    XBoard,
    Protover(u8),
    Ping(isize),
    New,
    SetBoard(String),
    Quit,
    Unknown(String),

    // Custom commands
    State,
    Ignore(String),
    DebugOn,
    DebugOff,
    Board,
}
