pub enum UciOut {
    // UCI specification
    Id,
    ReadyOk,
    InfoString(String),
    Quit,

    // Custom
    PrintBoard,
}
