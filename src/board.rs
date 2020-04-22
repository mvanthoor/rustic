pub mod fen;
pub mod playmove;
pub mod representation;
pub mod zobrist;

use crate::defs::{Bitboard, Square, NR_OF_FILES, NR_OF_RANKS};

// Piece location: (file, rank)
pub type Location = (u8, u8);

// This enum holds the direction in which a ray of a slider piece can point.
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

// Bitboards for the first file and first rank.
pub const BB_FILE_A: Bitboard = 0x0101_0101_0101_0101;
pub const BB_RANK_1: Bitboard = 0xFF;

// Contains bitboards for each file.
pub const BB_FILES: [Bitboard; NR_OF_FILES as usize] = [
    BB_FILE_A,
    BB_FILE_A << 1,
    BB_FILE_A << 2,
    BB_FILE_A << 3,
    BB_FILE_A << 4,
    BB_FILE_A << 5,
    BB_FILE_A << 6,
    BB_FILE_A << 7,
];

// Contains bitboards for each rank.
pub const BB_RANKS: [Bitboard; NR_OF_RANKS as usize] = [
    BB_RANK_1,
    BB_RANK_1 << 8,
    BB_RANK_1 << 16,
    BB_RANK_1 << 24,
    BB_RANK_1 << 32,
    BB_RANK_1 << 40,
    BB_RANK_1 << 48,
    BB_RANK_1 << 56,
];

// Compute on which file and rank a given square is.
pub fn square_on_file_rank(square: Square) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

// Compute if a given square is or isn't on the given rank.
pub fn square_on_rank(square: Square, rank: Square) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}
