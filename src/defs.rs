use crate::Board;

pub const ENGINE: &str = "Stainless";
pub const VERSION: &str = "0.1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub type Bitboard = u64;
pub type FunctionPointerFenPartHandler = fn(part: &str, board: &mut Board);

pub enum Color {
    WHITE,
    BLACK,
}

pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
pub const ASCII_EMPTY_SQUARE: char = '.';
pub const RANK_8: u8 = 7;
pub const FILE_A: u8 = 0;
pub const BB_K: usize = 0;
pub const BB_Q: usize = 1;
pub const BB_R: usize = 2;
pub const BB_B: usize = 3;
pub const BB_N: usize = 4;
pub const BB_P: usize = 5;
pub const CASTLE_WK: u8 = 1;
pub const CASTLE_WQ: u8 = 2;
pub const CASTLE_BK: u8 = 4;
pub const CASTLE_BQ: u8 = 8;

pub const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
pub const LETTERS: &str = "abcdefgh";
pub const EN_PASSANT_RANKS: &str = "36";
pub const WHITE_OR_BLACK: &str = "wb";
pub const CASTLE_RIGHTS: &str = "KQkq";
pub const SPLITTER: char = '/';
pub const DASH: char = '-';
pub const SPACE: char = ' ';
pub const MAX_FULL_MOVES: u16 = 9999;
