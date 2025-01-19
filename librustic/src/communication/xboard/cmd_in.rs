#[derive(Clone, Eq, PartialEq)]
pub enum XBoardIn {
    XBoard,
    New,
    Quit,
    Unknown(String),
}
