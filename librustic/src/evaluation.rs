pub mod defs;
mod phase;
mod psqt;
mod weights;

use crate::{board::Board, defs::Sides};

pub struct Evaluation;
impl Evaluation {
    pub fn evaluate_position(board: &Board) -> i16 {
        // Determine the side which is evaluating.
        let side = board.game_state.active_color as usize;

        // Establish base evaluation value by PST score.
        let mut value = Evaluation::psqt_score(board);

        // Flip point of view if black is evaluating.
        value = if side == Sides::BLACK { -value } else { value };

        value
    }
}
