use crate::defines::{Bitboard, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utils::{create_bb_files, create_bb_ranks, square_on_file_rank};

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
    let bb_rook_file = bb_files[location.0 as usize];
    let bb_rook_rank = bb_ranks[location.1 as usize];
    let bb_edges = (bb_files[FILE_A] & !bb_rook_file)
        | (bb_files[FILE_H] & !bb_rook_file)
        | (bb_ranks[RANK_1] & !bb_rook_rank)
        | (bb_ranks[RANK_8] & !bb_rook_rank);
    let bb_mask = bb_rook_file | bb_rook_rank;
    let final_result = bb_mask & !bb_edges & !bb_rook_square;

    final_result
}
