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
const GAME_LENGTH: usize = 50;
const MIN_MOVES_TO_GO: usize = 10;
const SPEED_MOVES_TO_GO: usize = 20;
const MIN_TIME: u128 = 1_000;
const OK_TIME: u128 = MIN_TIME * 5;

impl Search {
    // This function just returns true if the search time for the currently
    // searched move is up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.timer_elapsed();
        let allotted = refs.search_info.time_for_move;

        // Calculate a factor with which it is allowed to overshoot the
        // allocated search time. The more time the engine has, the more it
        // is allowed to use more time for the move, beyond to what was
        // initially allocated.
        let factor = match allotted {
            x if x > OK_TIME => 2.0,                  // Allow large overshoot.
            x if x > MIN_TIME && x <= OK_TIME => 1.5, // Low on time. Reduce overshoot.
            x if x <= MIN_TIME => 1.0,                // Critical time. Don't overshoot.
            _ => 1.0,                                 // General case should never happen.
        };

        elapsed >= (factor * allotted as f64).round() as u128
    }

    // This function calculates the time the engine allocates for searching
    // a single move. This depends on the number of moves still to go in
    // the game.
    pub fn time_for_move(refs: &SearchRefs) -> u128 {
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs);
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let increment = if white { gt.winc } else { gt.binc };
        let base_time = if clock > MIN_TIME || (increment == 0) {
            ((clock as f64) / (mtg as f64)).round() as u128
        } else {
            0
        };

        base_time + increment - OVERHEAD
    }

    // This function tries to come up with some sort of sensible value for
    // "moves to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            // Guess moves to go if not supplied.
            let white = refs.board.us() == Sides::WHITE;
            let ply = refs.board.history.len();
            let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };
            let early = GAME_LENGTH - MIN_MOVES_TO_GO;
            let late = GAME_LENGTH + MIN_MOVES_TO_GO;

            // If in the "early" stage, report 'actual' moves to go, using
            // GAME_LENGTH as a guess. If the number of moves made are
            // between GAME_lENGTH and the late stage of the game, report
            // MIN_MOVES_TO_GO, so the engine uses the same base time for
            // each move. If the game takes very long, start reporting
            // SPEED_MOVES_TO_GO all the time to allocate even less time
            // per move.
            match moves_made {
                x if x <= early => GAME_LENGTH - moves_made,
                x if x > early && x <= late => MIN_MOVES_TO_GO,
                x if x > late => SPEED_MOVES_TO_GO,
                _ => SPEED_MOVES_TO_GO,
            }
        }
    }
}
