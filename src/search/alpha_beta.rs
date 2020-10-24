use super::{
    defs::{
        SearchControl, SearchCurrentMove, SearchStats, SearchTerminate, CHECKMATE, CHECKPOINT,
        STALEMATE, UPDATE_STATS,
    },
    Search, SearchRefs, SearchReport,
};
use crate::{
    engine::defs::{ErrFatal, Information},
    evaluation,
    movegen::defs::{Move, MoveList, MoveType},
};

impl Search {
    pub fn alpha_beta(depth: u8, mut alpha: i16, beta: i16, refs: &mut SearchRefs) -> i16 {
        // Check for stop or quit commands.
        // ======================================================================

        let checkpoint = refs.search_info.nodes % CHECKPOINT == 0;
        if checkpoint {
            // Terminate search if stop or quit command is received.
            let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
            match cmd {
                SearchControl::Stop => refs.search_info.terminate = SearchTerminate::Stop,
                SearchControl::Quit => refs.search_info.terminate = SearchTerminate::Quit,
                _ => (),
            };

            // Terminate search if allowed time for this move has run out.
            let elapsed = refs.search_info.start_time.elapsed().as_millis();
            let time_up = elapsed >= refs.search_params.time_for_move;
            if time_up {
                refs.search_info.terminate = SearchTerminate::Stop
            }
        }

        // ======================================================================

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth == 0 {
            return evaluation::evaluate_position(refs.board);
        }

        // Search a new node, so we increase the node counter.
        refs.search_info.nodes += 1;

        // Temporary variables.
        let mut best_move_at_depth = Move::new(0);
        let old_alpha = alpha;

        // Generate the moves in this position
        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg
            .generate_moves(refs.board, &mut move_list, MoveType::All);

        // Do move scoring, so the best move will be searched first.
        Search::score_moves(&mut move_list);

        // Iterate over the moves.
        for i in 0..move_list.len() {
            if refs.search_info.terminate != SearchTerminate::Nothing {
                break;
            }

            // This function finds the best move to test according to the
            // move scoring, and puts it at the current index of the move
            // list, so get_move() will get this next.
            Search::pick_move(&mut move_list, i);

            let current_move = move_list.get_move(i);
            let is_legal = refs.board.make(current_move, refs.mg);

            // If not legal, skip the move and the rest of the function.
            if !is_legal {
                continue;
            }

            // At this point, a legal move was found.
            legal_moves_found += 1;

            // Move is legal; increase the ply count.
            refs.search_info.ply += 1;

            // Send currently researched move to engine thread. Send this
            // only when we are at the root of the tree.
            if Search::is_root(refs.search_info.depth, depth) {
                let scm = SearchCurrentMove::new(current_move, legal_moves_found);
                let scm_report = SearchReport::SearchCurrentMove(scm);
                let information = Information::Search(scm_report);
                refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
            }

            // Send search stats to the engine, once per second. These are
            // technical stats such as nodes, speed, TT full, etc.
            if refs.search_info.nodes >= refs.search_info.last_stats + UPDATE_STATS {
                let milli_seconds = refs.search_info.start_time.elapsed().as_millis();
                let nps = Search::nodes_per_second(refs.search_info.nodes, milli_seconds);
                let stats = SearchStats::new(refs.search_info.nodes, nps);
                let stats_report = SearchReport::SearchStats(stats);
                let information = Information::Search(stats_report);
                refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
                refs.search_info.last_stats = refs.search_info.nodes;
            }

            // We are not yet in a leaf node (the "bottom" of the tree, at
            // the requested depth), so start Alpha-Beta again, for the
            // opponent's side to go one ply deeper.
            let eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, refs);

            // Take back the move, and decrease ply accordingly.
            refs.board.unmake();
            refs.search_info.ply -= 1;

            // Beta-cut-off. We return this score, because searching any
            // further down this path would make the situation worse for us
            // and better for our opponent. This is called "fail-high".
            if eval_score >= beta {
                return beta;
            }

            // We found a better move for us.
            if eval_score > alpha {
                // Save our better evaluation score.
                alpha = eval_score;
                best_move_at_depth = current_move;
            }
        }

        // If we exit the loop without legal moves being found, the
        // side to move is either in checkmate or stalemate.
        if legal_moves_found == 0 {
            let king_square = refs.board.king_square(refs.board.us());
            let opponent = refs.board.opponent();
            let check = refs.mg.square_attacked(refs.board, opponent, king_square);

            if check {
                // The return value is minus CHECKMATE (negative), because
                // if we have no legal moves AND are in check, we have
                // lost. This is a very negative outcome.
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        // Alpha was improved while walking through the move list, so a
        // better move was found.
        if alpha != old_alpha {
            refs.search_info.bm_at_depth = best_move_at_depth;
        }

        // We have traversed the entire move list and found the best
        // possible move/eval_score for us at this depth. We can't improve
        // this any further, so return the result. This called "fail-low".
        return alpha;
    }
}
