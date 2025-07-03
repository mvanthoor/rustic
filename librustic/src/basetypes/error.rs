pub struct ErrNormal;
impl ErrNormal {
    pub const NOT_LEGAL: &'static str = "This is not a legal move in this position";
    pub const NOT_INT: &'static str = "The value given was not an integer";
    pub const FEN_FAILED: &'static str = "Setting up FEN failed";
    pub const UNKNOWN_COMMAND: &'static str = "Unknown command";
    pub const COMMAND_INVALID: &'static str = "Command invalid in current engine state";
    pub const INCORRECT_FEN: &'static str = "Incorrect FEN-string";
    pub const TIME_CONTROL_NOT_SET: &'static str = "Time control not set";
}

// This struct holds messages that are reported on fatal engine errors.
// These should never happen; if they do the engine is in an unknown state,
// and it will panic without trying any recovery whatsoever.
pub struct ErrFatal;
impl ErrFatal {
    pub const CREATE_COMM: &'static str = "Comm creation failed.";
    pub const NEW_GAME: &'static str = "Setting up new game failed.";
    pub const LOCK: &'static str = "Lock failed.";
    pub const READ_IO: &'static str = "Reading I/O failed.";
    pub const HANDLE: &'static str = "Broken handle.";
    pub const THREAD: &'static str = "Thread has failed.";
    pub const CHANNEL: &'static str = "Broken channel.";
    pub const NO_INFO_RX: &'static str = "No incoming Info channel.";
    pub const GENERATED_ILLEGAL_MOVE: &'static str = "The engine generated an illegal move!";
}

pub struct ErrUci;
impl ErrUci {
    pub const UNKNOWN_CMD: &str = "Unknown command";
    pub const OPTION_UNKNOWN_NAME: &str = "Unknown option name";
    pub const OPTION_NO_NAME: &str = "Option must have a name";
    pub const OPTION_NO_VALUE: &str = "Value is required for option";
    pub const OPTION_VALUE_NOT_INT: &str = "Value must be integer for option";
}

pub struct ErrXboard;
impl ErrXboard {
    pub const UNKNOWN_CMD: &str = "Unknown command";
}
