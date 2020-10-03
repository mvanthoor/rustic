use crate::{board::Board, defs::Sides, misc::bits};

pub const PIECE_VALUES_MG: [i16; 6] = [0, 900, 500, 320, 310, 100];

pub fn count(board: &Board) -> (i16, i16) {
    let mut white_material: i16 = 0;
    let mut black_material: i16 = 0;
    let bb_w = board.bb_pieces[Sides::WHITE];
    let bb_b = board.bb_pieces[Sides::BLACK];

    for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
        let mut white_pieces = *w; // White pieces of "piece_type"
        let mut black_pieces = *b; // black pieces of "piece_type"

        while white_pieces > 0 {
            white_material += PIECE_VALUES_MG[piece_type];
            bits::next(&mut white_pieces);
        }

        while black_pieces > 0 {
            black_material += PIECE_VALUES_MG[piece_type];
            bits::next(&mut black_pieces);
        }
    }

    (white_material, black_material)
}
