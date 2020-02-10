use crate::board::Board;
use std::ops::RangeInclusive;

pub const ENGINE: &str = "Dead End";
pub const VERSION: &str = "0.1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub type Bitboard = u64;
pub type Piece = usize;
pub type Side = usize;
pub type AsciiBoard = [char; NR_OF_SQUARES as usize];
pub type NonSliderDirections = [i8; 8];
pub type SliderDirections = [i8; 4];
pub type NonSliderAttacks = [Bitboard; NR_OF_SQUARES as usize];
pub type SliderMoves = [Bitboard; NR_OF_SQUARES as usize];
pub type FenPartHandlers = fn(part: &str, board: &mut Board);

pub const WHITE: Side = 0;
pub const BLACK: Side = 1;

#[rustfmt::skip]
#[allow(dead_code)]
pub const SQUARE_NAME: [&str; NR_OF_SQUARES as usize] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];

#[allow(dead_code)]
pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const ASCII_EMPTY_SQUARE: char = '.';
pub const RANK_1: u8 = 0;
pub const RANK_8: u8 = 7;
pub const FILE_A: u8 = 0;
pub const FILE_H: u8 = 7;
pub const NR_OF_SQUARES: u8 = 64;
pub const NR_OF_FILES: u8 = 8;
pub const ALL_RANKS: RangeInclusive<u8> = RANK_1..=RANK_8;
pub const ALL_FILES: RangeInclusive<u8> = FILE_A..=FILE_H;
pub const ALL_SQUARES: RangeInclusive<u8> = 0..=63;

pub const BITBOARDS_PER_SIDE: u8 = 6;
pub const BITBOARDS_FOR_PIECES: u8 = 2;
pub const BITBOARDS_PER_FILE: u8 = 8;

pub const KING: Piece = 0;
pub const QUEEN: Piece = 1;
pub const ROOK: Piece = 2;
pub const BISHOP: Piece = 3;
pub const KNIGHT: Piece = 4;
pub const PAWN: Piece = 5;

pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;

pub const CHAR_WK: char = 'K';
pub const CHAR_WQ: char = 'Q';
pub const CHAR_WR: char = 'R';
pub const CHAR_WB: char = 'B';
pub const CHAR_WN: char = 'N';
pub const CHAR_WP: char = 'I';
pub const CHAR_BK: char = 'k';
pub const CHAR_BQ: char = 'q';
pub const CHAR_BR: char = 'r';
pub const CHAR_BB: char = 'b';
pub const CHAR_BN: char = 'n';
pub const CHAR_BP: char = 'i';

pub const MOVE_MAX: u8 = 255;
pub enum MoveType {
    Quiet,
    Capture,
}
