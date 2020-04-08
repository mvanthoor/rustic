pub mod fen;
pub mod make_move;
pub mod representation;
pub mod unmake_move;
pub mod zobrist;

use crate::defs::{Bitboard, RANK_1, RANK_8};

// Piece location: (file, rank)
pub type Location = (u8, u8);

/** Direction is an enum holding all the directions the
 * pieces can move in.
 * Up, Right, Down, Left for the rook.
 * UpLeft, UpRight, DownRight, DownLeft for the bishop.
 * The queen is a combination of both.
 */
#[derive(Copy, Clone)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    UpLeft,
    UpRight,
    DownRight,
    DownLeft,
}

/**
 * This function creates an array of 8 bitboards containing file masks.
 * The bitboard's least significant bit is on the right, so A1 is the
 * first bit (or bit 0).
 * There are 8 elements in the array, from 0 up to and including 7.
 * 0x0101_0101_0101_0101 is the hexadecimal representation of setting
 * bit 0 (A1), bit 8 (A2), bit 16 (A3), etc... for the entire A-file.
 * Then, for each file, the bits are shifted.
 * 0 shifts: masks the A file
 * 1 shifts: masks the B file
 * And so on up to and including the H-file.
*/
pub fn create_bb_files() -> [Bitboard; 8] {
    let mut bb_files: [Bitboard; 8] = [0; 8];
    for (i, file) in bb_files.iter_mut().enumerate() {
        *file = 0x0101_0101_0101_0101 << i;
    }
    bb_files
}

/**
 * This function works exacty the same as create_bb_files.
 * It set the entire first byte, or the first rank A1-H1, and then
 * it shifts upward by 8 bits, masking each of the ranks.
 */
pub fn create_bb_ranks() -> [Bitboard; 8] {
    let mut bb_ranks: [Bitboard; 8] = [0; 8];
    for (i, rank) in bb_ranks.iter_mut().enumerate() {
        *rank = 0xFF << (i * 8);
    }
    bb_ranks
}

/**
 * This function returns a (file, rank) tuple, containing
 * the file and rank a given square is on.
 */
pub fn square_on_file_rank(square: u8) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

/**
 * This function returns true if the given square is on the
 * given rank, and false of the square is not on the rank.
 */
pub fn square_on_rank(square: u8, rank: u8) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}
