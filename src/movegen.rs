use crate::board::Board;
use crate::defines::*;
use crate::magics::Magics;
use crate::print;

/*
Move format explanation

"data" contains all the move information, starting from LSB:

Field       :   bits     Decimal values
============================================
PIECE       :   3        0-7 (use only 0-5)
FROM SQUARE :   6        0-63
TO SQUARE   :   6        0-63
CAPTURE     :   3        0-7

Field:      CAPTURE     TO          FROM        PIECE
            111         111111      111111      111
Shift:      15 bits     9 bits      3 bits      0 bits
& Value:    0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

Get the TO field from "data" by:
    -- Shift 9 bits Right
    -- AND (&) with 0x3F

Obviously, storing information in "data" is the other way around.PIECE_NAME
Storing the "To" square: Shift LEFT 9 bits, then XOR with "data".
*/

pub const MAX_LEGAL_MOVES: u8 = 255;
pub type MoveList = Vec<Move>;

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

    pub fn captured(&self) -> u8 {
        ((self.data >> 15) & 0x7) as u8
    }
}

pub fn generate(board: &Board, side: Side, magics: &Magics, list: &mut MoveList) {
    non_slider(KING, board, side, magics, list);
    non_slider(KNIGHT, board, side, magics, list);
    pawn(board, side, magics, list);
}

fn non_slider(piece: Piece, board: &Board, side: Side, magics: &Magics, list: &mut MoveList) {
    debug_assert!(piece == KING || piece == KNIGHT, "Not a non-slider piece.");
    let us = board.bb_pieces[side];
    let mut pieces = board.get_pieces(piece, side);
    while pieces > 0 {
        let from = next(&mut pieces) as u8;
        let target = magics.get_non_slider_moves(piece, from);
        let moves = target & !us;
        add_move(board, piece, side, from as u64, moves, list);
    }
}

fn pawn(board: &Board, side: Side, magics: &Magics, list: &mut MoveList) {
    debug_assert!(side == 0 || side == 1, "Incorrect side.");
    let rank_4 = board.bb_ranks[RANK_4 as usize];
    let rank_5 = board.bb_ranks[RANK_5 as usize];
    let mut pawns = board.get_pieces(PAWN, side);
    let target: (u64, u64, i8) = if side == WHITE {
        let one = (pawns << 8) & !board.occupancy();
        let two = (one << 8) & !board.occupancy() & rank_4;
        (one, two, 8)
    } else {
        let one = (pawns >> 8) & !board.occupancy();
        let two = (one >> 8) & !board.occupancy() & rank_5;
        (one, two, -8)
    };
    while pawns > 0 {
        let from = next(&mut pawns) as u8;
        let moves_1 = target.0 & (1u64 << (from as i8 + target.2) as u64);
        let moves_2 = target.1 & (1u64 << (from as i8 + target.2 + target.2) as u64);
        add_move(board, PAWN, side, from as u64, moves_1, list);
        add_move(board, PAWN, side, from as u64, moves_2, list);
    }
}

fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1u64 << location;
    location as u8
}

fn is_capture(board: &Board, side: Side, to_square: u8) -> Piece {
    let target_square = 1u64 << (to_square as u64);
    let opponent_pieces = board.bb_pieces[side ^ 1];
    if target_square & opponent_pieces > 0 {
        return board.which_piece(to_square);
    };
    PNONE
}

fn add_move(board: &Board, piece: Piece, side: Side, from: u64, to: Bitboard, list: &mut MoveList) {
    let mut bitboard_to = to;
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        let capture = is_capture(board, side, to_square);
        list.push(Move {
            data: (piece as u64)
                ^ ((from as u64) << 3)
                ^ ((to_square as u64) << 9)
                ^ ((capture as u64) << 15),
            score: 0,
        });
    }
}
