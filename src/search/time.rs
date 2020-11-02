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

impl Search {
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        // Use this as the basis for game length if "movestogo" is
        // not supplied (base time is for entire game).
        const DEFAULT_GAME_LENGTH: usize = 80;

        // Substract some time for communication to the GUI, so the
        // engine won't overshoot its time.
        const COMM_OVERHEAD: u128 = 100;

        // Shorthand for game_time.
        let gt = &refs.search_params.game_time;

        // Get which side to search for.
        let white = refs.search_params.search_side == Sides::WHITE;

        // Initialize time variables and moves to go.
        let (time, inc, moves_to_go): (u128, u128, usize) = if white {
            let mtg = if let Some(x) = gt.moves_to_go {
                x
            } else {
                let moves_made = refs.board.history.len() / 2;
                DEFAULT_GAME_LENGTH - moves_made
            };

            (gt.wtime, gt.winc, mtg)
        } else {
            let mtg = if let Some(x) = gt.moves_to_go {
                x
            } else {
                let moves_made = (refs.board.history.len() - 1) / 2;
                DEFAULT_GAME_LENGTH - moves_made
            };

            (gt.btime, gt.binc, mtg)
        };

        // Calculate time per move. Stop if this time has elapsed
        // since the search began.
        let base = (time as f64 / moves_to_go as f64).round() as u128;
        let available = base + inc - COMM_OVERHEAD;
        let elapsed = refs.search_info.start_time.elapsed().as_millis();

        elapsed >= available
    }
}
