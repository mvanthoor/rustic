pub mod console;
pub mod uci;
pub mod xboard;

use crate::{board::Board, movegen::MoveGenerator};

pub trait IComm {
    fn start(&mut self, board: &mut Board, _mg: &MoveGenerator);
}
