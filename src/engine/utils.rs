use super::{defs::ErrFatal, Engine};
use crate::{
    board::Board,
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    misc::parse,
    misc::parse::PotentialMove,
    movegen::{
        defs::{Move, MoveList, MoveType},
        MoveGenerator,
    },
};
use if_chain::if_chain;
use std::sync::Mutex;

impl Engine {
    // This function sets up a position using a given FEN-string.
    pub fn setup_position(&mut self) -> EngineRunResult {
        // Get either the provided FEN-string or KiwiPete. If both are
        // provided, the KiwiPete position takes precedence.
        let f = &self.cmdline.fen()[..];
        let kp = self.cmdline.has_kiwipete();
        let fen = if kp { FEN_KIWIPETE_POSITION } else { f };

        // Lock the board, setup the FEN-string, and drop the lock.
        self.board
            .lock()
            .expect(ErrFatal::LOCK)
            .fen_read(Some(fen))?;

        Ok(())
    }

    pub fn execute_move(&mut self, m: String) -> bool {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let potential_move = parse::algebraic_move_to_number(&m[..]).unwrap_or(empty);
        let is_pseudo_legal = self.pseudo_legal(potential_move, &self.board, &self.mg);
        let mut is_legal = false;

        if let Ok(ips) = is_pseudo_legal {
            is_legal = self.board.lock().expect(ErrFatal::LOCK).make(ips, &self.mg);
        }
        is_legal
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually possible in the current board position.
    pub fn pseudo_legal(
        &self,
        m: PotentialMove,
        board: &Mutex<Board>,
        mg: &MoveGenerator,
    ) -> Result<Move, ()> {
        let mut result = Err(());
        let mut ml = MoveList::new();
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        mg.generate_moves(&mtx_board, &mut ml, MoveType::All);
        std::mem::drop(mtx_board);

        // See if the provided potential move is actually a possible
        // pseudo-legal move. make() will later determine final legality,
        // i.e. if the king is left in check.
        for i in 0..ml.len() {
            let current = ml.get_move(i);
            if_chain! {
                if m.0 == current.from();
                if m.1 == current.to();
                if m.2 == current.promoted();
                then {
                    result = Ok(current);
                    break;
                }
            }
        }
        result
    }
}
