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

pub mod defs;
mod phase;
mod pst;

use crate::{board::Board, defs::Sides};

pub struct Evaluation;
impl Evaluation {
    pub fn evaluate_position(board: &Board) -> i16 {
        // Determine the side which is evaluating.
        let side = board.game_state.active_color as usize;

        // Establish base evaluation value by PST score.
        let mut value = Evaluation::pst_score(board);

        // Flip point of view if black is evaluating.
        value = if side == Sides::BLACK { -value } else { value };

        value
    }
}
