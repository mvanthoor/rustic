pub mod defs;
pub mod material;
mod psqt;

use crate::{board::Board, defs::Sides};

pub fn evaluate_position(board: &Board) -> i16 {
    let w_mc = board.material_count[Sides::WHITE];
    let b_mc = board.material_count[Sides::BLACK];

    // Starting value based on material count:
    let mut value = (w_mc - b_mc) as i16;

    // Apply PSQT adjustments.
    value += psqt::apply(board);

    value as i16
}
