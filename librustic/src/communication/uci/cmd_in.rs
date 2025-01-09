#[derive(Clone, Eq, PartialEq)]
pub enum UciIn {
    Uci,
    Quit,
    IsReady,
    Unknown(String),
}
