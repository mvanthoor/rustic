use super::representation::Board;
use crate::defs::{A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8, PAWN, PNONE, ROOK, WHITE};
use crate::utils::bits;

/**
 * This function retracts moves made by make_move().
 * Notice that it doesn't use "board.put_piece()" and "board.remove_piece()"
 * and related functions the way make_move does. The reason is that these
 * functions incrementally update the Zobrist key, which is necessary for
 * make_move; doing this update incrementally is much faster than calculating
 * the key from scratch when make_move is done.
 *
 * Unmake_move on the other hand, justretrieves the zobrist key of the
 * previous board from unmake_info, so it  * does not need to keep track
 * of the key. The same is true for the rest of the board state such as
 * castling rights. Therefore, unmake_move just putspieces back where they
 * came from all by itself, without keeping track of the board state,
 * and it copies that state back from the list of saved states at the end.
 * Doing it this way makes unmake_move faster, because all of the incremental
 * state updates can be omitted.
*/

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
            // remove the piece from the to-square
            bits::clear_bit(&mut board.bb_side[us][piece], to);
            bits::clear_bit(&mut board.bb_pieces[us], to);
            board.piece_list[to as usize] = PNONE;

            // Put the piece onto the from-square
            bits::set_bit(&mut board.bb_side[us][piece], from);
            bits::set_bit(&mut board.bb_pieces[us], from);
            board.piece_list[from as usize] = piece;
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
                G1 => {
                    bits::clear_bit(&mut board.bb_side[us][ROOK], F1);
                    bits::clear_bit(&mut board.bb_pieces[us], F1);
                    board.piece_list[F1 as usize] = PNONE;

                    board.piece_list[H1 as usize] = ROOK;
                    bits::set_bit(&mut board.bb_side[us][ROOK], H1);
                    bits::set_bit(&mut board.bb_pieces[us], H1);
                }
                C1 => {
                    bits::clear_bit(&mut board.bb_side[us][ROOK], D1);
                    bits::clear_bit(&mut board.bb_pieces[us], D1);
                    board.piece_list[D1 as usize] = PNONE;

                    board.piece_list[A1 as usize] = ROOK;
                    bits::set_bit(&mut board.bb_side[us][ROOK], A1);
                    bits::set_bit(&mut board.bb_pieces[us], A1);
                }
                G8 => {
                    bits::clear_bit(&mut board.bb_side[us][ROOK], F8);
                    bits::clear_bit(&mut board.bb_pieces[us], F8);
                    board.piece_list[F8 as usize] = PNONE;

                    board.piece_list[H8 as usize] = ROOK;
                    bits::set_bit(&mut board.bb_side[us][ROOK], H8);
                    bits::set_bit(&mut board.bb_pieces[us], H8);
                }
                C8 => {
                    bits::clear_bit(&mut board.bb_side[us][ROOK], D8);
                    bits::clear_bit(&mut board.bb_pieces[us], D8);
                    board.piece_list[D8 as usize] = PNONE;

                    board.piece_list[A8 as usize] = ROOK;
                    bits::set_bit(&mut board.bb_side[us][ROOK], A8);
                    bits::set_bit(&mut board.bb_pieces[us], A8);
                }
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
