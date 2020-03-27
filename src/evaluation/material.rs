use super::evaldefs::PIECE_VALUES;
use super::EvalScore;
use crate::board::representation::Board;
use crate::utils::next;

pub fn count_difference(board: &Board) -> EvalScore {
    let mut white_value: i64 = 0;
    let mut black_value: i64 = 0;

    for (piece, (bb_w, bb_b)) in board.bb_w.iter().zip(board.bb_b.iter()).enumerate() {
        let mut white_pieces = *bb_w;
        let mut black_pieces = *bb_b;

        while white_pieces > 0 {
            white_value += PIECE_VALUES[piece];
            next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_value += PIECE_VALUES[piece];
            next(&mut black_pieces);
        }
    }

    white_value - black_value
}
