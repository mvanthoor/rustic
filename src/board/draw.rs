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

use crate::{
    board::Board,
    defs::{Sides, MAX_MOVE_RULE},
};

use super::defs::Pieces;

impl Board {
    // Returns true if the position should be evaluated as a draw.
    pub fn is_draw(&self) -> bool {
        (!self.sufficient_material_to_force_checkmate())
            || self.draw_by_repetition_rule() > 0
            || self.draw_by_fifty_move_rule()
    }

    // Checks for the 50-move rule.
    pub fn draw_by_fifty_move_rule(&self) -> bool {
        self.game_state.halfmove_clock >= MAX_MOVE_RULE
    }

    // This function returns true if the amount of material on the board is
    // not sufficient to deliver checkmate using any sequence of legal
    // moves, even if the losing side is trying to assist in getting
    // checkmated. In such a position a draw can officially be claimed
    // under FIDE rules. Note that this is different from
    // sufficient_material_to_force_checkmate().
    pub fn draw_by_insufficient_material_rule(&self) -> bool {
        false
    }

    // Detects position repetitions in the game's history and returns the
    // of times a position was repeated.
    pub fn draw_by_repetition_rule(&self) -> u8 {
        let mut count = 0;
        let mut stop = false;
        let mut i = (self.history.len() - 1) as i16;

        // Search the history list.
        while i >= 0 && !stop {
            let historic = self.history.get_ref(i as usize);

            // If the historic zobrist key is equal to the one of the board
            // passed into the function, then we found a repetition.
            if historic.zobrist_key == self.game_state.zobrist_key {
                count += 1;
            }

            // If the historic HMC is 0, it indicates that this position
            // was created by a capture or pawn move. We don't have to
            // search further back, because before this, we can't ever
            // repeat. After all, the capture or pawn move can't be
            // reverted or repeated.
            stop = historic.halfmove_clock == 0;

            // Search backwards.
            i -= 1;
        }
        count
    }

    // This function determines if there is enough material available for
    // one of the sides to force a checkmate, even with best play from the
    // losing side. If mate cannot be forced (even though it is
    // theoretically still possible if the losing side plays poorly, or
    // even assists in getting mated), then this function returns false.
    pub fn sufficient_material_to_force_checkmate(&self) -> bool {
        self.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0
            || self.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0
            || self.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0
            || self.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0
            || self.has_bishop_pair(Sides::WHITE)
            || self.has_bishop_pair(Sides::BLACK)
            || (self.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() >= 1
                && self.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() >= 1)
            || (self.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() >= 1
                && self.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() >= 1)
            || self.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() >= 3
            || self.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() >= 3
    }
}
