#[derive(Clone, Eq, PartialEq)]
pub enum XBoardIn {
    XBoard,
    Protover(u8),
    Ping(isize),
    New,
    Quit,
    Unknown(String),

    // Custom commands
    DebugOn,
    DebugOff,
}
