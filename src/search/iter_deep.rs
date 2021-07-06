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
    defs::{SearchRefs, SearchResult, INF},
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
        let is_game_time = refs.search_params.is_game_time();

        // Determine available time in case of GameTime search mode.
        if is_game_time {
            refs.search_info.allocated_time = Search::calculate_time_slice(refs);
        }

        // Set the starting values for alpha and beta, for use with the
        // aspiration window. We always start with a fully open window.
        let alpha: i16 = -INF;
        let beta: i16 = INF;

        // Start the search
        refs.search_info.timer_start();
        while (depth <= MAX_PLY)
            && (depth <= refs.search_params.depth)
            && !refs.search_info.interrupted()
        {
            // Set the current depth
            refs.search_info.depth = depth;

            // Get the evaluation for this depth.
            let eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);

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

                // Search one ply deeper.
                depth += 1;
            }
        }

        // Search is done. Report best move and reason to terminate.
        (best_move, refs.search_info.terminate)
    }
}
