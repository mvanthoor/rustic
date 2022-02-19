/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2022, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

pub mod defs;
mod display;
mod draw;
mod gamestate;
mod history;
mod init;
mod playmove;
mod to_fen;
mod utils;
mod zobrist;

use self::{
    defs::{Pieces, BB_SQUARES},
    gamestate::GameState,
    history::History,
    zobrist::ZobristRandoms,
};
use crate::{
    defs::{Bitboard, NrOf, Piece, Side, Sides, Square, EMPTY},
    evaluation::defs::{PHASE_VALUES, PST_EG, PST_MG},
    misc::bits,
};
use std::sync::Arc;

// This file implements the engine's board representation; it is bit-board
// based, with the least significant bit being A1.
#[derive(Clone)]
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub game_state: GameState,
    pub history: History,
    pub piece_list: [Piece; NrOf::SQUARES],
    zr: Arc<ZobristRandoms>,
}

// Public functions for use by other modules.
impl Board {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            game_state: GameState::new(),
            history: History::new(),
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            zr: Arc::new(ZobristRandoms::new()),
        }
    }

    // Return a bitboard with locations of a certain piece type for one of the sides.
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    pub fn get_bitboards(&self, side: Side) -> &[u64; NrOf::PIECE_TYPES] {
        &self.bb_pieces[side]
    }

    // Return a bitboard containing all the pieces on the board.
    pub fn occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    // Returns the side to move.
    pub fn us(&self) -> usize {
        self.game_state.active_color as usize
    }

    // Returns the side that is NOT moving.
    pub fn opponent(&self) -> usize {
        (self.game_state.active_color ^ 1) as usize
    }

    // Returns the square the king is currently on.
    pub fn king_square(&self, side: Side) -> Square {
        self.bb_pieces[side][Pieces::KING].trailing_zeros() as Square
    }

    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
        self.piece_list[square] = Pieces::NONE;
        self.game_state.zobrist_key ^= self.zr.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        self.game_state.phase_value -= PHASE_VALUES[piece];

        let s = Board::flip(side, square);
        self.game_state.pst_mg[side] -= PST_MG[piece][s];
        self.game_state.pst_eg[side] -= PST_EG[piece][s];
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.piece_list[square] = piece;
        self.game_state.zobrist_key ^= self.zr.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        self.game_state.phase_value += PHASE_VALUES[piece];

        let s = Board::flip(side, square);
        self.game_state.pst_mg[side] += PST_MG[piece][s];
        self.game_state.pst_eg[side] += PST_EG[piece][s];
    }

    // Remove a piece from the from-square, and put it onto the to-square.
    pub fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: Square) {
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = Some(square as u8);
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
    }

    // Clear the ep-square. (If the ep-square is None already, nothing changes.)
    pub fn clear_ep_square(&mut self) {
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = None;
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
    }

    // Swap side from WHITE <==> BLACK
    pub fn swap_side(&mut self) {
        self.game_state.zobrist_key ^= self.zr.side(self.game_state.active_color as usize);
        self.game_state.active_color ^= 1;
        self.game_state.zobrist_key ^= self.zr.side(self.game_state.active_color as usize);
    }

    // Update castling permissions and take Zobrist-key into account.
    pub fn update_castling_permissions(&mut self, new_permissions: u8) {
        self.game_state.zobrist_key ^= self.zr.castling(self.game_state.castling);
        self.game_state.castling = new_permissions;
        self.game_state.zobrist_key ^= self.zr.castling(self.game_state.castling);
    }

    pub fn has_bishop_pair(&self, side: Side) -> bool {
        let mut bishops = self.get_pieces(Pieces::BISHOP, side);
        let mut white_square = 0;
        let mut black_square = 0;

        if bishops.count_ones() >= 2 {
            while bishops > 0 {
                let square = bits::next(&mut bishops);

                if Board::is_white_square(square) {
                    white_square += 1;
                } else {
                    black_square += 1;
                }
            }
        }

        white_square >= 1 && black_square >= 1
    }
}
