use crate::defs::{Bitboard, NrOf, Piece, Square};
use std::ops::RangeInclusive;

// Exports
pub use super::fen::ERR_FEN_PARTS;
pub use super::zobrist::ZobristRandoms;

#[rustfmt::skip]
#[allow(dead_code)]
pub const SQUARE_NAME: [&str; NrOf::SQUARES] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];
pub const PIECE_NAME: [&str; NrOf::PIECE_TYPES + 1] =
    ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn", "-"];

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

pub struct RangeOf;
impl RangeOf {
    pub const RANKS: RangeInclusive<u8> = (Ranks::R1 as u8)..=(Ranks::R8 as u8);
    pub const FILES: RangeInclusive<u8> = (Files::A as u8)..=(Files::H as u8);
    pub const SQUARES: RangeInclusive<Square> = 0..=63;
}

// Bitboards for the first file and first rank.
pub const BB_FILE_A: Bitboard = 0x0101_0101_0101_0101;
pub const BB_RANK_1: Bitboard = 0xFF;

// Contains bitboards for each file.
pub const BB_FILES: [Bitboard; NrOf::FILES as usize] = [
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
pub const BB_RANKS: [Bitboard; NrOf::RANKS as usize] = [
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
pub const BB_SQUARES: [Bitboard; NrOf::SQUARES] = [
    1u64, 1u64 << 1, 1u64 << 2, 1u64 << 3, 1u64 << 4, 1u64 << 5, 1u64 << 6, 1u64 << 7,
    1u64 << 8, 1u64 << 9, 1u64 << 10, 1u64 << 11, 1u64 << 12, 1u64 << 13, 1u64 << 14, 1u64 << 15,
    1u64 << 16, 1u64 << 17, 1u64 << 18, 1u64 << 19, 1u64 << 20, 1u64 << 21, 1u64 << 22, 1u64 << 23,
    1u64 << 24, 1u64 << 25, 1u64 << 26, 1u64 << 27, 1u64 << 28, 1u64 << 29, 1u64 << 30, 1u64 << 31,
    1u64 << 32, 1u64 << 33, 1u64 << 34, 1u64 << 35, 1u64 << 36, 1u64 << 37, 1u64 << 38, 1u64 << 39,
    1u64 << 40, 1u64 << 41, 1u64 << 42, 1u64 << 43, 1u64 << 44, 1u64 << 45, 1u64 << 46, 1u64 << 47,
    1u64 << 48, 1u64 << 49, 1u64 << 50, 1u64 << 51, 1u64 << 52, 1u64 << 53, 1u64 << 54, 1u64 << 55,
    1u64 << 56, 1u64 << 57, 1u64 << 58, 1u64 << 59, 1u64 << 60, 1u64 << 61, 1u64 << 62, 1u64 << 63,
];

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
