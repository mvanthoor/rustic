// gen.rs is the file generating pseudo-legal moves for the current board position.

use super::{
    info,
    movedefs::{Move, Shift},
    movelist::MoveList,
};
use crate::board::{self, representation::Board, Pieces, Ranks, Squares, BB_RANKS};
use crate::defs::{Bitboard, Castling, Piece, Square, BLACK, WHITE};
use crate::utils::bits;

const PROMOTION_PIECES: [usize; 4] = [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT];

// This function generates all pseudo-legal moves for the current board and side to move.
pub fn all_moves(board: &Board, list: &mut MoveList) {
    piece(Pieces::KING, board, list);
    piece(Pieces::KNIGHT, board, list);
    piece(Pieces::ROOK, board, list);
    piece(Pieces::BISHOP, board, list);
    piece(Pieces::QUEEN, board, list);
    pawns(board, list);
    castling(board, list);
}

/// This function generates pseudo-legal moves for the given piece type.
fn piece(piece: Piece, board: &Board, list: &mut MoveList) {
    let side = board.game_state.active_color as usize;
    let bb_occupancy = board.occupancy();
    let bb_own_pieces = board.bb_pieces[side];
    let mut bb_pieces = board.get_pieces(piece, side);

    // Generate moves for each piece of the type passed into the function.
    while bb_pieces > 0 {
        let from = bits::next(&mut bb_pieces);
        let bb_target = match piece {
            Pieces::QUEEN | Pieces::ROOK | Pieces::BISHOP => {
                board.get_slider_attacks(piece, from, bb_occupancy)
            }
            Pieces::KING | Pieces::KNIGHT => board.get_non_slider_attacks(piece, from),
            _ => 0,
        };

        // A piece can move to where there is no piece of our own.
        let bb_moves = bb_target & !bb_own_pieces;
        add_move(board, piece, from, bb_moves, list);
    }
}

// This function generates all the pawn moves.
fn pawns(board: &Board, list: &mut MoveList) {
    const UP: i8 = 8;
    const DOWN: i8 = -8;

    let side = board.game_state.active_color as usize;
    let bb_opponent_pieces = board.bb_pieces[side ^ 1];
    let bb_empty = !board.occupancy();
    let bb_fourth = if side == WHITE {
        BB_RANKS[Ranks::R4]
    } else {
        BB_RANKS[Ranks::R5]
    };
    let mut bb_pawns = board.get_pieces(Pieces::PAWN, side);
    let direction = if side == WHITE { UP } else { DOWN };

    // As long as there are pawns, generate moves for each of them.
    while bb_pawns > 0 {
        let from = bits::next(&mut bb_pawns);
        let bb_push = 1u64 << (from as i8 + direction);
        let bb_one_step = bb_push & bb_empty;
        let bb_two_step = bb_one_step.rotate_left((64 + direction) as u32) & bb_empty & bb_fourth;
        let bb_targets = board.get_pawn_attacks(side, from);
        let bb_captures = bb_targets & bb_opponent_pieces;
        let bb_ep_capture = if let Some(ep) = board.game_state.en_passant {
            bb_targets & (1u64 << ep)
        } else {
            0
        };

        // Gather all moves for the pawn into one bitboard.
        let bb_moves = bb_one_step | bb_two_step | bb_captures | bb_ep_capture;
        add_move(board, Pieces::PAWN, from, bb_moves, list);
    }
}

// This function generates castling moves (king part only).
fn castling(board: &Board, list: &mut MoveList) {
    let side = board.game_state.active_color as usize;
    let opponent = side ^ 1;
    let castle_perms_white = (board.game_state.castling & (Castling::WK | Castling::WQ)) > 0;
    let castle_perms_black = (board.game_state.castling & (Castling::BK | Castling::BQ)) > 0;
    let bb_occupancy = board.occupancy();
    let mut bb_king = board.get_pieces(Pieces::KING, side);
    let from = bits::next(&mut bb_king);

    if side == WHITE && castle_perms_white {
        // Kingside
        if board.game_state.castling & Castling::WK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << Squares::F1) | (1u64 << Squares::G1);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

            if !is_kingside_blocked
                && !info::square_attacked(board, opponent, Squares::E1)
                && !info::square_attacked(board, opponent, Squares::F1)
            {
                let to = (1u64 << from) << 2;
                add_move(board, Pieces::KING, from, to, list);
            }
        }

        if board.game_state.castling & Castling::WQ > 0 {
            // Queenside
            let bb_queenside_blockers: u64 =
                (1u64 << Squares::B1) | (1u64 << Squares::C1) | (1u64 << Squares::D1);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

            if !is_queenside_blocked
                && !info::square_attacked(board, opponent, Squares::E1)
                && !info::square_attacked(board, opponent, Squares::D1)
            {
                let to = (1u64 << from) >> 2;
                add_move(board, Pieces::KING, from, to, list);
            }
        }
    } else if side == BLACK && castle_perms_black {
        // Kingside
        if board.game_state.castling & Castling::BK > 0 {
            let bb_kingside_blockers: u64 = (1u64 << Squares::F8) | (1u64 << Squares::G8);
            let is_kingside_blocked = (bb_occupancy & bb_kingside_blockers) > 0;

            if !is_kingside_blocked
                && !info::square_attacked(board, opponent, Squares::E8)
                && !info::square_attacked(board, opponent, Squares::F8)
            {
                let to = (1u64 << from) << 2;
                add_move(board, Pieces::KING, from, to, list);
            }
        }

        // Queenside
        if board.game_state.castling & Castling::BQ > 0 {
            let bb_queenside_blockers: u64 =
                (1u64 << Squares::B8) | (1u64 << Squares::C8) | (1u64 << Squares::D8);
            let is_queenside_blocked = (bb_occupancy & bb_queenside_blockers) > 0;

            if !is_queenside_blocked
                && !info::square_attacked(board, opponent, Squares::E8)
                && !info::square_attacked(board, opponent, Squares::D8)
            {
                let to = (1u64 << from) >> 2;
                add_move(board, Pieces::KING, from, to, list);
            }
        }
    }
}

// This function turns the given parameters into actual moves and puts them into the move list.
fn add_move(board: &Board, piece: Piece, from: Square, to: Bitboard, list: &mut MoveList) {
    let mut bb_to = to;
    let side = board.game_state.active_color as usize;
    let promotion_rank = if side == WHITE { Ranks::R8 } else { Ranks::R1 };

    // As long as there are still to-squres in bb_to, this piece has moves to add.
    while bb_to > 0 {
        let to_square = bits::next(&mut bb_to);
        let capture = board.piece_list[to_square];
        let promotion = (piece == Pieces::PAWN) && board::square_on_rank(to_square, promotion_rank);
        let en_passant = if let Some(square) = board.game_state.en_passant {
            (piece == Pieces::PAWN) && (square as usize == to_square)
        } else {
            false
        };
        let double_step = (piece == Pieces::PAWN) && ((to_square as i8 - from as i8).abs() == 16);
        let castling = (piece == Pieces::KING) && ((to_square as i8 - from as i8).abs() == 2);

        // Gather all data for this move into one 64-bit integer.
        let move_data = (piece as u64)
            | ((from as u64) << Shift::FromSq as u64)
            | ((to_square as u64) << Shift::ToSq as u64)
            | ((capture as u64) << Shift::Capture as u64)
            | ((en_passant as u64) << Shift::EnPassant as u64)
            | ((double_step as u64) << Shift::DoubleStep as u64)
            | ((castling as u64) << Shift::Castling as u64);

        if !promotion {
            // No promotion: add this move.
            let m = Move {
                data: move_data | ((Pieces::NONE as u64) << Shift::Promotion as u64),
            };
            list.push(m);
        } else {
            // Promotion. Add one move for each piece to promote to.
            for piece in PROMOTION_PIECES.iter() {
                let m = Move {
                    data: move_data | ((*piece as u64) << Shift::Promotion as u64),
                };
                list.push(m);
            }
        }
    }
}
