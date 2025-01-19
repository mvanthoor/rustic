#[derive(Clone, Eq, PartialEq)]
pub enum XBoardIn {
    XBoard,
    Protover(u8),
    New,
    Quit,
    Unknown(String),
}
