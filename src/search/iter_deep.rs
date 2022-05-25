use super::{
    defs::{PrincipalVariation, SearchMode, SearchRefs, SearchResult, INF},
    ErrFatal, Information, Search, SearchReport, SearchSummary,
};
use crate::{defs::MAX_PLY, engine::defs::Verbosity, movegen::defs::Move};

// Actual search routines.
impl Search {
    pub fn iterative_deepening(refs: &mut SearchRefs) -> SearchResult {
        // Working variables
        let mut depth = 1;
        let mut best_move = Move::new(0);
        let mut root_pv = PrincipalVariation::new();
        let is_game_time = refs.search_params.is_game_time();

        // Determine available time in case of GameTime search mode.
        if is_game_time {
            let time_slice = Search::calculate_time_slice(refs);

            // We have some time to spend.
            if time_slice > 0 {
                refs.search_info.allocated_time = time_slice;
            } else {
                // Base time is under time-out safeguard, and we have no
                // increment. We do a last-ditch effort by calculating a
                // move at one ply deep.
                refs.search_params.search_mode = SearchMode::Depth;
                refs.search_params.depth = 1;
            }
        }

        // Set the starting values for alpha and beta.
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
                best_move = root_pv[0];

                if refs.search_params.verbosity != Verbosity::Silent {
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
                }

                // Search one ply deeper.
                depth += 1;
            }
        }

        // Search is done. Report best move and reason to terminate.
        (best_move, refs.search_info.terminate)
    }
}
