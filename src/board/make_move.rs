use super::representation::{Board, UnMakeInfo};
use super::unmake_move::unmake_move;
use crate::defs::{
    Bitboard, A1, A8, C1, C8, CASTLE_BK, CASTLE_BQ, CASTLE_WK, CASTLE_WQ, D1, D8, F1, F8, G1, G8,
    H1, H8, KING, PAWN, PNONE, ROOK, WHITE,
};
use crate::movegen::information::square_attacked;
use crate::movegen::movedefs::Move;
use crate::movegen::MoveGenerator;
use crate::print;
use crate::utils::{clear_bit, set_bit};

pub fn make_move(board: &mut Board, m: Move, mg: &MoveGenerator) -> bool {
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
    let bb_mine: &mut [Bitboard];
    let bb_opponent: &mut [Bitboard];

    if us == WHITE {
        bb_mine = &mut board.bb_w;
        bb_opponent = &mut board.bb_b;
    } else {
        bb_mine = &mut board.bb_b;
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

    let normal_move = (promoted == PNONE) && !castling;
    let promotion_move = promoted != PNONE;

    // If a piece is captured by this move, then remove it from the to-square
    if captured != PNONE {
        board.zobrist_key ^= board.zobrist_randoms.piece(opponent, captured, to);
        clear_bit(&mut bb_opponent[captured], to);
        clear_bit(&mut board.bb_pieces[opponent], to);
    }

    // take the moving piece off the from-square
    board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, from);
    clear_bit(&mut bb_mine[piece], from);
    clear_bit(&mut board.bb_pieces[us], from);

    // put the moving piece on the to-square: normal move
    if normal_move {
        set_bit(&mut bb_mine[piece], to);
        set_bit(&mut board.bb_pieces[us], to);
        board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, to);
    }

    // put the moving piece on the to-square: promotion
    if promotion_move {
        set_bit(&mut bb_mine[promoted], to);
        set_bit(&mut board.bb_pieces[us], to);
        board.zobrist_key ^= board.zobrist_randoms.piece(us, promoted, to);
    }

    // We're castling. This is a special case.
    if castling {
        // Because of castling, we just picked up the king as a moving piece.
        // Put it down first. The to-square is contained in the king's move.
        set_bit(&mut bb_mine[piece], to);
        set_bit(&mut board.bb_pieces[us], to);
        board.zobrist_key ^= board.zobrist_randoms.piece(us, piece, to);

        // Now move the correct rook.
        if to == G1 {
            // White is castling short. Pick up rook h1.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, H1);
            clear_bit(&mut bb_mine[ROOK], H1);
            clear_bit(&mut board.bb_pieces[us], H1);

            // Put it back down on f1.
            set_bit(&mut bb_mine[ROOK], F1);
            set_bit(&mut board.bb_pieces[us], F1);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, F1);

            // Remove all castling permissions for white
            board.castling -= CASTLE_WK;
            board.castling -= CASTLE_WQ;
        }

        if to == C1 {
            // White is castling long. Pick up rook A1.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, A1);
            clear_bit(&mut bb_mine[ROOK], A1);
            clear_bit(&mut board.bb_pieces[us], A1);

            // Put it back down on d1.
            set_bit(&mut bb_mine[ROOK], D1);
            set_bit(&mut board.bb_pieces[us], D1);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, D1);

            // Remove all castling permissions for white.
            board.castling -= CASTLE_WK;
            board.castling -= CASTLE_WQ;
        }

        if to == G8 {
            // Black is castling short. Pick up rook h8.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, H8);
            clear_bit(&mut bb_mine[ROOK], H8);
            clear_bit(&mut board.bb_pieces[us], H8);

            // Put it back down on f8.
            set_bit(&mut bb_mine[ROOK], F8);
            set_bit(&mut board.bb_pieces[us], F8);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, F8);

            // Remove all castling permissions for black
            board.castling -= CASTLE_BK;
            board.castling -= CASTLE_BQ;
        }

        if to == C8 {
            // Black is castling long. Pick up rook a8.
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, A8);
            clear_bit(&mut bb_mine[ROOK], A8);
            clear_bit(&mut board.bb_pieces[us], A8);

            // Put it back down on d8.
            set_bit(&mut bb_mine[ROOK], D8);
            set_bit(&mut board.bb_pieces[us], D8);
            board.zobrist_key ^= board.zobrist_randoms.piece(us, ROOK, D8);

            // Remove all castling permissions for black.
            board.castling -= CASTLE_BK;
            board.castling -= CASTLE_BQ;
        }
    }

    // After the en-passant maneuver, the opponent's pawn has yet to be removed.
    if en_passant {
        let pawn_square = if us == WHITE { to - 8 } else { to + 8 };
        board.zobrist_key ^= board.zobrist_randoms.piece(opponent, PAWN, pawn_square);
        clear_bit(&mut bb_opponent[PAWN], pawn_square);
        clear_bit(&mut board.bb_pieces[opponent], pawn_square);
    }

    //region Updating the board state

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

    // increment move counter
    board.fullmove_number += 1;
    //endregion

    /*** Validating move ***/

    // Move is done. Check if it's actually legal. (King can not be in check.)
    let king_square = bb_mine[KING].trailing_zeros() as u8;
    let is_legal = !square_attacked(board, opponent, mg, king_square);

    if !is_legal {
        unmake_move(board);
    }

    is_legal
}
