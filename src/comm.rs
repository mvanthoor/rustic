pub mod console;
// pub mod uci;
// pub mod xboard;

use crate::{board::Board, misc::parse};
use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

pub trait IComm {
    fn start(&self, board: Arc<Mutex<Board>>) -> JoinHandle<()>;
}

pub struct ErrComm {}
impl ErrComm {
    const LOCK_BOARD: &'static str = "Comm: Board lock failed.";
    const READ_IO: &'static str = "Comm: Reading I/O failed.";
}

#[derive(PartialEq)]
pub enum Command {
    NoCmd,
    Quit,
    Move(String),
}

impl Command {
    pub fn is_correct(&self) -> bool {
        // Match the incoming command.
        match self {
            // Some commands don't need to be verified.
            Command::NoCmd | Command::Quit => true,
            // Make sure that the move actually contains existing squares
            // and either none, or an existing promotion piece.
            Command::Move(m) => parse::algebraic_move_to_number(&m[..]).is_ok(),
        }
    }
}
