mod material;
mod psqt;

use crate::board::Board;

pub fn evaluate_position(board: &Board) -> i16 {
    // Base evaluation, by counting material.
    let material = material::count(board);
    let mut value = material.0 - material.1;

    // Gets a (white, black) tuple with PSQT values.
    let wb_psqt = psqt::apply(board);

    value += wb_psqt.0 - wb_psqt.1;

    value
}
