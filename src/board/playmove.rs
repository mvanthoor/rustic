use super::{representation::Board, Pieces, Squares, BB_SQUARES};
use crate::defs::{Castling, Piece, Side, Square, BLACK, NR_OF_SQUARES, WHITE};
use crate::evaluation::evaldefs::PIECE_VALUES;
use crate::movegen::{info, movedefs::Move};

// Full castling permissions are 1111, or value 15. CASTLE_ALL = All castling
// permissions for both sides. N_WKQ = Not White Kingside/Queenside, and so on.
const N_WKQ: u8 = Castling::ALL & !(Castling::WK | Castling::WQ);
const N_BKQ: u8 = Castling::ALL & !(Castling::BK | Castling::BQ);
const N_WK: u8 = Castling::ALL & !Castling::WK;
const N_WQ: u8 = Castling::ALL & !Castling::WQ;
const N_BK: u8 = Castling::ALL & !Castling::BK;
const N_BQ: u8 = Castling::ALL & !Castling::BQ;

#[rustfmt::skip]
// First element in this array is square A1. The N_* constants mark which
// castling rights are lost if the king or rook moves from that starting square.
const CASTLING_PERMS: [u8; NR_OF_SQUARES] = [
    N_WQ, 15,  15,  15,  N_WKQ, 15,  15,  N_WK,
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15,
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    N_BQ, 15,  15,  15,  N_BKQ, 15,  15,  N_BK,
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
    let piece = m.piece();
    let from = m.from();
    let to = m.to();
    let captured = m.captured();
    let promoted = m.promoted();
    let castling = m.castling();
    let double_step = m.double_step();
    let en_passant = m.en_passant();
    let is_promotion = promoted != Pieces::NONE;
    let is_capture = captured != Pieces::NONE;

    // Assume this is not a pawn move or a capture.
    board.game_state.halfmove_clock += 1;

    // Every move except double_step unsets the up-square.
    if board.game_state.en_passant != None {
        board.clear_ep_square();
    }

    // If a piece was captured with this move then remove it. Also reset halfmove_clock.
    if is_capture {
        board.remove_piece(opponent, captured, to);
        board.game_state.halfmove_clock = 0;
        // Change castling permissions on rook capture in the corner.
        if captured == Pieces::ROOK && (board.game_state.castling > 0) {
            board.zobrist_castling();
            board.game_state.castling &= CASTLING_PERMS[to as usize];
            board.zobrist_castling();
        }
    }

    // Make the move. Just move the piece if it's not a pawn.
    if !(piece == Pieces::PAWN) {
        board.move_piece(us, piece, from, to);
    } else {
        // It's a pawn move. Take promotion into account and reset halfmove_clock.
        board.remove_piece(us, piece, from);
        board.put_piece(us, if !is_promotion { piece } else { promoted }, to);
        board.game_state.halfmove_clock = 0;

        // After an en-passant maneuver, the opponent's pawn must also be removed.
        if en_passant {
            board.remove_piece(opponent, Pieces::PAWN, to ^ 8);
        }

        // A double-step is the only move that sets the ep-square.
        if double_step {
            board.set_ep_square(to ^ 8);
        }
    }

    // Remove castling permissions if king/rook leaves from starting square.
    // (This will also adjust permissions when castling, because the king moves.)
    if (CASTLING_PERMS[from as usize] != Castling::ALL) && (board.game_state.castling > 0) {
        board.zobrist_castling();
        board.game_state.castling &= CASTLING_PERMS[from as usize];
        board.zobrist_castling();
    }

    // If the king is castling, then also move the rook.
    if castling {
        match to {
            Squares::G1 => board.move_piece(us, Pieces::ROOK, Squares::H1, Squares::F1),
            Squares::C1 => board.move_piece(us, Pieces::ROOK, Squares::A1, Squares::D1),
            Squares::G8 => board.move_piece(us, Pieces::ROOK, Squares::H8, Squares::F8),
            Squares::C8 => board.move_piece(us, Pieces::ROOK, Squares::A8, Squares::D8),
            _ => panic!("Error moving rook during castling."),
        }
    }

    // Swap the side to move.
    board.swap_side();

    // Increase full move number if black has moved
    if us == BLACK {
        board.game_state.fullmove_number += 1;
    }

    /*** Validating move: see if "us" is in check. If so, undo everything. ***/
    let king_square = board.get_pieces(Pieces::KING, us).trailing_zeros() as Square;
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
    board.game_state = board.history.pop();

    // Set "us" and "opponent"
    let us = board.game_state.active_color as usize;
    let opponent = (us ^ 1) as usize;

    // Dissect the move to undo
    let m = board.game_state.this_move;
    let piece = m.piece();
    let from = m.from();
    let to = m.to();
    let captured = m.captured();
    let promoted = m.promoted();
    let castling = m.castling();
    let en_passant = m.en_passant();

    // Moving backwards...
    if promoted == Pieces::NONE {
        reverse_move(board, us, piece, to, from);
    } else {
        remove_piece(board, us, promoted, to);
        put_piece(board, us, Pieces::PAWN, from);
    }

    // The king's move was already undone as a normal move.
    // Now undo the correct castling rook move.
    if castling {
        match to {
            Squares::G1 => reverse_move(board, us, Pieces::ROOK, Squares::F1, Squares::H1),
            Squares::C1 => reverse_move(board, us, Pieces::ROOK, Squares::D1, Squares::A1),
            Squares::G8 => reverse_move(board, us, Pieces::ROOK, Squares::F8, Squares::H8),
            Squares::C8 => reverse_move(board, us, Pieces::ROOK, Squares::D8, Squares::A8),
            _ => panic!("Error: Reversing castling rook move."),
        };
    }

    // If a piece was captured, put it back onto the to-square
    if captured != Pieces::NONE {
        put_piece(board, opponent, captured, to);
    }

    // If this was an e-passant move, put the opponent's pawn back
    if en_passant {
        let ep_square = if us == WHITE { to - 8 } else { to + 8 };
        put_piece(board, opponent, Pieces::PAWN, ep_square);
    }
}

// ===== Helper functions to reverse piece moves without doing zobrist updates. =====

// Removes a piece from the board.
fn remove_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_side[side][piece] ^= BB_SQUARES[square];
    board.bb_pieces[side] ^= BB_SQUARES[square];
    board.piece_list[square] = Pieces::NONE;
    board.material_count[side] -= PIECE_VALUES[piece];
}

// Puts a piece onto the board.
fn put_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_side[side][piece] |= BB_SQUARES[square];
    board.bb_pieces[side] |= BB_SQUARES[square];
    board.piece_list[square] = piece;
    board.material_count[side] += PIECE_VALUES[piece];
}

// Moves a piece from one square to the other.
fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: Square, put: Square) {
    remove_piece(board, side, piece, remove);
    put_piece(board, side, piece, put);
}
