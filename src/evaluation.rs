pub mod defs;
pub mod material;
mod psqt;

use crate::{board::Board, defs::Sides};

pub fn evaluate_position(board: &Board) -> i16 {
    let w_material = board.material_count[Sides::WHITE];
    let b_material = board.material_count[Sides::BLACK];

    // Base evaluation, by counting material.
    let mut value = (w_material - b_material) as i16;

    // Gets a (white, black) tuple with PSQT values.
    let wb_psqt = psqt::apply(board);

    value += wb_psqt.0 - wb_psqt.1;

    value
}
