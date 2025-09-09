#[derive(Clone, Eq, PartialEq, Debug)]
pub enum XBoardIn {
    // XBoard protocol
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

impl std::fmt::Display for XBoardIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XBoardIn::XBoard => write!(f, "XBoard"),
            XBoardIn::Protover(version) => write!(f, "protover {}", version),
            XBoardIn::Ping(id) => write!(f, "ping {}", id),
            XBoardIn::New => write!(f, "new"),
            XBoardIn::SetBoard(board) => write!(f, "setboard {}", board),
            XBoardIn::Analyze => write!(f, "analyze"),
            XBoardIn::Exit => write!(f, "exit"),
            XBoardIn::Quit => write!(f, "quit"),
            XBoardIn::Unknown(command) => write!(f, "unknown {}", command),
            XBoardIn::State => write!(f, "state"),
            XBoardIn::Ignore(ignore) => write!(f, "ignore {}", ignore),
            XBoardIn::DebugOn => write!(f, "debug on"),
            XBoardIn::DebugOff => write!(f, "debug off"),
            XBoardIn::Board => write!(f, "board"),
        }
    }
}
