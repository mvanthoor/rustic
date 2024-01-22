/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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
pub mod psqt;

use crate::{board::Board, defs::Sides};
use psqt::KING_EDGE;

pub fn evaluate_position(board: &Board) -> i16 {
    const KING_ONLY: i16 = 300; // PSQT-points
    let side = board.game_state.active_color as usize;
    let w_psqt = board.game_state.psqt[Sides::WHITE];
    let b_psqt = board.game_state.psqt[Sides::BLACK];
    let mut value = w_psqt - b_psqt;

    // If one of the sides is down to a bare king, apply the KING_EDGE PSQT
    // to drive that king to the edge and mate it.
    if w_psqt < KING_ONLY || b_psqt < KING_ONLY {
        let w_king_edge = KING_EDGE[board.king_square(Sides::WHITE)];
        let b_king_edge = KING_EDGE[board.king_square(Sides::BLACK)];
        value += w_king_edge - b_king_edge;
    }

    // This function calculates the evaluation from white's point of view:
    // a positive value means "white is better", a negative value means
    // "black is better". Alpha/Beta requires the value returned from the
    // viewpoint of the side that is being evaluated. Therefore if it is
    // black to move, the value must first be flipped to black's viewpoint
    // before it can be returned.

    value = if side == Sides::BLACK { -value } else { value };

    value
}
