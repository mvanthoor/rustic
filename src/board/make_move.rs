use super::representation::{Board, UnMakeInfo};
use super::unmake_move::unmake_move;
use crate::defs::{
    Bitboard, A1, A8, BLACK, C1, C8, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, D1, D8, E1, E8,
    F1, F8, G1, G8, H1, H8, KING, PAWN, PNONE, ROOK, WHITE,
};
use crate::movegen::information::square_attacked;
use crate::movegen::movedefs::Move;
use crate::utils::{clear_bit, set_bit};

#[allow(clippy::cognitive_complexity)]
pub fn make_move(board: &mut Board, m: Move) -> bool {
    // create the unmake info and store it.
    let unmake_info = UnMakeInfo::new(
        board.active_color,
        board.castling,
        board.en_passant,
        board.halfmove_clock,
        board.fullmove_number,
        board.zobrist_key,
        m,
    );
    board.unmake_list.push(unmake_info);

    // Set "us" and "opponent"
    let us = board.active_color as usize;
    let opponent = (us ^ 1) as usize;

    // Set which bitboards are "us" and "opponent" pieces
    let bb_us: &mut [Bitboard];
    let bb_opponent: &mut [Bitboard];

    if us == WHITE {
        bb_us = &mut board.bb_w;
        bb_opponent = &mut board.bb_b;
    } else {
        bb_us = &mut board.bb_b;
        bb_opponent = &mut board.bb_w;
    };

    // Dissect the move
    let piece = m.piece() as usize;
    let from = m.from();
    let to = m.to();
    let captured = m.captured() as usize;
    let promoted = m.promoted() as usize;
    let castling = m.castling();
    let double_step = m.double_step();
    let en_passant = m.en_passant();

    let promotion_move = promoted != PNONE;

    // If a piece is captured by this move, then remove it from the to-square
    if captured != PNONE {
        board.zobrist_key ^= board.zobrist_randoms.piece(opponent, captured, to);
        clear_bit(&mut bb_opponent[captured], to);
        clear_bit(&mut board.bb_pieces[opponent], to);

        // If a rook in the corner is captured, drop the corresponding castling permissions.
        if captured == ROOK {
            // Remove current castling permissions from zobrist key.
            board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);

            // Remove the correct castling permissions from the position.
            if us == BLACK && to == H1 {
                board.castling &= !CASTLE_WK;
            };
            if us == BLACK && to == A1 {
                board.castling &= !CASTLE_WQ;
            };
            if us == WHITE && to == H8 {
                board.castling &= !CASTLE_BK;
            };
            if us == WHITE && to == A8 {
                board.castling &= !CASTLE_BQ;
            };

            // Add the new castling permission state back into the zobrist key.
            board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);
        }
    }

    // take the moving piece off the from-square
    board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, from);
    clear_bit(&mut bb_us[piece], from);
    clear_bit(&mut board.bb_pieces[us], from);

    // put the moving piece on the to-square
    if !promotion_move {
        // normal move (including the king part of castling).
        set_bit(&mut bb_us[piece], to);
        set_bit(&mut board.bb_pieces[us], to);
        board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, to);
    } else {
        // promotion move. Put promotion piece on the to-square instead of the pawn.
        set_bit(&mut bb_us[promoted], to);
        set_bit(&mut board.bb_pieces[us], to);
        board.zobrist_key ^= board.zobrist_randoms.piece(us, promoted, to);
    }

    // We're castling. This is a special case.
    if castling {
        // remove current castling rights from the zobrist key.
        board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);

        // The king was already moved as a "normal" move. Now move the correct rook.
        if to == G1 {
            // White is castling short. Pick up rook h1.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, H1);
            clear_bit(&mut bb_us[ROOK], H1);
            clear_bit(&mut board.bb_pieces[us], H1);

            // Put it back down on f1.
            set_bit(&mut bb_us[ROOK], F1);
            set_bit(&mut board.bb_pieces[us], F1);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, F1);

            // Remove all castling permissions for white (clear bits 0 and 1)
            board.castling &= !(CASTLE_WK + CASTLE_WQ);
        }

        if to == C1 {
            // White is castling long. Pick up rook A1.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, A1);
            clear_bit(&mut bb_us[ROOK], A1);
            clear_bit(&mut board.bb_pieces[us], A1);

            // Put it back down on d1.
            set_bit(&mut bb_us[ROOK], D1);
            set_bit(&mut board.bb_pieces[us], D1);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, D1);

            // Remove all castling permissions for white (clear bits 0 and 1)
            board.castling &= !(CASTLE_WK + CASTLE_WQ);
        }

        if to == G8 {
            // Black is castling short. Pick up rook h8.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, H8);
            clear_bit(&mut bb_us[ROOK], H8);
            clear_bit(&mut board.bb_pieces[us], H8);

            // Put it back down on f8.
            set_bit(&mut bb_us[ROOK], F8);
            set_bit(&mut board.bb_pieces[us], F8);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, F8);

            // Remove all castling permissions for black (clear bits 2 and 3)
            board.castling &= !(CASTLE_BK + CASTLE_BQ);
        }

        if to == C8 {
            // Black is castling long. Pick up rook a8.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, A8);
            clear_bit(&mut bb_us[ROOK], A8);
            clear_bit(&mut board.bb_pieces[us], A8);

            // Put it back down on d8.
            set_bit(&mut bb_us[ROOK], D8);
            set_bit(&mut board.bb_pieces[us], D8);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, D8);

            // Remove all castling permissions for black (clear bits 2 and 3)
            board.castling &= !(CASTLE_BK + CASTLE_BQ);
        }

        // add resulting castling permissions to the zobrist key.
        board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);
    }

    // After the en-passant maneuver, the opponent's pawn has yet to be removed.
    if en_passant {
        let pawn_square = if us == WHITE { to - 8 } else { to + 8 };
        board.zobrist_key ^= board.zobrist_randoms.piece(opponent, PAWN, pawn_square);
        clear_bit(&mut bb_opponent[PAWN], pawn_square);
        clear_bit(&mut board.bb_pieces[opponent], pawn_square);
    }

    //region: Updating the board state

    // If moving the king or one of the rooks, castling permissions are dropped.
    if !castling && (piece == KING || piece == ROOK) {
        // remove current castling permissions from zobrist key.
        board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);

        if us == WHITE && from == H1 {
            // remove kingside castling (clear bit 0)
            board.castling &= !CASTLE_WK;
        }
        if us == WHITE && from == A1 {
            // remove queenside castling (clear bit 1)
            board.castling &= !CASTLE_WQ;
        }
        if us == WHITE && from == E1 {
            // remove both castling rights (clear bit 0 and 1)
            board.castling &= !(CASTLE_WK + CASTLE_WQ);
        }

        if us == BLACK && from == H8 {
            // remove kingside castling (clear bit 2)
            board.castling &= !CASTLE_BK;
        }
        if us == BLACK && from == A8 {
            // remove queenside castling (clear bit 3)
            board.castling &= !CASTLE_BQ;
        }
        if us == BLACK && from == E8 {
            // remove all castling rights (clear bit 2 and 3)
            board.castling &= !(CASTLE_BK + CASTLE_BQ);
        }

        // add resulting castling rights back into zobrist key.
        board.zobrist_key ^= board.zobrist_randoms.castling(board.castling);
    }

    // If the en-passant square is set, every move will unset it...
    if board.en_passant.is_some() {
        board.zobrist_key ^= board.zobrist_randoms.en_passant(board.en_passant);
        board.en_passant = None;
    }

    // ...except a pawn double-step, which will set it (again, if just unset).
    if double_step {
        board.en_passant = if us == WHITE {
            Some(to - 8)
        } else {
            Some(to + 8)
        };
        board.zobrist_key ^= board.zobrist_randoms.en_passant(board.en_passant);
    }

    // change the color to move: out with "us", in with "opponent"
    board.zobrist_key ^= board.zobrist_randoms.side(us);
    board.zobrist_key ^= board.zobrist_randoms.side(opponent);
    board.active_color = opponent as u8;

    // If a pawn moves or a piece is captured, reset the 50-move counter.
    // Otherwise, increment the counter by one move.
    if (piece == PAWN) || (captured != PNONE) {
        board.halfmove_clock = 0;
    } else {
        board.halfmove_clock += 1;
    }

    // Increase full move number if black has moved
    if us == BLACK {
        board.fullmove_number += 1;
    }
    //endregion

    /*** Validating move ***/

    // Move is done. Check if it's actually legal. (King can not be in check.)
    let king_square = bb_us[KING].trailing_zeros() as u8;
    let is_legal = !square_attacked(board, opponent, king_square);

    if !is_legal {
        unmake_move(board);
    }

    is_legal
}
