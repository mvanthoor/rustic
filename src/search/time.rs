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

const OVERHEAD: i128 = 50; // msecs
const GAME_LENGTH: usize = 25; // moves
const MOVES_BUFFER: usize = 5; //moves
const CRITICAL_TIME: u128 = 1_000; // msecs
const OK_TIME: u128 = CRITICAL_TIME * 5; // msecs

impl Search {
    // Determine if allocated search time has been used up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.timer_elapsed();
        let allocated = refs.search_info.allocated_time;

        // Calculate a factor with which it is allowed to overshoot the
        // allocated search time. The more time the engine has, the larger
        // the overshoot-factor can be.
        let overshoot_factor = match allocated {
            x if x > OK_TIME => 2.0,                       // Allow large overshoot.
            x if x > CRITICAL_TIME && x <= OK_TIME => 1.5, // Low on time. Reduce overshoot.
            x if x <= CRITICAL_TIME => 1.0,                // Critical time. Don't overshoot.
            _ => 1.0,                                      // This case shouldn't happen.
        };

        elapsed >= (overshoot_factor * allocated as f64).round() as u128
    }

    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        // Calculate the time slice step by step.
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs);
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let increment = if white { gt.winc } else { gt.binc } as i128;
        let base_time = ((clock as f64) / (mtg as f64)).round() as i128;
        let time_slice = base_time + increment - OVERHEAD;

        // Make sure we're never sending less than 0 msecs of available time.
        if time_slice > 0 {
            // Just send the calculated slice.
            time_slice as u128
        } else if (base_time + increment) > (OVERHEAD / 5) {
            // Don't substract GUI lag protection (overhead) if this leads
            // to a negative time allocation.
            (base_time + increment) as u128
        } else {
            // We actually don't have any time.
            0
        }
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            // Guess moves to go if not supplied.
            let white = refs.board.us() == Sides::WHITE;
            let ply = refs.board.history.len();
            let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };

            GAME_LENGTH - (moves_made % GAME_LENGTH) + MOVES_BUFFER
        }
    }
}
