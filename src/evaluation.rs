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

use self::defs::{PHASE_MAX, PHASE_MIN};

pub struct Evaluation;
impl Evaluation {
    pub fn evaluate_position(board: &Board) -> i16 {
        let side = board.game_state.active_color as usize;

        // Get PST values
        let pst_w_mg = board.game_state.pst_mg[Sides::WHITE] as f32;
        let pst_b_mg = board.game_state.pst_mg[Sides::BLACK] as f32;
        let pst_w_eg = board.game_state.pst_eg[Sides::WHITE] as f32;
        let pst_b_eg = board.game_state.pst_eg[Sides::BLACK] as f32;

        // Get phase
        let phase = Evaluation::phase(PHASE_MAX, PHASE_MIN, board.game_state.phase_value);

        // Mix the tables by interpolation
        let pst_w = pst_w_mg + (phase * (pst_w_eg - pst_w_mg));
        let pst_b = pst_b_mg + (phase * (pst_b_eg - pst_b_mg));

        // Establish base evaluation
        let mut value = (pst_w - pst_b).round() as i16;

        // Flip point of view if black is evaluating.
        value = if side == Sides::BLACK { -value } else { value };

        value
    }

    pub fn phase(edge0: i16, edge1: i16, value: i16) -> f32 {
        let clamp = (value - edge0) as f32 / (edge1 - edge0) as f32;
        f32::min(1.0, f32::max(0.0, clamp))
    }
}
