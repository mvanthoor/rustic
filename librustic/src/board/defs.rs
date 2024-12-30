use crate::defs::{Bitboard, NrOf, Piece, Square};
use std::ops::RangeInclusive;

pub use crate::board::fen::FenError;
pub use crate::board::zobrist::ZobristKey;

#[cfg(feature = "extra")]
pub use crate::board::fen::fen_setup_fast;

#[rustfmt::skip]
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
pub const PIECE_CHAR_SMALL: [&str; NrOf::PIECE_TYPES + 1] = ["k", "q", "r", "b", "n", "", ""];

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

pub const BB_FILES: [Bitboard; NrOf::FILES] = init_bb_files();
pub const BB_RANKS: [Bitboard; NrOf::RANKS] = init_bb_ranks();
pub const BB_SQUARES: [Bitboard; NrOf::SQUARES] = init_bb_squares();

#[derive(Copy, Clone)]
pub struct Location {
    pub file: u8,
    pub rank: u8,
}

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

// ===== const fn functions ===== //

// Initializes a constant array of bitboards, with each bitboard
// representing a file.
const fn init_bb_files() -> [Bitboard; NrOf::FILES] {
    const BB_FILE_A: Bitboard = 0x0101_0101_0101_0101;
    let mut bb_files = [0; NrOf::FILES];
    let mut i = 0;

    while i < (NrOf::FILES) {
        bb_files[i] = BB_FILE_A << i;
        i += 1;
    }

    bb_files
}

// Initializes a constant array of bitboards, with each bitboard
// representing a rank.
const fn init_bb_ranks() -> [Bitboard; NrOf::RANKS] {
    const BB_RANK_1: Bitboard = 0xFF;
    let mut bb_ranks = [0; NrOf::RANKS];
    let mut i = 0;

    while i < NrOf::RANKS {
        bb_ranks[i] = BB_RANK_1 << (i * 8);
        i += 1;
    }

    bb_ranks
}

// Initializes a constant array of bitboards, with each bitboard
// representing a single square.
const fn init_bb_squares() -> [Bitboard; NrOf::SQUARES] {
    let mut bb_squares = [0; NrOf::SQUARES];
    let mut i = 0;

    while i < NrOf::SQUARES {
        bb_squares[i] = 1u64 << i;
        i += 1;
    }

    bb_squares
}
