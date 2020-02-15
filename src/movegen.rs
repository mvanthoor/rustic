use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-5)
FROM SQUARE :   6        0-63
TO SQUARE   :   6        0-63
MOVETYPE    :   3        0-7 (use only 0-4)

Field:      MOVETYPE    TO          FROM        PIECE
            111         111111      111111      111
ShiftR:     15 bits     9 bits      3 bits      0 bits
& Value:    0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

pub const MAX_LEGAL_MOVES: u8 = 255;
pub type MoveList = Vec<Move>;

type MoveType = u8;

const MT_NORMAL: MoveType = 0;
const MT_CASTLE: MoveType = 1;
const MT_CAPTURE: MoveType = 2;
const MT_ENPASSANT: MoveType = 3;
const MT_PROMOTION: MoveType = 4;

pub struct Move {
    data: u64,
    score: u32,
}

impl Move {
    pub fn piece(&self) -> u8 {
        ((self.data) & 0x7) as u8
    }

    pub fn from(&self) -> u8 {
        ((self.data >> 3) & 0x3F) as u8
    }

    pub fn to(&self) -> u8 {
        ((self.data >> 9) & 0x3F) as u8
    }

    pub fn move_type(&self) -> u8 {
        ((self.data >> 15) & 0x7) as u8
    }
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
        add_move(piece, from as u64, capture_to, MT_CAPTURE, moves);
        add_move(piece, from as u64, normal_to, MT_NORMAL, moves);
    }
}

fn next(bitboard: &mut Bitboard) -> u64 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1 << location;
    location as u64
}

fn add_move(piece: Piece, from: u64, to: Bitboard, mtype: MoveType, moves: &mut MoveList) {
    let mut bitboard_to = to;
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        moves.push(Move {
            data: (piece as u64) ^ (from << 3) ^ (to_square << 9) ^ ((mtype as u64) << 15),
            score: 0,
        });
    }
}
