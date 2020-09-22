pub mod console;
// pub mod uci;
// pub mod xboard;

use crate::{board::Board, misc::parse};
use std::sync::{Arc, Mutex};

// These are the types of communication the engine is capable of.
pub struct CommType {}
impl CommType {
    // pub const UCI: &'static str = "uci";
    // pub const XBOARD: &'static str = "xboard";
    pub const CONSOLE: &'static str = "console";
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn print_before_read(&self, board: Arc<Mutex<Board>>);
    fn read(&self) -> Incoming;
    fn get_protocol_name(&self) -> &'static str;
}

// If one of those errors occurs, something is wrong with the engine or one
// of its threads, and the program will panic, displaying these messages.
pub struct ErrFatal {}
impl ErrFatal {
    const LOCK_BOARD: &'static str = "Board lock failed.";
    const READ_IO: &'static str = "Reading I/O failed.";
    const FLUSH_IO: &'static str = "Flushing I/O failed.";
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq, Clone)]
pub enum Incoming {
    NoCmd,
    Quit,
    Move(String),
}

impl Incoming {
    pub fn is_correct(&self) -> bool {
        // Match the incoming command.
        match self {
            // Some commands don't need to be verified.
            Self::NoCmd | Self::Quit => true,
            // Check if squares and promotion piece actually exist.
            Self::Move(m) => parse::algebraic_move_to_number(&m[..]).is_ok(),
        }
    }
}
