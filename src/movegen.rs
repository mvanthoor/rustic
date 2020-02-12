use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;

pub const MAX_LEGAL_MOVES: u8 = 255;

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-5)
FROM SQUARE :   6        0-63
TO SQUARE   :   6        0-63

Field:  TO          FROM        PIECE
        111111      111111      111
ShiftR: 9 bits      3 bits      0 bits
Value:  0x3F (63)   0x3F (63)   0x7 (7)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

pub struct Move {
    data: u64,
    score: u32,
}

impl Move {
    pub fn piece(&self) -> &str {
        let piece = (self.data) & 0x7;
        debug_assert!(piece <= 5, "Invalid piece.");
        let x = PIECE_NAME[piece as usize];
        x
    }

    pub fn from(&self) -> &str {
        let from = (self.data >> 3) & 0x3F;
        debug_assert!(from <= 63, "Invalid square.");
        let x = SQUARE_NAME[from as usize];
        x
    }

    pub fn to(&self) -> &str {
        let to = (self.data >> 9) & 0x3F;
        debug_assert!(to <= 63, "Invalid square.");
        SQUARE_NAME[to as usize]
    }
}

pub type MoveList = Vec<Move>;

enum MoveType {
    Normal,
    Capture,
    Castle,
    EnPassant,
}

pub fn generate(board: &Board, side: Side, magics: &Magics, moves: &mut MoveList) {
    non_slider(KING, board, side, magics, moves);
    non_slider(KNIGHT, board, side, magics, moves);
}

fn non_slider(piece: Piece, board: &Board, side: Side, magics: &Magics, moves: &mut MoveList) {
    debug_assert!(piece == KING || piece == KNIGHT, "Not a non-slider piece!");
    let opponent = side ^ 1;
    let mut bitboard = board.piece(piece, side);
    while bitboard > 0 {
        let from = next(&mut bitboard) as usize;
        let mask: Bitboard = match piece {
            KING => magics.king[from],
            KNIGHT => magics.knight[from],
            _ => 0,
        };
        let normal_to = mask & !board.occupancy();
        let capture_to = mask & board.bb_pieces[opponent];
        add_move(piece, from as u64, normal_to, MoveType::Normal, moves);
        add_move(piece, from as u64, capture_to, MoveType::Capture, moves);
    }
}

fn next(bitboard: &mut Bitboard) -> u64 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1 << location;
    location as u64
}

fn add_move(piece: Piece, from: u64, to: Bitboard, mtype: MoveType, moves: &mut MoveList) {
    let mut bitboard_to = to;
    match mtype {
        MoveType::Normal => (),
        MoveType::Capture => (),
        _ => (),
    }
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        moves.push(Move {
            data: (piece as u64) ^ (from << 3) ^ (to_square << 9),
            score: 0,
        });
    }
}
