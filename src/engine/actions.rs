use super::{
    defs::{ErrFatal, ErrNormal},
    Engine,
};
use crate::{comm::CommControl, misc::parse};

// These actions are executed to act on incoming information.
impl Engine {
    // Received: CommReport::Move.
    pub fn execute_move(&mut self, m: String) {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let potential_move = parse::algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let is_pseudo_legal = self.pseudo_legal(potential_move, &self.board, &self.mg);

        // If the move is possibly legal, execute it and determine that the
        // king is not left in check.
        if let Ok(m) = is_pseudo_legal {
            let is_legal = self.board.lock().expect(ErrFatal::LOCK).make(m, &self.mg);
            if !is_legal {
                let msg = String::from(ErrNormal::MOVE_NOT_ALLOWED);
                self.comm.send(CommControl::Print(msg));
            }
        } else {
            let msg = String::from(ErrNormal::MOVE_NOT_LEGAL);
            self.comm.send(CommControl::Print(msg));
        }
    }

    pub fn takeback_move(&mut self) {
        let mut mtx_board = self.board.lock().expect(ErrFatal::LOCK);

        // First check if there are actually moves to unmake.
        if mtx_board.history.len() > 0 {
            mtx_board.unmake();
        } else {
            let msg = String::from(ErrNormal::NOTHING_TO_TAKE_BACK);
            self.comm.send(CommControl::Print(msg));
        }
    }
}
