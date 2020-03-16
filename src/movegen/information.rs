use super::MoveGenerator;
use crate::board::representation::Board;
use crate::defs::{Side, BISHOP, FILE_A, FILE_H, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE};

/**
 * square_attacked reports true or false regarding the question if a square is attacekd by the
 * given side. It works using this notion:
 *      - If a PIECE on SQ1 is attacking SQ2, then that PIECE on SQ2 is also attacking SQ1
 * What this means is that if a knight on e4 attacks d2, a knight on d2 also attacks e4.
 * Taking the knight as an example:
 * We generate the moves of the knight, when set on the given square. This is bb_knight.
 * We then check if the given side has a knight on one of those squares:
 *      bb_knight & pieces[KNIGHT] > 0
 * Do this for all of the pieces.
 * The queen does not have her own check. She is a combination of the rook and the bishop.
 * The queen can see what the rook can see, and she can see what the bishop can see.
 * So, when checking to find a rook, we also check for the queen; same with the bishop.
 * For pawns, we calculate on which squares a pawn should be to attack the given square,
 *  and then check if the given side actually has at least one pawn on one of those squares.
 * "pieces" and "pawns" are obviously dependent on the side we're looking at.
 */
pub fn square_attacked(board: &Board, side: Side, mg: &MoveGenerator, square: u8) -> bool {
    let pieces = if side == WHITE {
        board.bb_w
    } else {
        board.bb_b
    };
    let bb_square = 1u64 << square;
    let occupancy = board.occupancy();
    let bb_king = mg.get_non_slider_attacks(KING, square);
    let bb_rook = mg.get_slider_attacks(ROOK, square, occupancy);
    let bb_bishop = mg.get_slider_attacks(BISHOP, square, occupancy);
    let bb_knight = mg.get_non_slider_attacks(KNIGHT, square);
    let bb_pawns = if side == WHITE {
        (bb_square & !board.bb_files[FILE_A]) >> 9 | (bb_square & !board.bb_files[FILE_H]) >> 7
    } else {
        (bb_square & !board.bb_files[FILE_A]) << 7 | (bb_square & !board.bb_files[FILE_H]) << 9
    };

    (bb_king & pieces[KING] > 0)
        || (bb_rook & pieces[ROOK] > 0)
        || (bb_rook & pieces[QUEEN] > 0)
        || (bb_bishop & pieces[BISHOP] > 0)
        || (bb_bishop & pieces[QUEEN] > 0)
        || (bb_knight & pieces[KNIGHT] > 0)
        || (bb_pawns & pieces[PAWN] > 0)
}
