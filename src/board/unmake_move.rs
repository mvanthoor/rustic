use super::representation::Board;
use crate::defs::{
    Piece, Side, A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8, PAWN, PNONE, ROOK, WHITE,
};
//use crate::movegen::movedefs::Move;
use crate::utils::bits;

// TODO: Update comments

pub fn unmake_move(board: &mut Board) {
    let stored = board.unmake_list.pop();

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
            _ => (),
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

// Removes a piece from the board.
fn remove_piece(board: &mut Board, side: Side, piece: Piece, square: u8) {
    bits::clear_bit(&mut board.bb_side[side][piece], square);
    bits::clear_bit(&mut board.bb_pieces[side], square);
    board.piece_list[square as usize] = PNONE;
}

// Puts a piece onto the board.
fn put_piece(board: &mut Board, side: Side, piece: Piece, square: u8) {
    bits::set_bit(&mut board.bb_side[side][piece], square);
    bits::set_bit(&mut board.bb_pieces[side], square);
    board.piece_list[square as usize] = piece;
}

// Moves a piece from one square to the other.
fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: u8, put: u8) {
    remove_piece(board, side, piece, remove);
    put_piece(board, side, piece, put);
}
