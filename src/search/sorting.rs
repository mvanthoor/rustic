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

use super::{
    defs::{SearchRefs, MAX_KILLER_MOVES},
    Search,
};
use crate::{board::defs::Pieces, defs::NrOf, movegen::defs::MoveList, movegen::defs::ShortMove};

const MVV_LVA_OFFSET: u32 = u32::MAX - 256;
const TTMOVE_SORT_VALUE: u32 = 60;
const KILLER_VALUE: u32 = 10;

// MVV_VLA[victim][attacker]
pub const MVV_LVA: [[u16; NrOf::PIECE_TYPES + 1]; NrOf::PIECE_TYPES + 1] = [
    [0, 0, 0, 0, 0, 0, 0],       // victim K, attacker K, Q, R, B, N, P, None
    [50, 51, 52, 53, 54, 55, 0], // victim Q, attacker K, Q, R, B, N, P, None
    [40, 41, 42, 43, 44, 45, 0], // victim R, attacker K, Q, R, B, N, P, None
    [30, 31, 32, 33, 34, 35, 0], // victim B, attacker K, Q, R, B, N, P, None
    [20, 21, 22, 23, 24, 25, 0], // victim K, attacker K, Q, R, B, N, P, None
    [10, 11, 12, 13, 14, 15, 0], // victim P, attacker K, Q, R, B, N, P, None
    [0, 0, 0, 0, 0, 0, 0],       // victim None, attacker K, Q, R, B, N, P, None
];

impl Search {
    pub fn score_moves(ml: &mut MoveList, tt_move: ShortMove, refs: &SearchRefs) {
        for i in 0..ml.len() {
            let m = ml.get_mut_ref_move(i);
            let mut value: u32 = 0;

            // Sort order priority is: TT Move first, then captures, then
            // quiet moves that are in the list of killer moves.
            if m.get_move() == tt_move.get_move() {
                value = MVV_LVA_OFFSET + TTMOVE_SORT_VALUE;
            } else if m.captured() != Pieces::NONE {
                // Order captures higher than MVV_LVA_OFFSET
                value = MVV_LVA_OFFSET + MVV_LVA[m.captured()][m.piece()] as u32;
            } else {
                let ply = refs.search_info.ply as usize;
                let mut n = 0;
                while n < MAX_KILLER_MOVES && value == 0 {
                    let killer = refs.search_info.killer_moves[ply][n];
                    if m.get_move() == killer.get_move() {
                        // Order killers below MVV_LVA_OFFSET
                        value = MVV_LVA_OFFSET - ((n as u32 + 1) * KILLER_VALUE);
                    }
                    n += 1;
                }
            }

            m.set_sort_score(value);
        }
    }

    // This function puts the move with the highest sort score at the
    // "start_index" position, where alpha-beta will pick the next move.
    pub fn pick_move(ml: &mut MoveList, current_index: u8) {
        let mut best_index = current_index;

        for i in (current_index + 1)..ml.len() {
            if ml.get_move(i).get_sort_score() > ml.get_move(best_index).get_sort_score() {
                best_index = i;
            }
        }

        ml.swap(current_index as usize, best_index as usize);
    }
}
