mod evaldefs;
mod material;

use crate::board::representation::Board;

pub type EvalScore = i64;

pub fn evaluate(board: &Board) -> EvalScore {
    let evaluation = material::count_difference(board);

    evaluation
}
