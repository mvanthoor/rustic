/**
 * defs.rs holds all the definitions that are needed throughout the program.
 * If there are definitions that are needed only inside a single module, those
 * can be found within that module.
*/

pub const ENGINE: &str = "Rustic";
pub const VERSION: &str = "Alpha 1";
pub const AUTHOR: &str = "Marcel Vanthoor";

pub type Bitboard = u64;
pub type Piece = usize;
pub type Side = usize;
pub type Square = u8;

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
pub const PIECE_NAME: [&str; 7] = ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn", "-"];

pub const FEN_START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const NR_OF_SQUARES: u8 = 64;
pub const NR_OF_FILES: u8 = 8;
pub const NR_OF_RANKS: u8 = 8;
pub const NR_OF_PIECES: u8 = 6;
pub const EACH_SIDE: u8 = 2;

// White side squares that are important for castling
pub const A1: u8 = 0;
pub const B1: u8 = 1;
pub const C1: u8 = 2;
pub const D1: u8 = 3;
pub const E1: u8 = 4;
pub const F1: u8 = 5;
pub const G1: u8 = 6;
pub const H1: u8 = 7;

// Black side squares that are important for castling
pub const A8: u8 = 56;
pub const B8: u8 = 57;
pub const C8: u8 = 58;
pub const D8: u8 = 59;
pub const E8: u8 = 60;
pub const F8: u8 = 61;
pub const G8: u8 = 62;
pub const H8: u8 = 63;

pub const KING: Piece = 0;
pub const QUEEN: Piece = 1;
pub const ROOK: Piece = 2;
pub const BISHOP: Piece = 3;
pub const KNIGHT: Piece = 4;
pub const PAWN: Piece = 5;
pub const PNONE: Piece = 6;

pub struct Castling;
impl Castling {
    pub const WK: u8 = 1;
    pub const WQ: u8 = 2;
    pub const BK: u8 = 4;
    pub const BQ: u8 = 8;
    pub const ALL: u8 = 15;
}
pub const NR_OF_CASTLING_PERMISSIONS: u8 = 16; // 0-15

pub const EMPTY: u64 = 0;
pub const MAX_GAME_MOVES: usize = 2048;
