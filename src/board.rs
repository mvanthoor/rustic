pub mod fen;
pub mod playmove;
pub mod representation;
pub mod zobrist;

use crate::defs::{Bitboard, Piece, Square, NR_OF_FILES, NR_OF_RANKS, NR_OF_SQUARES};
use std::ops::RangeInclusive;

// Piece location: (file, rank)
pub type Location = (u8, u8);

pub const ALL_RANKS: RangeInclusive<u8> = (Ranks::R1 as u8)..=(Ranks::R8 as u8);
pub const ALL_FILES: RangeInclusive<u8> = (Files::A as u8)..=(Files::H as u8);
pub const ALL_SQUARES: RangeInclusive<Square> = 0..=63;

#[rustfmt::skip]
#[allow(dead_code)]
pub const SQUARE_NAME: [&str; NR_OF_SQUARES] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];
pub const PIECE_NAME: [&str; 7] = ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn", "-"];

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

pub struct Pieces;
impl Pieces {
    pub const KING: Piece = 0;
    pub const QUEEN: Piece = 1;
    pub const ROOK: Piece = 2;
    pub const BISHOP: Piece = 3;
    pub const KNIGHT: Piece = 4;
    pub const PAWN: Piece = 5;
    pub const NONE: Piece = 6;
}

pub struct Files;
impl Files {
    pub const A: usize = 0;
    pub const B: usize = 1;
    pub const G: usize = 6;
    pub const H: usize = 7;
}

pub struct Ranks;
impl Ranks {
    pub const R1: usize = 0;
    pub const R2: usize = 1;
    pub const R4: usize = 3;
    pub const R5: usize = 4;
    pub const R7: usize = 6;
    pub const R8: usize = 7;
}

pub struct Squares;
impl Squares {
    // White side squares that are important for castling
    pub const A1: Square = 0;
    pub const B1: Square = 1;
    pub const C1: Square = 2;
    pub const D1: Square = 3;
    pub const E1: Square = 4;
    pub const F1: Square = 5;
    pub const G1: Square = 6;
    pub const H1: Square = 7;

    // Black side squares that are important for castling
    pub const A8: Square = 56;
    pub const B8: Square = 57;
    pub const C8: Square = 58;
    pub const D8: Square = 59;
    pub const E8: Square = 60;
    pub const F8: Square = 61;
    pub const G8: Square = 62;
    pub const H8: Square = 63;

    // White EP-squares start/end
    pub const A3: Square = 16;
    pub const H3: Square = 23;

    // Black EP-squares start/end
    pub const A6: Square = 40;
    pub const H6: Square = 47;
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

#[rustfmt::skip]
#[allow(clippy::unreadable_literal)]
// Bitboards for each square, with one bit set for each particular square.
pub const BB_SQUARES: [Bitboard; NR_OF_SQUARES] = [
    1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768,
    65536, 131072, 262144, 524288, 1048576, 2097152, 4194304, 8388608, 16777216,
    33554432, 67108864, 134217728, 268435456, 536870912, 1073741824, 2147483648,
    4294967296, 8589934592, 17179869184, 34359738368, 68719476736, 137438953472,
    274877906944, 549755813888, 1099511627776, 2199023255552, 4398046511104,
    8796093022208, 17592186044416, 35184372088832, 70368744177664, 140737488355328,
    281474976710656, 562949953421312, 1125899906842624, 2251799813685248,
    4503599627370496, 9007199254740992, 18014398509481984, 36028797018963968,
    72057594037927936, 144115188075855872, 288230376151711744, 576460752303423488,
    1152921504606846976, 2305843009213693952, 4611686018427387904, 9223372036854775808,
];

// Compute on which file and rank a given square is.
pub fn square_on_file_rank(square: Square) -> Location {
    let file = (square % 8) as u8; // square mod 8
    let rank = (square / 8) as u8; // square div 8
    (file, rank)
}

// Compute if a given square is or isn't on the given rank.
pub fn square_on_rank(square: Square, rank: Square) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}
