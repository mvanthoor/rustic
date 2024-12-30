use crate::{
    defs::MAX_PLY,
    engine::defs::Verbosity,
    evaluation::Evaluation,
    movegen::defs::{MoveList, MoveType, ShortMove},
    search::{
        defs::{PrincipalVariation, SearchTerminated, CHECK_TERMINATION, SEND_STATS},
        Search, SearchRefs,
    },
};

impl Search {
    pub fn quiescence(
        mut alpha: i16,
        beta: i16,
        pv: &mut PrincipalVariation,
        refs: &mut SearchRefs,
    ) -> i16 {
        // We created a new node which we'll search, so count it.
        refs.search_info.nodes += 1;

        // No intermediate stats updates if quiet.
        let verbosity = refs.search_params.verbosity;

        // Check if search needs to be terminated.
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        // Abort if we have to terminate. Depth not finished.
        if refs.search_info.terminate != SearchTerminated::Nothing {
            return 0;
        }

        // Immediately evaluate and return on reaching MAX_PLY
        if refs.search_info.ply >= MAX_PLY {
            return Evaluation::evaluate_position(refs.board);
        }

        // Do a stand-pat here: Check how we're doing, even before we make
        // a move. If the evaluation score is larger than beta, then we're
        // already so bad we don't need to search any further. Just return
        // the beta score.
        let eval_score = Evaluation::evaluate_position(refs.board);
        if eval_score >= beta {
            return beta;
        }

        // If the evaluation score is bigger than alpha, then we can
        // improve our position. So set alpha to this score and keep
        // searching until there are no more captures.
        if eval_score > alpha {
            alpha = eval_score
        }

        // Stand-pat is done. Start searching the captures in our position.
        // This is basically the same as alpha/beta, but without depth. We
        // simply keep searching until the stand-pat above breaks us out of
        // the recursion, or until there are no more captures available.
        // Then the function will return after looping the move list.

        // Generate only capture moves.
        let mut move_list = MoveList::new();
        refs.mg
            .generate_moves(refs.board, &mut move_list, MoveType::Capture);

        // Do move scoring, so the best move will be searched first.
        Search::score_moves(&mut move_list, ShortMove::new(0), refs);

        // Update search stats in the GUI. Check every SEND_STATS nodes if
        // the minimum MIN_TIME_STATS has elapsed before sending.
        if verbosity == Verbosity::Full && (refs.search_info.nodes & SEND_STATS == 0) {
            Search::send_stats_to_gui(refs);
        }

        // Iterate over the capture moves.
        for i in 0..move_list.len() {
            // Pick the next moves with the highest score.
            Search::pick_move(&mut move_list, i);

            let current_move = move_list.get_move(i);
            let is_legal = refs.board.make(current_move, refs.mg);

            // If not legal, skip the move and the rest of the function.
            if !is_legal {
                continue;
            }

            // Move is legal; increase the ply count.
            refs.search_info.ply += 1;

            // Update seldepth if we're searching deeper than requested.
            if refs.search_info.ply > refs.search_info.seldepth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Create a PV for this node.
            let mut node_pv = PrincipalVariation::new();

            // The position is not yet quiet. Go one ply deeper.
            let eval_score = -Search::quiescence(-beta, -alpha, &mut node_pv, refs);

            // Take back the move, and decrease ply accordingly.
            refs.board.unmake();
            refs.search_info.ply -= 1;

            // If we are worse than beta (the opponent), then stop
            // searching, because we can't improve anymore.
            if eval_score >= beta {
                return beta;
            }

            // We found a better move for us.
            if eval_score > alpha {
                // Save our better evaluation score.
                alpha = eval_score;

                // Update the Principal Variation.
                pv.clear();
                pv.push(current_move);
                pv.append(&mut node_pv);
            }
        }

        // We have traversed the entire move list and found the best score for us,
        // so we return this.
        alpha
    }
}
