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

use super::{
    defs::{
        SearchControl, SearchCurrentMove, SearchMode, SearchRefs, SearchReport, SearchStats,
        SearchTerminated, MAX_KILLER_MOVES, MIN_TIME_CURR_MOVE, MIN_TIME_STATS,
    },
    Search,
};
use crate::{
    engine::defs::{ErrFatal, Information},
    movegen::defs::Move,
};

impl Search {
    // This function calculates the number of nodes per second.
    pub fn nodes_per_second(nodes: usize, msecs: u128) -> usize {
        let mut nps: usize = 0;
        let seconds = msecs as f64 / 1000f64;
        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }
        nps
    }

    // Send intermediate statistics to GUI.
    pub fn send_stats_to_gui(refs: &mut SearchRefs) {
        let elapsed = refs.search_info.timer_elapsed();
        let last_stats = refs.search_info.last_stats_sent;

        if elapsed >= last_stats + MIN_TIME_STATS {
            let hash_full = refs.tt.lock().expect(ErrFatal::LOCK).hash_full();
            let msecs = refs.search_info.timer_elapsed();
            let nps = Search::nodes_per_second(refs.search_info.nodes, msecs);
            let stats = SearchStats::new(msecs, refs.search_info.nodes, nps, hash_full);
            let stats_report = SearchReport::SearchStats(stats);
            let information = Information::Search(stats_report);

            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
            refs.search_info.last_stats_sent = elapsed;
        }
    }

    // Send currently processed move to GUI.
    pub fn send_move_to_gui(refs: &mut SearchRefs, current_move: Move, nr: u8, total: u8) {
        let elapsed = refs.search_info.timer_elapsed();
        let lcm = refs.search_info.last_curr_move_sent;

        if elapsed >= lcm + MIN_TIME_CURR_MOVE {
            let scm = SearchCurrentMove::new(current_move, nr, total);
            let scm_report = SearchReport::SearchCurrentMove(scm);
            let information = Information::Search(scm_report);

            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
            refs.search_info.last_curr_move_sent = elapsed;
        }
    }

    // This function checks termination conditions and sets the termination
    // flag if this is required.
    pub fn check_termination(refs: &mut SearchRefs) {
        // Terminate search if stop or quit command is received.
        let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        match cmd {
            SearchControl::Stop => refs.search_info.terminate = SearchTerminated::Stopped,
            SearchControl::Abandon => refs.search_info.terminate = SearchTerminated::Abandoned,
            SearchControl::Quit => refs.search_info.terminate = SearchTerminated::Quit,
            SearchControl::Start(_) | SearchControl::Nothing => (),
        };

        // Terminate search if certain conditions are met. Only check this
        // if the search was not terminated by a direct command.
        if refs.search_info.terminate == SearchTerminated::Nothing {
            let search_mode = refs.search_params.search_mode;
            match search_mode {
                SearchMode::MoveTime => {
                    let elapsed = refs.search_info.timer_elapsed();
                    if elapsed >= refs.search_params.move_time {
                        refs.search_info.terminate = SearchTerminated::Stopped
                    }
                }
                SearchMode::Nodes => {
                    if refs.search_info.nodes >= refs.search_params.nodes {
                        refs.search_info.terminate = SearchTerminated::Stopped
                    }
                }
                SearchMode::GameTime => {
                    if Search::out_of_time(refs) {
                        refs.search_info.terminate = SearchTerminated::Stopped
                    }
                }
                // Depth is handled by iterative deepening. Infinite is
                // handled by a direct "stop" or "abandon" command.
                SearchMode::Depth | SearchMode::Infinite | SearchMode::Nothing => (),
            }
        }
    }
}

// Killer moves and history heuristics.
impl Search {
    // This function stores a move in the list of killer moves. Normally we
    // store two killer moves per ply. By checking that the move we want to
    // store is not the same as the first killer move in the list, we make sure
    // that both moves are always different. It is possible to store three or
    // more killer moves, but experience shows that checking for ALL of them to
    // be unique costs more time than the extra killer moves could save.
    pub fn store_killer_move(current_move: Move, refs: &mut SearchRefs) {
        const FIRST: usize = 0;
        let ply = refs.search_info.ply as usize;
        let first_killer = refs.search_info.killer_moves[ply][FIRST];

        // First killer must not be the same as the move being stored.
        if first_killer.get_move() != current_move.get_move() {
            // Shift all the moves one index upward...
            for i in (1..MAX_KILLER_MOVES).rev() {
                let n = i as usize;
                let previous = refs.search_info.killer_moves[ply][n - 1];
                refs.search_info.killer_moves[ply][n] = previous;
            }

            // and add the new killer move in the first spot.
            refs.search_info.killer_moves[ply][0] = current_move.to_short_move();
        }
    }
}
