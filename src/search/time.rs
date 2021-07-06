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

use super::{defs::SearchRefs, Search};
use crate::defs::Sides;

pub const OVERHEAD: f64 = 20.0; // msecs
const GAME_LENGTH: usize = 30; // moves
const MAX_USAGE: f64 = 0.8; // percentage

impl Search {
    // Determine if allocated search time has been used up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        refs.search_info.timer_elapsed() > refs.search_info.allocated_time
    }

    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        // Calculate the time slice step by step.
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs) as f64;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime } as f64;
        let increment = if white { gt.winc } else { gt.binc } as f64;
        let base_time = ((clock - OVERHEAD) * MAX_USAGE / mtg).round();
        let mut time_slice = base_time + increment;
        time_slice = if time_slice <= 0.0 { 1.0 } else { time_slice };

        time_slice as u128
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            GAME_LENGTH
        }
    }
}
