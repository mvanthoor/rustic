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

        // Establish base evaluation
        let mut value = Evaluation::pst_score(board);

        // Flip point of view if black is evaluating.
        value = if side == Sides::BLACK { -value } else { value };

        value
    }

    // Interpolate PST values between midgame and endgame tables.
    fn pst_score(board: &Board) -> i16 {
        // Get current PST values. These are kept incrementally during play.
        let pst_w_mg = board.game_state.pst_mg[Sides::WHITE] as f32;
        let pst_b_mg = board.game_state.pst_mg[Sides::BLACK] as f32;
        let pst_w_eg = board.game_state.pst_eg[Sides::WHITE] as f32;
        let pst_b_eg = board.game_state.pst_eg[Sides::BLACK] as f32;

        // Get the game phase, from 1 (opening/midgame) to 0 (endgame)
        let phase = Evaluation::phase(PHASE_MIN, PHASE_MAX, board.game_state.phase_value);

        // Mix the tables by taking parts of both mg and eg.
        let pst_w = (pst_w_mg * phase) + (pst_w_eg * (1.0 - phase));
        let pst_b = (pst_b_mg * phase) + (pst_b_eg * (1.0 - phase));

        // Return final PST score.
        (pst_w - pst_b).round() as i16
    }

    // Get the game phase by using the Linstep method.
    fn phase(edge0: i16, edge1: i16, value: i16) -> f32 {
        // Interpolate from edge0 to edge1.
        let result = (value - edge0) as f32 / (edge1 - edge0) as f32;

        // Clamp the result: don't exceed 1.0 or drop below 0.0.
        f32::min(1.0, f32::max(0.0, result))
    }
}
