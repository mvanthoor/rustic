pub mod evaldefs;
pub mod material;

use crate::board::representation::Board;
use crate::defs::{BLACK, WHITE};

pub type EvalScore = i64;

// Evaluate the current position. Calculate the evaluation from White's point of
// view (shorthand: "pov"). Thus, a final score of +50 is good for white, -50 is
// good for black. At the end, if it is black that has just moved and is now
// evaluating, then swap the score. That way, the evaluation is always returned
// from the point of view of the side to move.
pub fn evaluate(board: &Board) -> EvalScore {
    let mut eval_score: i64 = 0;

    // The currently active color is the side to move next, as set by
    // playmove::make() after a legal move. Therefore, the OTHER side is
    // the one that has just moved, and is now evaluating.
    let side_that_moved = (board.game_state.active_color ^ 1) as usize;

    // Set the current point of view to either white or black.
    let pov_white = side_that_moved == WHITE;

    // Calculate evaluation parameters (These are all named p_...);
    let p_material = (board.material_count[WHITE] - board.material_count[BLACK]) as EvalScore;

    // Add up all the parameters.
    eval_score += p_material;

    // If it is white evaluating, then keep the score. Otherwise, swap to black POV.
    eval_score = if pov_white { eval_score } else { -eval_score };

    eval_score
}
