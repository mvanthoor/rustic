#[derive(Clone, Eq, PartialEq)]
pub enum XBoardOut {
    NewLine,
    XboardFeatures,
    Quit,
    Error(String, String),
    Custom(String),
}
