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
PROMOTION   :   3        0-7

Field:      PROMOTION   CAPTURE     TO          FROM        PIECE
            111         111         111111      111111      111
Shift:      18 bits     15 bits     9 bits      3 bits      0 bits
& Value:    0x7 (7)     0x7 (7)     0x3F (63)   0x3F (63)   0x7 (7)

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

    pub fn promoted(&self) -> u8 {
        ((self.data >> 18) & 0x7) as u8
    }
}

pub fn generate(board: &Board, side: Side, magics: &Magics, list: &mut MoveList) {
    debug_assert!(side == 0 || side == 1, "Incorrect side.");
    non_slider(KING, board, side, magics, list);
    non_slider(KNIGHT, board, side, magics, list);
    pawn_push(board, side, list);
}

fn non_slider(piece: Piece, board: &Board, side: Side, magics: &Magics, list: &mut MoveList) {
    let us = board.bb_pieces[side];
    let mut pieces = board.get_pieces(piece, side);
    while pieces > 0 {
        let from = next(&mut pieces) as u8;
        let target = magics.get_non_slider_moves(piece, from);
        let moves = target & !us;
        add_move(board, piece, side, from as u64, moves, list);
    }
}

fn pawn_push(board: &Board, side: Side, list: &mut MoveList) {
    let direction = if side == WHITE { 8 } else { -8 };
    let rank = if side == WHITE { BB_RANK_4 } else { BB_RANK_5 };
    let mut pawns = board.get_pieces(PAWN, side);
    while pawns > 0 {
        let from = next(&mut pawns) as u8;
        let target = 1u64 << (from as i8 + direction);
        let one_step = target & !board.occupancy();
        let two_step = one_step.rotate_left((64 + direction) as u32) & !board.occupancy() & rank;
        add_move(board, PAWN, side, from as u64, one_step ^ two_step, list);
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
    let promotion_rank = if side == WHITE { RANK_8 } else { RANK_1 };
    let promotion_pieces: [usize; 4] = [QUEEN, ROOK, BISHOP, KNIGHT];
    while bitboard_to > 0 {
        let to_square = next(&mut bitboard_to);
        let capture = is_capture(board, side, to_square);
        let promotion = piece == PAWN && board.square_on_rank(to_square, promotion_rank);
        if promotion {
            // Add promotions
            for p in promotion_pieces.iter() {
                list.push(Move {
                    data: (piece as u64)
                        ^ ((from as u64) << 3)
                        ^ ((to_square as u64) << 9)
                        ^ ((capture as u64) << 15)
                        ^ ((*p as u64) << 18),
                    score: 0,
                });
            }
        } else {
            // Add normal move
            list.push(Move {
                data: (piece as u64)
                    ^ ((from as u64) << 3)
                    ^ ((to_square as u64) << 9)
                    ^ ((capture as u64) << 15)
                    ^ ((PNONE as u64) << 18),
                score: 0,
            });
        }
    }
}
