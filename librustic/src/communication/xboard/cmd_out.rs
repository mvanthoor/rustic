#[derive(Clone, Eq, PartialEq)]
pub enum XBoardOut {
    NewLine,
    Quit,
    Custom(String),
}
