use super::representation::Board;
use crate::defs::{
    Piece, Side, A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8, PAWN, PNONE, ROOK, WHITE,
};
use crate::utils::bits;

// TODO: Update comments

pub fn unmake_move(board: &mut Board) {
    let unamke_info = board.unmake_list.pop();
    if let Some(stored) = unamke_info {
        // Set "us" and "opponent"
        let us = stored.active_color as usize;
        let opponent = (us ^ 1) as usize;

        // Dissect the move to undo
        let last_move = stored.this_move;
        let piece = last_move.piece() as usize;
        let from = last_move.from();
        let to = last_move.to();
        let captured = last_move.captured() as usize;
        let promoted = last_move.promoted() as usize;
        let castling = last_move.castling();
        let en_passant = last_move.en_passant();

        let promotion_move = promoted != PNONE;

        // Moving backwards...
        if !promotion_move {
            reverse_move(board, us, piece, to, from);
        } else {
            // When this was a promotion, the piece actually changes into a pawn again.
            // Remove the promoted piece from the to-square
            bits::clear_bit(&mut board.bb_side[us][promoted], to);
            bits::clear_bit(&mut board.bb_pieces[us], to);
            board.piece_list[to as usize] = PNONE;

            // Put a pawn onto the from-square
            bits::set_bit(&mut board.bb_side[us][PAWN], from);
            bits::set_bit(&mut board.bb_pieces[us], from);
            board.piece_list[from as usize] = PAWN;
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
            bits::set_bit(&mut board.bb_side[opponent][captured], to);
            bits::set_bit(&mut board.bb_pieces[opponent], to);
            board.piece_list[to as usize] = captured;
        }

        // If this was an e-passant move, put the opponent's pawn back
        if en_passant {
            let pawn_square = if us == WHITE { to - 8 } else { to + 8 };
            bits::set_bit(&mut board.bb_side[opponent][PAWN], pawn_square);
            bits::set_bit(&mut board.bb_pieces[opponent], pawn_square);
            board.piece_list[pawn_square as usize] = PAWN;
        }

        // restore the previous board state.
        board.active_color = stored.active_color;
        board.castling = stored.castling;
        board.en_passant = stored.en_passant;
        board.halfmove_clock = stored.halfmove_clock;
        board.fullmove_number = stored.fullmove_number;
        board.zobrist_key = stored.zobrist_key;
    }
}

fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: u8, put: u8) {
    bits::clear_bit(&mut board.bb_side[side][piece], remove);
    bits::clear_bit(&mut board.bb_pieces[side], remove);
    board.piece_list[remove as usize] = PNONE;

    board.piece_list[put as usize] = piece;
    bits::set_bit(&mut board.bb_side[side][piece], put);
    bits::set_bit(&mut board.bb_pieces[side], put);
}
