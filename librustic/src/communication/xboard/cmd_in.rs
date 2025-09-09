use std::fmt;

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
    Force,
    Usermove(String),
    Quit,
    Unknown(String),

    // Custom commands
    State,
    Ignore(String),
    DebugOn,
    DebugOff,
    Board,
}

impl fmt::Display for XBoardIn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // XBoard Protocol
            XBoardIn::XBoard => write!(f, "xboard"),
            XBoardIn::Protover(n) => write!(f, "protover {}", n),
            XBoardIn::Ping(n) => write!(f, "ping {}", n),
            XBoardIn::New => write!(f, "new"),
            XBoardIn::SetBoard(s) => write!(f, "setboard {}", s),
            XBoardIn::Analyze => write!(f, "analyze"),
            XBoardIn::Exit => write!(f, "exit"),
            XBoardIn::Force => write!(f, "force"),
            XBoardIn::Usermove(m) => write!(f, "usermove {}", m),
            XBoardIn::Quit => write!(f, "quit"),
            XBoardIn::Unknown(s) => write!(f, "unknown {}", s),

            // Custom commands
            XBoardIn::State => write!(f, "state"),
            XBoardIn::Ignore(s) => write!(f, "ignore {}", s),
            XBoardIn::DebugOn => write!(f, "debug on"),
            XBoardIn::DebugOff => write!(f, "debug off"),
            XBoardIn::Board => write!(f, "board"),
        }
    }
}
