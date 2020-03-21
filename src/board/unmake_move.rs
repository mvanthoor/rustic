use super::representation::Board;
use crate::defs::{Bitboard, PAWN, PNONE, WHITE};
use crate::utils::{clear_bit, set_bit};

pub fn unmake_move(board: &mut Board) {
    let unamke_info = board.unmake_list.pop();
    if let Some(stored) = unamke_info {
        // Set "us" and "opponent"
        let us = stored.active_color as usize;
        let opponent = (us ^ 1) as usize;

        // Set which bitboards are "us" and "opponent" pieces
        let bb_mine: &mut [Bitboard];
        let bb_opponent: &mut [Bitboard];

        if us == WHITE {
            bb_mine = &mut board.bb_w;
            bb_opponent = &mut board.bb_b;
        } else {
            bb_mine = &mut board.bb_b;
            bb_opponent = &mut board.bb_w;
        };

        // Dissect the move to undo
        let last_move = stored.this_move;
        let piece = last_move.piece() as usize;
        let from = last_move.from();
        let to = last_move.to();
        let captured = last_move.captured() as usize;
        let promoted = last_move.promoted() as usize;

        // Moving backwards... normal move.
        if promoted == PNONE {
            // remove the piece from the to-square
            clear_bit(&mut bb_mine[piece], to);
            clear_bit(&mut board.bb_pieces[us], to);

            // Put the piece onto the from-square
            set_bit(&mut bb_mine[piece], from);
            set_bit(&mut board.bb_pieces[us], from);
        }

        // Moving backwards... promotion
        if promoted != PNONE {
            // Remove the promoted piece from the to-square
            clear_bit(&mut bb_mine[promoted], to);
            clear_bit(&mut board.bb_pieces[us], to);

            // Put a pawn back
            set_bit(&mut bb_mine[PAWN], from);
            set_bit(&mut board.bb_pieces[us], from);
        }

        // If a piece was captured, put it back onto the to-square
        if captured != PNONE {
            set_bit(&mut bb_opponent[captured], to);
            set_bit(&mut board.bb_pieces[opponent], to);
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
