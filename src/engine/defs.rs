use crate::comm::CommReport;
// This struct holds messages that are reported on fatal engine errors.
// These should never happen; if they do the engine is in an unknown state,
// and it will panic without trying any recovery whatsoever.
pub struct ErrFatal;
impl ErrFatal {
    pub const CREATE_COMM: &'static str = "Comm creation failed.";
    pub const LOCK: &'static str = "Lock failed.";
    pub const READ_IO: &'static str = "Reading I/O failed.";
    pub const FLUSH_IO: &'static str = "Flushing I/O failed.";
    pub const HANDLE: &'static str = "Broken handle.";
    pub const THREAD: &'static str = "Thread has failed.";
    pub const CHANNEL: &'static str = "Broken channel.";
    pub const NO_INFO_RX: &'static str = "No incoming Info channel.";
}

pub struct ErrNormal;
impl ErrNormal {
    pub const MOVE_NOT_ALLOWED: &'static str = "Move not allowed: King left in check.";
    pub const MOVE_NOT_LEGAL: &'static str = "This is not a legal move.";
    pub const NOTHING_TO_TAKE_BACK: &'static str = "Nothing to take back.";
}

// This struct holds the engine's settings.
pub struct Settings {
    pub threads: usize,
}

// This enum provides informatin to the engine, with regard to incoming
// messages and search results.
#[derive(PartialEq)]
pub enum Information {
    Comm(CommReport),
}
