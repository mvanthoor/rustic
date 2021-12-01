/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
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

use crate::{board::Board, defs::MAX_MOVE_RULE};

impl Board {
    // Returns true if the position should be evaluated as a draw.
    pub fn is_draw(&self) -> bool {
        (!self.is_checkmate_possible())
            || self.is_draw_by_repetition() > 0
            || self.is_draw_by_fifty_move_rule()
    }

    // Checks the 50-move rule.
    pub fn is_draw_by_fifty_move_rule(&self) -> bool {
        self.game_state.halfmove_clock >= MAX_MOVE_RULE
    }

    // For some positions with insufficient material a draw can be claimed
    // according to FIDE rules.
    pub fn is_draw_by_insufficient_material(&self) -> bool {
        false
    }

    // Detects position repetitions in the game's history and returns the
    // of times a position was repeated.
    pub fn is_draw_by_repetition(&self) -> u8 {
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
}
