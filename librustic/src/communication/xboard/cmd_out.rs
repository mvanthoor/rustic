#[derive(Clone, Eq, PartialEq)]
pub enum XBoardOut {
    // Xboard specification
    NewLine,
    Features,
    Pong(isize),
    Quit,
    Error(String, String),

    // Custom output
    Custom(String),
}
