pub mod fen;
pub mod playmove;
pub mod representation;
pub mod zobrist;

use crate::defs::{Bitboard, Square, NR_OF_FILES, NR_OF_RANKS, NR_OF_SQUARES};
use std::ops::RangeInclusive;

// Piece location: (file, rank)
pub type Location = (u8, u8);

pub const ALL_RANKS: RangeInclusive<u8> = (Rank::R1 as u8)..=(Rank::R8 as u8);
pub const ALL_FILES: RangeInclusive<u8> = (File::A as u8)..=(File::H as u8);
pub const ALL_SQUARES: RangeInclusive<u8> = 0..=63;

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

pub struct File;
impl File {
    pub const A: usize = 0;
    pub const B: usize = 1;
    pub const G: usize = 6;
    pub const H: usize = 7;
}

pub struct Rank;
impl Rank {
    pub const R1: usize = 0;
    pub const R2: usize = 1;
    pub const R4: usize = 3;
    pub const R5: usize = 4;
    pub const R7: usize = 6;
    pub const R8: usize = 7;
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
pub const BB_SQUARES: [Bitboard; NR_OF_SQUARES as usize] = [
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
