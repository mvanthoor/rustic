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

use super::{
    defs::{SearchRefs, SearchResult, SearchTerminate, INF},
    ErrFatal, Information, Search, SearchReport, SearchSummary,
};
use crate::{defs::MAX_DEPTH, movegen::defs::Move};

// Actual search routines.
impl Search {
    pub fn iterative_deepening(refs: &mut SearchRefs) -> SearchResult {
        // Working variables
        let mut depth = 1;
        let mut best_move = Move::new(0);
        let mut temp_pv: Vec<Move> = Vec::new();
        let mut stop = false;

        // Set up time management.
        let allotted = Search::time_for_move(refs);
        let max_time = (allotted as f64 * 0.35).round() as u128;
        refs.search_info.time_for_move = allotted;
        refs.search_info.timer_start();

        while (depth < MAX_DEPTH) && (depth <= refs.search_params.depth) && !stop {
            // Set the current depth
            refs.search_info.depth = depth;

            // Get the evaluation for this depth.
            let eval = Search::alpha_beta(depth, -INF, INF, &mut temp_pv, refs);

            // Create summary if search was not interrupted.
            let interrupted = refs.search_info.terminate != SearchTerminate::Nothing;
            if !interrupted {
                // Save the best move until now.
                best_move = refs.search_info.best_move;

                // Create search summary for this depth.
                let elapsed = refs.search_info.timer_elapsed();
                let nodes = refs.search_info.nodes;
                let summary = SearchSummary {
                    depth,
                    seldepth: refs.search_info.seldepth,
                    time: elapsed,
                    cp: eval,
                    mate: 0,
                    nodes,
                    nps: Search::nodes_per_second(nodes, elapsed),
                    pv: refs.search_info.pv.clone(),
                };

                // Create information for the engine
                let report = SearchReport::SearchSummary(summary);
                let information = Information::Search(report);
                refs.report_tx.send(information).expect(ErrFatal::CHANNEL);

                // Search one ply deepr.
                depth += 1;
            }

            // Determine if trying for another depth is feasible.
            let elapsed = refs.search_info.timer_elapsed();
            let abort = elapsed > max_time;
            stop = interrupted || abort;
        }

        // Search is done. Report best move and reason to terminate.
        (best_move, refs.search_info.terminate)
    }
}
