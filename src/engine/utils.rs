use crate::{
    board::Board,
    defs::{EngineRunResult, FEN_KIWIPETE_POSITION},
    engine::{defs::ErrFatal, Engine},
    misc::parse,
    misc::parse::ConvertedMove,
    movegen::defs::{Move, MoveType},
    movegen::{defs::allocate_move_list_memory, MoveGenerator},
};
use if_chain::if_chain;
use std::{error::Error, fmt, sync::Mutex};

#[derive(Debug)]
pub enum PseudoLegalMoveError {
    NotPseudoLegalMove,
}

impl fmt::Display for PseudoLegalMoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Self::NotPseudoLegalMove => "Not a pseudo-legal move in this position",
        };
        write!(f, "{error}")
    }
}

impl Error for PseudoLegalMoveError {}

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
    // TODO: Refactor this to accept a reference instead of a String
    pub fn execute_move(&mut self, m: String) -> bool {
        // Prepare shorthand variables.
        let empty = (0usize, 0usize, 0usize);
        let converted_move = parse::algebraic_move_to_numbers(m.as_str()).unwrap_or(empty);
        let is_pseudo_legal = self.pseudo_legal(converted_move, &self.board, &self.mg);
        let mut is_legal = false;

        if let Ok(ips) = is_pseudo_legal {
            is_legal = self.board.lock().expect(ErrFatal::LOCK).make(ips, &self.mg);
        }
        is_legal
    }

    // After the engine receives an incoming move, it checks if this move
    // is actually in the list of pseudo-legal moves for this position.
    pub fn pseudo_legal(
        &self,
        converted_move: ConvertedMove,
        board: &Mutex<Board>,
        mg: &MoveGenerator,
    ) -> Result<Move, PseudoLegalMoveError> {
        // Get the pseudo-legal move list for this position.
        let mut memory = allocate_move_list_memory();
        let mtx_board = board.lock().expect(ErrFatal::LOCK);
        let ml = mg.generate_moves(&mtx_board, &mut memory, MoveType::All);
        std::mem::drop(mtx_board);

        // Determine if the potential move is pseudo-legal. make() wil
        // determine final legality when executing the move.
        for i in 0..ml.len() {
            let m = ml.get_move(i);
            if_chain! {
                if converted_move.0 == m.from();
                if converted_move.1 == m.to();
                if converted_move.2 == m.promoted();
                then {
                    return Ok(m);
                }
            }
        }

        Err(PseudoLegalMoveError::NotPseudoLegalMove)
    }
}
