#[derive(Clone, Eq, PartialEq)]
pub enum XBoardOut {
    NewLine,
    Features,
    Quit,
    Error(String, String),
    Custom(String),
}
