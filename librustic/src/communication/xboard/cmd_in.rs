#[derive(Clone, Eq, PartialEq)]
pub enum XBoardIn {
    XBoard,
    Quit,
    Unknown(String),
}
