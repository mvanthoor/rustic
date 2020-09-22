pub mod console;
// pub mod uci;
// pub mod xboard;

use crate::{board::Board, misc::parse};
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

// These are the types of communication the engine is capable of.
pub struct CommType {}
impl CommType {
    // pub const UCI: &'static str = "uci";
    // pub const XBOARD: &'static str = "xboard";
    pub const CONSOLE: &'static str = "console";
}

// Defines the public functions a Comm module must implement.
pub trait IComm {
    fn start(&self, board: Arc<Mutex<Board>>) -> JoinHandle<()>;
}

// If one of those errors occurs, something is wrong with the engine or one
// of its threads, and the program will panic, displaying these messages.
pub struct ErrFatal {}
impl ErrFatal {
    const LOCK_BOARD: &'static str = "Comm: Board lock failed.";
    const READ_IO: &'static str = "Comm: Reading I/O failed.";
    const FLUSH_IO: &'static str = "Comm: Flushing I/O failed.";
}

// These are the commands a Comm module can create and send back to the
// engine in the main thread.
#[derive(PartialEq)]
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
            // Make sure that the move actually contains existing squares
            // and either none, or an existing promotion piece. The engine
            // itself is responsible for actually verifying that the move
            // is possible and legal.
            Self::Move(m) => parse::algebraic_move_to_number(&m[..]).is_ok(),
        }
    }
}
