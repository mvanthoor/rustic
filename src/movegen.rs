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
    let mut pieces = board.get_pieces(PAWN, side);
    while pieces > 0 {
        let from = next(&mut pieces) as u8;
        pawn_normal(from, board, side, list);
    }
}

fn pawn_normal(from: u8, board: &Board, side: Side, list: &mut MoveList) {
    // The following statement will generate the moves, for example:
    // a2a3, and a2a4. If a3 is blocked, a2a3 will not be generated.
    // However, a2a4 will still be generated if a4 is not blocked.
    // Therefore, for a pawn on the second rank, check if the one stop
    // move is blocked, and if so, remove the second step move if any.

    /*
        let mut normal_to = normal_move & !board.occupancy();

        // Point of view from either White or Black.
        let mut on_second_rank = false;
        let mut one_step: Bitboard = 0;
        let mut two_step: Bitboard = 0;
        match side {
            WHITE => {
                on_second_rank = ((1u64 << from) & board.bb_ranks[RANK_2 as usize]) > 0;
                one_step = 1u64 << (from + 8);
                two_step = 1u64 << (from + 16);
            }
            BLACK => {
                on_second_rank = ((1u64 << from) & board.bb_ranks[RANK_7 as usize]) > 0;
                one_step = 1u64 << (from - 8);
                two_step = 1u64 << (from - 16);
            }
            _ => (),
        }
        let one_step_blocked = one_step & board.occupancy() > 0;
        if on_second_rank && one_step_blocked {
            normal_to &= !two_step;
        }
        add_move(PAWN, from as u64, normal_to, MT_NORMAL, moves);
    */
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
        if let Some(cp) = board.which_piece(to_square) {
            return cp.1;
        }
    }
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
