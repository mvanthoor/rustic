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

// Move sorting routines.

use super::Search;
use crate::{defs::NrOf, movegen::defs::MoveList, movegen::defs::TTMove};

const TTMOVE_SORT_VALUE: u8 = 60;

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u8; NrOf::PIECE_TYPES + 1]; NrOf::PIECE_TYPES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim K, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

impl Search {
    pub fn score_moves(ml: &mut MoveList, tt_move: TTMove) {
        for i in 0..ml.len() {
            let m = ml.get_mut_move(i);
            let is_tt_move = m.get_move() == tt_move.get_move();

            // Sort the hash move as the first in the list.
            let value = if is_tt_move {
                TTMOVE_SORT_VALUE
            } else {
                // Sort other moves by MVV-LVA scheme.
                MVV_LVA[m.captured()][m.piece()]
            };

            m.set_score(value);
        }
    }

    pub fn pick_move(ml: &mut MoveList, start_index: u8) {
        for i in (start_index + 1)..ml.len() {
            if ml.get_move(i).sort_score() > ml.get_move(start_index).sort_score() {
                ml.swap(start_index as usize, i as usize);
            }
        }
    }
}
