#[derive(Clone, Eq, PartialEq)]
pub enum XBoardOut {
    NewLine,
    Features,
    Pong(isize),
    Quit,
    Error(String, String),
    Custom(String),
}
