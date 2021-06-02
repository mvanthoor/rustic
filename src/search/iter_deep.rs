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
    defs::{SearchMode, SearchRefs, SearchResult, ASPIRATION_WINDOW, INF},
    ErrFatal, Information, Search, SearchReport, SearchSummary,
};
use crate::{defs::MAX_PLY, movegen::defs::Move};

// Actual search routines.
impl Search {
    pub fn iterative_deepening(refs: &mut SearchRefs) -> SearchResult {
        // Working variables
        let mut depth = 1;
        let mut best_move = Move::new(0);
        let mut root_pv: Vec<Move> = Vec::new();
        let mut stop = false;
        let is_game_time = refs.search_params.is_game_time();

        // Determine available time in case of GameTime search mode.
        if is_game_time {
            // Determine the maximum time slice available for this move.
            let time_slice = Search::calculate_time_slice(refs);

            // Experience reveals that after using about 40-50% of the
            // available time, the next depth will not be finished, so
            // don't allocated more than 40% of the calculated move time.
            let factor = 0.40;

            // If we have time, do a normal search in GameTime mode.
            if time_slice > 0 {
                // Determine the actual time to allot for this search.
                refs.search_info.allocated_time = (time_slice as f64 * factor).round() as u128;
            } else {
                // We have no time. Send the best move from ply 1 to avoid
                // killing ourselves by sending no move at all. Change mode
                // to "depth" and set it to 1 ply.
                refs.search_params.search_mode = SearchMode::Depth;
                refs.search_params.depth = 1;
            }
        }

        // Set the starting values for alpha and beta, for use with the
        // aspiration window. We always start with a fully open window.
        let mut alpha: i16 = -INF;
        let mut beta: i16 = INF;

        // Start the search
        refs.search_info.timer_start();
        while (depth <= MAX_PLY) && (depth <= refs.search_params.depth) && !stop {
            // Set the current depth
            refs.search_info.depth = depth;

            // Get the evaluation for this depth.
            let eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);

            // If the evaluation result falls outside the alpha-beta
            // window, then re-open the window. Then "continue" the loop.
            // This skips the rest of the loop and starts the search again
            // at the same depth.
            if eval <= alpha || eval >= beta {
                alpha = -INF;
                beta = INF;
                continue;
            }

            // We didn't fall outside the aspiration window, so we make it
            // smaller to search faster on the next iteration.
            alpha = eval - ASPIRATION_WINDOW;
            beta = eval + ASPIRATION_WINDOW;

            // Create summary if search was not interrupted.
            if !refs.search_info.interrupted() {
                // Save the best move until now.
                if !root_pv.is_empty() {
                    best_move = root_pv[0];
                }

                // Create search summary for this depth.
                let elapsed = refs.search_info.timer_elapsed();
                let nodes = refs.search_info.nodes;
                let hash_full = refs.tt.lock().expect(ErrFatal::LOCK).hash_full();
                let summary = SearchSummary {
                    depth,
                    seldepth: refs.search_info.seldepth,
                    time: elapsed,
                    cp: eval,
                    mate: 0,
                    nodes,
                    nps: Search::nodes_per_second(nodes, elapsed),
                    hash_full,
                    pv: root_pv.clone(),
                };

                // Create information for the engine
                let report = SearchReport::SearchSummary(summary);
                let information = Information::Search(report);
                refs.report_tx.send(information).expect(ErrFatal::CHANNEL);

                // Search one ply deepr.
                depth += 1;
            }

            // Determine if time is up, when in GameTime mode.
            let time_up = if is_game_time {
                refs.search_info.timer_elapsed() > refs.search_info.allocated_time
            } else {
                false
            };

            // Stop deepening the search if the current depth was
            // interrupted, or if the time is up.
            stop = refs.search_info.interrupted() || time_up;
        }

        // Search is done. Report best move and reason to terminate.
        (best_move, refs.search_info.terminate)
    }
}
