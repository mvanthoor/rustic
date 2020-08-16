// playmove.rs contains make() and unamke() for move execution and reversal.

use super::{
    defs::{Pieces, Squares, BB_SQUARES},
    Board,
};
use crate::{
    defs::{Castling, NrOf, Piece, Side, Sides, Square},
    evaluation::defs::PIECE_VALUES,
    movegen::{defs::Move, MoveGenerator},
};

// Full castling permissions are 1111, or value 15. CASTLE_ALL = All castling
// permissions for both sides. N_WKQ = Not White Kingside/Queenside, and so on.
const N_WKQ: u8 = Castling::ALL & !Castling::WK & !Castling::WQ;
const N_BKQ: u8 = Castling::ALL & !Castling::BK & !Castling::BQ;
const N_WK: u8 = Castling::ALL & !Castling::WK;
const N_WQ: u8 = Castling::ALL & !Castling::WQ;
const N_BK: u8 = Castling::ALL & !Castling::BK;
const N_BQ: u8 = Castling::ALL & !Castling::BQ;

#[rustfmt::skip]
// First element in this array is square A1. The N_* constants mark which
// castling rights are lost if the king or rook moves from that starting square.
const CASTLING_PERMS: [u8; NrOf::SQUARES] = [
    N_WQ, 15,  15,  15,  N_WKQ, 15,  15,  N_WK,
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15,
    15,   15,  15,  15,  15,    15,  15,  15, 
    15,   15,  15,  15,  15,    15,  15,  15, 
    N_BQ, 15,  15,  15,  N_BKQ, 15,  15,  N_BK,
];

/*** ================================================================================ ***/

// Make() executes the given move and checks if it is legal. If it's not legal,
// the move is immediately reversed using unmake(), and the board is not changed.

impl Board {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn make(&mut self, m: Move, mg: &MoveGenerator) -> bool {
        // Create the unmake info and store it.
        let mut current_game_state = self.game_state;
        current_game_state.next_move = m;
        self.history.push(current_game_state);

        // Set "us" and "opponent"
        let us = self.us();
        let opponent = us ^ 1;

        // Dissect the move so we don't need "m.function()" and type casts everywhere.
        let piece = m.piece();
        let from = m.from();
        let to = m.to();
        let captured = m.captured();
        let promoted = m.promoted();
        let castling = m.castling();
        let double_step = m.double_step();
        let en_passant = m.en_passant();

        // Shorthands
        let is_promotion = promoted != Pieces::NONE;
        let is_capture = captured != Pieces::NONE;
        let has_permissions = self.game_state.castling > 0;

        // Assume this is not a pawn move or a capture.
        self.game_state.halfmove_clock += 1;

        // Every move except double_step unsets the up-square.
        if self.game_state.en_passant != None {
            self.clear_ep_square();
        }

        // If a piece was captured with this move then remove it. Also reset halfmove_clock.
        if is_capture {
            self.remove_piece(opponent, captured, to);
            self.game_state.halfmove_clock = 0;
            // Change castling permissions on rook capture in the corner.
            if captured == Pieces::ROOK && has_permissions {
                self.update_castling_permissions(self.game_state.castling & CASTLING_PERMS[to]);
            }
        }

        // Make the move. Just move the piece if it's not a pawn.
        if !(piece == Pieces::PAWN) {
            self.move_piece(us, piece, from, to);
        } else {
            // It's a pawn move. Take promotion into account and reset halfmove_clock.
            self.remove_piece(us, piece, from);
            self.put_piece(us, if !is_promotion { piece } else { promoted }, to);
            self.game_state.halfmove_clock = 0;

            // After an en-passant maneuver, the opponent's pawn must also be removed.
            if en_passant {
                self.remove_piece(opponent, Pieces::PAWN, to ^ 8);
            }

            // A double-step is the only move that sets the ep-square.
            if double_step {
                self.set_ep_square(to ^ 8);
            }
        }

        // Remove castling permissions if king/rook leaves from starting square.
        // (This will also adjust permissions when castling, because the king moves.)
        if (piece == Pieces::KING || piece == Pieces::ROOK) && has_permissions {
            self.update_castling_permissions(self.game_state.castling & CASTLING_PERMS[from]);
        }

        // If the king is castling, then also move the rook.
        if castling {
            match to {
                Squares::G1 => self.move_piece(us, Pieces::ROOK, Squares::H1, Squares::F1),
                Squares::C1 => self.move_piece(us, Pieces::ROOK, Squares::A1, Squares::D1),
                Squares::G8 => self.move_piece(us, Pieces::ROOK, Squares::H8, Squares::F8),
                Squares::C8 => self.move_piece(us, Pieces::ROOK, Squares::A8, Squares::D8),
                _ => panic!("Error moving rook during castling."),
            }
        }

        // Swap the side to move.
        self.swap_side();

        // Increase full move number if black has moved
        if us == Sides::BLACK {
            self.game_state.fullmove_number += 1;
        }

        /*** Validating move: see if "us" is in check. If so, undo everything. ***/
        let is_legal = !mg.square_attacked(self, opponent, self.king_square(us));
        if !is_legal {
            self.unmake();
        }

        is_legal
    }
}

/*** ================================================================================ ***/

// Unmake() reverses the last move. The game state is restored by popping it
// from the history array, all variables at once.
impl Board {
    #[cfg_attr(debug_assertions, inline(never))]
    #[cfg_attr(not(debug_assertions), inline(always))]
    pub fn unmake(&mut self) {
        self.game_state = self.history.pop();

        // Set "us" and "opponent"
        let us = self.us();
        let opponent = us ^ 1;

        // Dissect the move to undo
        let m = self.game_state.next_move;
        let piece = m.piece();
        let from = m.from();
        let to = m.to();
        let captured = m.captured();
        let promoted = m.promoted();
        let castling = m.castling();
        let en_passant = m.en_passant();

        // Moving backwards...
        if promoted == Pieces::NONE {
            reverse_move(self, us, piece, to, from);
        } else {
            remove_piece(self, us, promoted, to);
            put_piece(self, us, Pieces::PAWN, from);
        }

        // The king's move was already undone as a normal move.
        // Now undo the correct castling rook move.
        if castling {
            match to {
                Squares::G1 => reverse_move(self, us, Pieces::ROOK, Squares::F1, Squares::H1),
                Squares::C1 => reverse_move(self, us, Pieces::ROOK, Squares::D1, Squares::A1),
                Squares::G8 => reverse_move(self, us, Pieces::ROOK, Squares::F8, Squares::H8),
                Squares::C8 => reverse_move(self, us, Pieces::ROOK, Squares::D8, Squares::A8),
                _ => panic!("Error: Reversing castling rook move."),
            };
        }

        // If a piece was captured, put it back onto the to-square
        if captured != Pieces::NONE {
            put_piece(self, opponent, captured, to);
        }

        // If this was an e-passant move, put the opponent's pawn back
        if en_passant {
            put_piece(self, opponent, Pieces::PAWN, to ^ 8);
        }
    }
}

/*** Functions local to playmove.rs ====================================================== ***/

// unamke() pops the entire game history from a list at the beginning,
// including the Zobrist-key. Therefore the Zobrist-key is already set to
// what it should be at the end of unmake(). Using the Board's functions
// would calculate on that already finished Zobrist key and thus mess it
// up. These helper functions don't calculate the Zobrist key.

// Removes a piece from the board without Zobrist key updates.
fn remove_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_side[side][piece] ^= BB_SQUARES[square];
    board.bb_pieces[side] ^= BB_SQUARES[square];
    board.piece_list[square] = Pieces::NONE;
    board.material_count[side] -= PIECE_VALUES[piece];
}

// Puts a piece onto the board without Zobrist key updates.
fn put_piece(board: &mut Board, side: Side, piece: Piece, square: Square) {
    board.bb_side[side][piece] |= BB_SQUARES[square];
    board.bb_pieces[side] |= BB_SQUARES[square];
    board.piece_list[square] = piece;
    board.material_count[side] += PIECE_VALUES[piece];
}

// Moves a piece from one square to another.
fn reverse_move(board: &mut Board, side: Side, piece: Piece, remove: Square, put: Square) {
    remove_piece(board, side, piece, remove);
    put_piece(board, side, piece, put);
}

/*** ======================================================================================= ***/

// This function can be used to check if the Zobrist-key and material count
// are updated correctly during make() and unmake().

// TODO: Change this function so it can be used in a debug_assert!() statement.
#[allow(dead_code)]
fn checkup(board: &Board, m: Move) {
    let key = board.init_zobrist_key();
    let count = crate::evaluation::material::count(&board);

    if key != board.game_state.zobrist_key {
        println!("Error in Zobrist-key.");
        crate::extra::print::move_data(m);
        panic!();
    };

    if count.0 != board.material_count[Sides::WHITE] {
        println!("Error in material count for White.");
        crate::extra::print::move_data(m);
        panic!();
    };

    if count.1 != board.material_count[Sides::BLACK] {
        println!("Error in material count for Black.");
        crate::extra::print::move_data(m);
        panic!();
    };
}
