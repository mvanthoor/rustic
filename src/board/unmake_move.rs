use super::representation::Board;
use crate::defs::WHITE;
use crate::utils::{clear_bit, set_bit};

pub fn unmake_move(board: &mut Board) {
    let unamke_info = board.unmake_list.pop();
    if let Some(stored) = unamke_info {
        let side = stored.active_color as usize;
        let last_move = stored.this_move;
        let piece = last_move.piece() as usize;
        let from = last_move.from();
        let to = last_move.to();
        let bb = if side == WHITE {
            &mut board.bb_w[..]
        } else {
            &mut board.bb_b[..]
        };

        // undo the move: "do the move backwards"
        clear_bit(&mut bb[piece], to);
        clear_bit(&mut board.bb_pieces[side], to);
        set_bit(&mut bb[piece], from);
        set_bit(&mut board.bb_pieces[side], from);

        // restore the previous board state.
        board.active_color = stored.active_color;
        board.castling = stored.castling;
        board.en_passant = stored.en_passant;
        board.halfmove_clock = stored.halfmove_clock;
        board.fullmove_number = stored.fullmove_number;
        board.zobrist_key = stored.zobrist_key;
    }
}
