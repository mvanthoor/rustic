use super::evaldefs::PIECE_VALUES;
use super::EvalScore;
use crate::board::representation::Board;
use crate::defs::{BLACK, WHITE};
use crate::utils::bits;

pub fn count_difference(board: &Board) -> EvalScore {
    let mut white_value: i64 = 0;
    let mut black_value: i64 = 0;
    let bb_w = board.bb_side[WHITE];
    let bb_b = board.bb_side[BLACK];

    for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        let mut white_pieces = *w;
        let mut black_pieces = *b;

        while white_pieces > 0 {
            white_value += PIECE_VALUES[piece];
            bits::next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_value += PIECE_VALUES[piece];
            bits::next(&mut black_pieces);
        }
    }

    white_value - black_value
}
