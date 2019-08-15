use crate::board::Board;

pub const ENGINE: &str = "Stainless";
pub const VERSION: &str = "0.1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub type Bitboard = u64;
pub type AsciiBoard = [char; 64];
pub type Mask = [Bitboard; 64];
pub type FunctionPointerFenPartHandler = fn(part: &str, board: &mut Board);

pub enum Color {
    WHITE,
    BLACK,
}

pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const ASCII_EMPTY_SQUARE: char = '.';
pub const NR_OF_FILES: u8 = 8;
pub const RANK_1: u8 = 0;
pub const RANK_8: u8 = 7;
pub const FILE_A: u8 = 0;
pub const FILE_H: u8 = 7;

pub const BB_K: usize = 0;
pub const BB_Q: usize = 1;
pub const BB_R: usize = 2;
pub const BB_B: usize = 3;
pub const BB_N: usize = 4;
pub const BB_P: usize = 5;
pub const MASK_K: usize = 0;
pub const MASK_N: usize = 1;
pub const MASK_R: usize = 2;

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

pub const LIST_OF_PIECES: &str = "kqrbnpKQRBNP";
pub const LETTERS: &str = "abcdefgh";
pub const EN_PASSANT_RANKS: &str = "36";
pub const WHITE_OR_BLACK: &str = "wb";
pub const CASTLE_RIGHTS: &str = "KQkq";
pub const SPLITTER: char = '/';
pub const DASH: char = '-';
pub const SPACE: char = ' ';
pub const MAX_FULL_MOVES: u16 = 9999;
