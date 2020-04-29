use crate::board::{representation::Board, Pieces};
use crate::defs::{Side, Square};

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
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub fn square_attacked(board: &Board, attacker: Side, square: Square) -> bool {
    let pieces = board.bb_side[attacker];
    let occupancy = board.occupancy();
    let bb_king = board.get_non_slider_attacks(Pieces::KING, square);
    let bb_rook = board.get_slider_attacks(Pieces::ROOK, square, occupancy);
    let bb_bishop = board.get_slider_attacks(Pieces::BISHOP, square, occupancy);
    let bb_knight = board.get_non_slider_attacks(Pieces::KNIGHT, square);
    let bb_pawns = board.get_pawn_attacks(attacker ^ 1, square);
    let bb_queen = bb_rook | bb_bishop;

    (bb_king & pieces[Pieces::KING] > 0)
        || (bb_rook & pieces[Pieces::ROOK] > 0)
        || (bb_queen & pieces[Pieces::QUEEN] > 0)
        || (bb_bishop & pieces[Pieces::BISHOP] > 0)
        || (bb_knight & pieces[Pieces::KNIGHT] > 0)
        || (bb_pawns & pieces[Pieces::PAWN] > 0)
}
