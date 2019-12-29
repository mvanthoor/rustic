use crate::board::Board;
use std::ops::RangeInclusive;

pub const ENGINE: &str = "Dead End";
pub const VERSION: &str = "0.1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub type Bitboard = u64;
pub type AsciiBoard = [char; NR_OF_SQUARES];
pub type NonSliderAttacks = [Bitboard; NR_OF_SQUARES];
pub type FenPartHandlers = fn(part: &str, board: &mut Board);

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const ASCII_EMPTY_SQUARE: char = '.';
pub const RANK_1: u8 = 0;
pub const RANK_8: u8 = 7;
pub const FILE_A: u8 = 0;
pub const FILE_H: u8 = 7;
pub const NR_OF_SQUARES: usize = 64;
pub const NR_OF_FILES: u8 = 8;
pub const ALL_RANKS: RangeInclusive<u8> = RANK_1..=RANK_8;
pub const ALL_FILES: RangeInclusive<u8> = FILE_A..=FILE_H;

pub const BITBOARDS_PER_SIDE: usize = 6;
pub const BITBOARDS_FOR_PIECES: usize = 3;
pub const BITBOARDS_PER_FILE: usize = 8;

pub const KING: usize = 0;
pub const QUEEN: usize = 1;
pub const ROOK: usize = 2;
pub const BISHOP: usize = 3;
pub const KNIGHT: usize = 4;
pub const PAWN: usize = 5;

pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;

pub const CHAR_WK: char = 'K';
pub const CHAR_WQ: char = 'Q';
pub const CHAR_WR: char = 'R';
pub const CHAR_WB: char = 'B';
pub const CHAR_WN: char = 'N';
pub const CHAR_WP: char = 'P';
pub const CHAR_BK: char = 'k';
pub const CHAR_BQ: char = 'q';
pub const CHAR_BR: char = 'r';
pub const CHAR_BB: char = 'b';
pub const CHAR_BN: char = 'n';
pub const CHAR_BP: char = 'a';
