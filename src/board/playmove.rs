use super::representation::Board;
use crate::defs::{
    Piece, Side, Square, A1, A8, BLACK, C1, C8, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, D1, D8,
    F1, F8, G1, G8, H1, H8, KING, NR_OF_SQUARES, PAWN, PNONE, ROOK, WHITE,
};
use crate::movegen::{info, movedefs::Move};
use crate::utils::bits;

// Full castling permissions are 1111, or value 15.
// CP_ALL = All castling permissions for both sides.
// N_WKQ = Not White Kingside/Queenside, and so on.
const CP_ALL: u8 = CASTLE_WK | CASTLE_WQ | CASTLE_BK | CASTLE_BQ;
const N_WKQ: u8 = CP_ALL & !(CASTLE_WK | CASTLE_WQ);
const N_WQ: u8 = CP_ALL & !CASTLE_WQ;
const N_WK: u8 = CP_ALL & !CASTLE_WK;
const N_BKQ: u8 = CP_ALL & !(CASTLE_BK | CASTLE_BQ);
const N_BQ: u8 = CP_ALL & !CASTLE_BQ;
const N_BK: u8 = CP_ALL & !CASTLE_BK;

#[rustfmt::skip]
// First element in this array is square A1.
// The N_* constants mark which castling rights are lost
// if the king or rook moves from that starting square.
const CASTLING_PERMS: [u8; NR_OF_SQUARES as usize] = [
    N_WQ,  15,  15,  15,  N_WKQ,  15,  15,  N_WK,
    15,    15,  15,  15,  15,     15,  15,  15, 
    15,    15,  15,  15,  15,     15,  15,  15, 
    15,    15,  15,  15,  15,     15,  15,  15, 
    15,    15,  15,  15,  15,     15,  15,  15,
    15,    15,  15,  15,  15,     15,  15,  15, 
    15,    15,  15,  15,  15,     15,  15,  15, 
    N_BQ,  15,  15,  15,  N_BKQ,  15,  15,  N_BK,
];

// TODO: Update comments
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub fn make(board: &mut Board, m: Move) -> bool {
    // Create the unmake info and store it.
    let mut current_game_state = board.game_state;
    current_game_state.this_move = m;
    board.history.push(current_game_state);

    // Set "us" and "opponent"
    let us = board.game_state.active_color as usize;
    let opponent = (us ^ 1) as usize;

    // Dissect the move so we don't need "m.function()" and type casts everywhere.
    let piece = m.piece() as usize;
    let from = m.from();
    let to = m.to();
    let captured = m.captured() as usize;
    let promoted = m.promoted() as usize;
    let castling = m.castling();
    let double_step = m.double_step();
    let en_passant = m.en_passant();
    let is_promotion = promoted != PNONE;

    // If piece was captured with this move then remove it.
    if captured != PNONE {
        board.remove_piece(opponent, captured, to);
        // Change castling permissions on rook capture.
        if captured == ROOK && (board.game_state.castling > 0) {
            board.zobrist_castling();
            board.game_state.castling &= CASTLING_PERMS[to as usize];
            board.zobrist_castling();
        }
    }

    // Make the move, taking promotion into account.
    board.remove_piece(us, piece, from);
    board.put_piece(us, if !is_promotion { piece } else { promoted }, to);

    // Remove castling permissions if king/rook leaves from starting square.
    // (This will also adjust permissions when castling, because it's the king which moves.)
    if (CASTLING_PERMS[from as usize] != CP_ALL) && (board.game_state.castling > 0) {
        board.zobrist_castling();
        board.game_state.castling &= CASTLING_PERMS[from as usize];
        board.zobrist_castling();
    }

    // If the king is castling, then also move the rook.
    if castling {
        match to {
            G1 => board.move_piece(us, ROOK, H1, F1),
            C1 => board.move_piece(us, ROOK, A1, D1),
            G8 => board.move_piece(us, ROOK, H8, F8),
            C8 => board.move_piece(us, ROOK, A8, D8),
            _ => panic!("Error moving rook during castling."),
        }
    }

    // Every move unsets the up-square (if not unset already).
    if board.game_state.en_passant != None {
        board.clear_ep_square();
    }

    // After an en-passant maneuver, the opponent's pawn has yet to be removed.
    if en_passant {
        board.remove_piece(opponent, PAWN, to ^ 8);
    }

    // A double-step is the only move that sets the ep-square.
    if double_step {
        board.set_ep_square(to ^ 8);
    }

    // Swap the side to move.
    board.swap_side();

    // Update the move counter
    if (piece == PAWN) || (captured != PNONE) {
        board.game_state.halfmove_clock = 0;
    } else {
        board.game_state.halfmove_clock += 1;
    }

    // Increase full move number if black has moved
    if us == BLACK {
        board.game_state.fullmove_number += 1;
    }

    /*** Validating move: see if "us" is in check. If so, undo everything. ***/
    let king_square = board.get_pieces(KING, us).trailing_zeros() as u8;
    if info::square_attacked(board, opponent, king_square) {
        unmake(board);
        return false;
    }

    true
}

/*** ================================================================================ ***/

// TODO: Update comments
#[cfg_attr(debug_assertions, inline(never))]
#[cfg_attr(not(debug_assertions), inline(always))]
pub fn unmake(board: &mut Board) {
    let stored = board.history.pop();

    // Set "us" and "opponent"
    let us = stored.active_color as usize;
    let opponent = (us ^ 1) as usize;

    // Dissect the move to undo
    let m = stored.this_move;
    let piece = m.piece() as usize;
    let from = m.from();
    let to = m.to();
    let captured = m.captured() as usize;
    let promoted = m.promoted() as usize;
    let castling = m.castling();
    let en_passant = m.en_passant();

    // Moving backwards...
    if promoted == PNONE {
        reverse_move(board, us, piece, to, from);
    } else {
        remove_piece(board, us, promoted, to);
        put_piece(board, us, PAWN, from);
    }

    // The king's move was already undone as a normal move.
    // Now undo the correct castling rook move.
    if castling {
        match to {
            G1 => reverse_move(board, us, ROOK, F1, H1),
            C1 => reverse_move(board, us, ROOK, D1, A1),
            G8 => reverse_move(board, us, ROOK, F8, H8),
            C8 => reverse_move(board, us, ROOK, D8, A8),
            _ => panic!("Error: Reversing castling rook move."),
        };
    }

    // If a piece was captured, put it back onto the to-square
    if captured != PNONE {
        put_piece(board, opponent, captured, to);
    }

    // If this was an e-passant move, put the opponent's pawn back
    if en_passant {
        let pawn_square = if us == WHITE { to - 8 } else { to + 8 };
        put_piece(board, opponent, PAWN, pawn_square);
    }

    // restore the previous board state.
    board.game_state = stored;
}

// ===== Helper functions to reverse piece moves without doing zobrist updates. =====

// Removes a piece from the board.
fn remove_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    bits::clear_bit(&mut board.bb_side[side][piece], square);
    bits::clear_bit(&mut board.bb_pieces[side], square);
    board.piece_list[square as usize] = PNONE;
}

// Puts a piece onto the board.
fn put_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    bits::set_bit(&mut board.bb_side[side][piece], square);
    bits::set_bit(&mut board.bb_pieces[side], square);
    board.piece_list[square as usize] = piece;
}

// Moves a piece from one square to the other.
fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: u8, put: u8) {
    remove_piece(board, side, piece, remove);
    put_piece(board, side, piece, put);
}
