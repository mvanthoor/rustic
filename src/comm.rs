pub mod console;
// pub mod uci;
// pub mod xboard;

use crate::{board::Board, engine::Information, misc::parse};
use crossbeam_channel::Sender;
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
    fn activate(
        &mut self,
        report_tx: Sender<Information>,
        board: Arc<Mutex<Board>>,
    ) -> Sender<CommControl>;
    fn wait_for_shutdown(&mut self);
    fn get_protocol_name(&self) -> &'static str;
}

// If one of those errors occurs, something is wrong with the engine or one
// of its threads, and the program will panic, displaying these messages.
pub struct ErrFatal {}
impl ErrFatal {
    const LOCK_BOARD: &'static str = "Board lock failed.";
    const READ_IO: &'static str = "Reading I/O failed.";
    const FLUSH_IO: &'static str = "Flushing I/O failed.";
    const BROKEN_HANDLE: &'static str = "Broken handle.";
    const FAILED_THREAD: &'static str = "Thread has failed.";
}

#[derive(PartialEq, Clone)]
pub enum CommControl {
    Update,
    Quit,
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq, Clone)]
pub enum CommReport {
    Quit,
    Search,
    Move(String),
}

impl CommReport {
    pub fn is_valid(&self) -> bool {
        // Match the incoming command.
        match self {
            // Some commands don't need to be verified.
            Self::Quit | Self::Search => true,
            // Check if squares and promotion piece actually exist.
            Self::Move(m) => parse::algebraic_move_to_number(&m[..]).is_ok(),
        }
    }
}
