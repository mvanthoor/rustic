pub mod domove;
pub mod fen;
pub mod representation;
pub mod zobrist;

use crate::defs::{Bitboard, NR_OF_FILES, NR_OF_RANKS};

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
    BB_FILE_A << 0,
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
    BB_RANK_1 << (0 * 8),
    BB_RANK_1 << (1 * 8),
    BB_RANK_1 << (2 * 8),
    BB_RANK_1 << (3 * 8),
    BB_RANK_1 << (4 * 8),
    BB_RANK_1 << (5 * 8),
    BB_RANK_1 << (6 * 8),
    BB_RANK_1 << (7 * 8),
];

// Compute on which file and rank a given square is.
pub fn square_on_file_rank(square: u8) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

// Compute if a given square is or isn't on the given rank.
pub fn square_on_rank(square: u8, rank: u8) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}
