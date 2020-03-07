/**
 * masks.rs is the module that creates rook and bishop masks,
 * which will be used to generate magic bitboards for sliding pieces.
 * Up front explanation of why edges are excluded from the mask:
 * Later, when generating attack bitboards for each piece on each square,
 * there will be pieces that block the rays of the sliding pieces. If there is a blocker on
 * a square, the slider cannot see beyond it, and anything beyond it (including the edge of the
 * board) becomes irrelevant. Therefore, edges do not need to be in the masks.
 * They can not be seen by the slider (if a blocker is in the way), or they can always be seen
 * (if there is no blocker). The generator for the attack boards takes this into account.
*/
use crate::defines::{Bitboard, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utils::{
    create_bb_files, create_bb_ranks, create_bb_ray, square_on_file_rank, Direction, Location,
};

/**
 * Explanation of create_rook mask, step by step.
 *
 * Get the location of square the rook is on, as a (file, rank) tuple.
 * Create the bitboards for files, ranks, and the rook's square.
 * Get the bitboards of the file and rank the rook is on.
 * Create a bitboard for the edges of the board, but do NOT include an
 * edge if the rook is actually on it. (Otherwise all bits would be unset.)
 * Create the rook's mask by combining its file and rank bitboards.
 * For the final result: exclude the edge squares and rook's square from the mask.
 */
pub fn create_rook_mask(square: u8) -> Bitboard {
    let location = square_on_file_rank(square);
    let bb_files = create_bb_files();
    let bb_ranks = create_bb_ranks();
    let bb_rook_square = 1u64 << square;
    let bb_edges = edges_without_piece(location);
    let bb_mask = bb_files[location.0 as usize] | bb_ranks[location.1 as usize];

    bb_mask & !bb_edges & !bb_rook_square
}

/**
 * create_bishop_mask() works a bit differently compared to create_rook_mask(), but in the end
 * it does the same thing: create a mask for a sliding piece.
 * First, a bitboard containing all the edges (if the piece is not on the edge).
 * Starting at the given square, the function generates four rays, one for eeach
 * diagonal direction, on an empty board.
 * As a final result, the four rays are combined, to generate all bishop moves from that square,
 * on an empty board. Then the edges are clipped off, as they are not needed in the mask.
*/
pub fn create_bishop_mask(square: u8) -> Bitboard {
    let location = square_on_file_rank(square);
    let bb_edges = edges_without_piece(location);
    let bb_up_left = create_bb_ray(0, square, Direction::UpLeft);
    let bb_up_right = create_bb_ray(0, square, Direction::UpRight);
    let bb_down_right = create_bb_ray(0, square, Direction::DownRight);
    let bb_down_left = create_bb_ray(0, square, Direction::DownLeft);

    (bb_up_left | bb_up_right | bb_down_right | bb_down_left) & !bb_edges
}

/**
 * This function creates a bitboard holding all the edges of the board, as needed to clip
 * the board edges off the rook and bishop masks. To prevent clipping the entire ray if the
 * piece itself is on an edge, the edge(s) containing the piece are excluded.
 */
fn edges_without_piece(location: Location) -> Bitboard {
    let bb_files = create_bb_files();
    let bb_ranks = create_bb_ranks();
    let bb_piece_file = bb_files[location.0 as usize];
    let bb_piece_rank = bb_ranks[location.1 as usize];

    (bb_files[FILE_A] & !bb_piece_file)
        | (bb_files[FILE_H] & !bb_piece_file)
        | (bb_ranks[RANK_1] & !bb_piece_rank)
        | (bb_ranks[RANK_8] & !bb_piece_rank)
}
