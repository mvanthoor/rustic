pub mod defs;
pub mod material;
pub mod psqt;

use crate::{board::Board, defs::Sides};

pub fn evaluate_position(board: &Board) -> i16 {
    let side = board.game_state.active_color as usize;
    let w_material = board.game_state.material[Sides::WHITE];
    let b_material = board.game_state.material[Sides::BLACK];

    // Base evaluation, by counting material.
    let mut value = (w_material - b_material) as i16;

    // Add PSQT values
    value += board.game_state.psqt[Sides::WHITE] - board.game_state.psqt[Sides::BLACK];

    // This function calculates the evaluation from white's point of view:
    // a positive value means "white is better", a negative value means
    // "black is better". Alpha/Beta requires the value returned from the
    // viewpoint of the side that is being evaluated. Therefore if it is
    // black to move, the value must first be flipped to black's viewpoint
    // before it can be returned.

    value = if side == Sides::BLACK { -value } else { value };

    value
}
