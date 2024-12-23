use crate::{
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    engine::{defs::ErrFatal, Engine},
    misc::parse,
    misc::parse::ConvertedMove,
    movegen::defs::allocate_move_list_memory,
    movegen::defs::{Move, MoveType},
};
use if_chain::if_chain;

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
            .fen_setup(Some(fen))?;

        Ok(())
    }

    // This function executes a move on the internal board, if it legal to
    // do so in the given position.
    pub fn execute_move(&mut self, m: &str) -> bool {
        // Prepare shorthand variables.
        let empty_move = (0usize, 0usize, 0usize);
        let converted_move = parse::algebraic_move_to_numbers(m).unwrap_or(empty_move);
        let pseudo_legal_move = self.is_pseudo_legal_move(converted_move);

        if let Some(m) = pseudo_legal_move {
            return self.board.lock().expect(ErrFatal::LOCK).make(m, &self.mg);
        } else {
            false
        }
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually in the list of pseudo-legal moves for this position.
    pub fn is_pseudo_legal_move(&self, converted_move: ConvertedMove) -> Option<Move> {
        // Get the pseudo-legal move list for this position.
        let mut memory = allocate_move_list_memory();
        let move_list = self.mg.generate_moves(
            &self.board.lock().expect(ErrFatal::LOCK),
            &mut memory,
            MoveType::All,
        );

        // Determine if the converted move is a possible pseudo-legal move
        // in the current position. make() wil determine final legality
        // when executing the move.
        for i in 0..move_list.len() {
            let m = move_list.get_move(i);
            if_chain! {
                if converted_move.0 == m.from();
                if converted_move.1 == m.to();
                if converted_move.2 == m.promoted();
                then {
                    return Some(m);
                }
            }
        }

        None
    }
}
