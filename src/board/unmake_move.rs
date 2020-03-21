use super::representation::Board;
use crate::defs::{
    Bitboard, A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8, PAWN, PNONE, ROOK, WHITE,
};
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
        let castling = last_move.castling();
        let en_passant = last_move.en_passant();

        let promotion_move = promoted != PNONE;

        // Moving backwards...
        if !promotion_move {
            // remove the piece from the to-square
            clear_bit(&mut bb_mine[piece], to);
            clear_bit(&mut board.bb_pieces[us], to);

            // Put the piece onto the from-square
            set_bit(&mut bb_mine[piece], from);
            set_bit(&mut board.bb_pieces[us], from);
        } else {
            // When this was a promotion, the piece actually changes into a pawn again.
            // Remove the promoted piece from the to-square
            clear_bit(&mut bb_mine[promoted], to);
            clear_bit(&mut board.bb_pieces[us], to);

            // Put a pawn onto the from-square
            set_bit(&mut bb_mine[PAWN], from);
            set_bit(&mut board.bb_pieces[us], from);
        }

        // The king's move was already undone.
        // Also Undo the rook move.
        if castling {
            // White short castle. Return the rook from F1 to H1.
            if to == G1 {
                clear_bit(&mut bb_mine[ROOK], F1);
                clear_bit(&mut board.bb_pieces[us], F1);
                set_bit(&mut bb_mine[ROOK], H1);
                set_bit(&mut board.bb_pieces[us], H1);
            }

            // White long castle. Return the rook from D1 to A1.
            if to == C1 {
                clear_bit(&mut bb_mine[ROOK], D1);
                clear_bit(&mut board.bb_pieces[us], D1);
                set_bit(&mut bb_mine[ROOK], A1);
                set_bit(&mut board.bb_pieces[us], A1);
            }

            // Black short castle.  Return the rook from F8 to H8.
            if to == G8 {
                clear_bit(&mut bb_mine[ROOK], F8);
                clear_bit(&mut board.bb_pieces[us], F8);
                set_bit(&mut bb_mine[ROOK], H8);
                set_bit(&mut board.bb_pieces[us], H8);
            }

            // Black long castle.  Return the rook from D8 to A8.
            if to == C8 {
                clear_bit(&mut bb_mine[ROOK], D8);
                clear_bit(&mut board.bb_pieces[us], D8);
                set_bit(&mut bb_mine[ROOK], A8);
                set_bit(&mut board.bb_pieces[us], A8);
            }
        }

        // If a piece was captured, put it back onto the to-square
        if captured != PNONE {
            set_bit(&mut bb_opponent[captured], to);
            set_bit(&mut board.bb_pieces[opponent], to);
        }

        // If this was an e-passant move, put the opponent's pawn back
        if en_passant {
            let pawn_square = if us == WHITE { to - 8 } else { to + 8 };
            set_bit(&mut bb_opponent[PAWN], pawn_square);
            set_bit(&mut board.bb_pieces[opponent], pawn_square);
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
