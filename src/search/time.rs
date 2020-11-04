/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2020, Marcel Vanthoor

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

use super::{defs::SearchRefs, Search};
use crate::defs::Sides;

const OVERHEAD: u128 = 100;
const GAME_LENGTH: usize = 60;
const MIN_MOVES_TO_GO: usize = 20;
const MIN_TIME: u128 = 1250;

impl Search {
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.timer_elapsed();
        let allotted = refs.search_info.time_for_move;

        elapsed >= allotted
    }

    pub fn time_for_move(refs: &SearchRefs) -> u128 {
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs);
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let increment = if white { gt.winc } else { gt.binc };
        let base_time = if clock > MIN_TIME || (increment == 0) {
            ((clock as f64 * 0.8) / (mtg as f64)).round() as u128
        } else {
            0
        };

        base_time + increment - OVERHEAD
    }

    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            // Guess moves to go if not supplied.
            let white = refs.board.us() == Sides::WHITE;
            let ply = refs.board.history.len();
            let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };
            if moves_made < (GAME_LENGTH - MIN_MOVES_TO_GO) {
                GAME_LENGTH - moves_made
            } else {
                MIN_MOVES_TO_GO
            }
        }
    }
}
