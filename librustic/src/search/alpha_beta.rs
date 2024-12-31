use crate::{
    board::defs::Pieces,
    defs::MAX_PLY,
    engine::defs::{ErrFatal, Verbosity},
    evaluation::Evaluation,
    movegen::defs::{Move, MoveList, MoveType, ShortMove},
    search::{
        defs::{
            PrincipalVariation, SearchTerminated, CHECKMATE, CHECK_TERMINATION, DRAW, INF,
            SEND_STATS, STALEMATE,
        },
        transposition::{HashFlag, SearchData},
        Search, SearchRefs,
    },
};

impl Search {
    pub fn alpha_beta(
        mut depth: i8,
        mut alpha: i16,
        beta: i16,
        pv: &mut PrincipalVariation,
        refs: &mut SearchRefs,
    ) -> i16 {
        let verbosity = refs.search_params.verbosity; // If quiet, don't send intermediate stats.
        let is_root = refs.search_info.ply == 0; // At root if no moves were played.
        let mut found_pv_move = false; // Used for PVS (Principal Variation Search)

        // Check if termination condition is met.
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        // If time is up, abort. This depth won't be considered in
        // iterative deepening as it is unfinished.
        if refs.search_info.terminate != SearchTerminated::Nothing {
            return 0;
        }

        // Stop going deeper if we hit MAX_PLY.
        if refs.search_info.ply >= MAX_PLY {
            return Evaluation::evaluate_position(refs.board);
        }

        // Determine if we are in check.
        let is_check = refs.mg.square_attacked(
            refs.board,
            refs.board.opponent(),
            refs.board.king_square(refs.board.us()),
        );

        // If so, extend search depth by 1 to determine the best way to get
        // out of the check before we go into quiescence search.
        if is_check {
            depth += 1;
        }

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth <= 0 {
            return Search::quiescence(alpha, beta, pv, refs);
        }

        // Count this node, as it is not aborted or searched by QSearch.
        refs.search_info.nodes += 1;

        // Variables to hold TT value and move if any.
        let mut tt_value: Option<i16> = None;
        let mut tt_move: ShortMove = ShortMove::new(0);

        // Probe the TT for information.
        if let Some(data) = refs
            .tt
            .lock()
            .expect(ErrFatal::LOCK)
            .probe(refs.board.game_state.zobrist_key)
        {
            let tt_result = data.get(depth, refs.search_info.ply, alpha, beta);
            tt_value = tt_result.0;
            tt_move = tt_result.1;
        }

        // If we have a value from the TT then return immediately. Don't do
        // a hash cut when in the root position, because we may end up
        // without actually having a move to play.
        if !is_root {
            if let Some(v) = tt_value {
                return v;
            }
        }

        /*=== Actual searching starts here ===*/

        // Generate the moves in this position
        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg
            .generate_moves(refs.board, &mut move_list, MoveType::All);

        // Do move scoring, so the best move will be searched first.
        Search::score_moves(&mut move_list, tt_move, refs);

        // After SEND_STATS nodes have been searched, check if the
        // MIN_TIME_STATS has been exceeded; if so, send the current
        // statistics to the GUI.
        if verbosity == Verbosity::Full && (refs.search_info.nodes & SEND_STATS == 0) {
            Search::send_stats_to_gui(refs);
        }

        // Set the initial best score (to the worst possible value)
        let mut best_score = -INF;

        // Set the initial TT flag type. Assume we do not beat Alpha.
        let mut hash_flag = HashFlag::Alpha;

        // Holds the best move in the move loop for storing into the TT.
        let mut best_move = Move::new(0);

        // Iterate over all pseudo-legal moves, trying to find the best
        // move in the current position..
        for i in 0..move_list.len() {
            // This function picks the move with the highest likelyhood to
            // be a good move and it puts it at the current index of the
            // move list, so get_move() will get this. This makes sure that
            // moves who are considered the best candidates for being good
            // moves are tried first.
            Search::pick_move(&mut move_list, i);

            // Get the move which was just picked and test if it is a legal
            // move. (This test is required because the list contains all
            // possible moves in the position, which are pseudo-legal. A
            // move could leave the king in check and we must test for
            // this.)
            let current_move = move_list.get_move(i);
            let is_legal = refs.board.make(current_move, refs.mg);

            // If the move turns out to be not legal, we skip it.
            if !is_legal {
                continue;
            }

            // We found a legal move.
            legal_moves_found += 1;
            refs.search_info.ply += 1;

            // Update seldepth if searching deeper than specified depth.
            if refs.search_info.ply > refs.search_info.seldepth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Send currently searched move to GUI.
            if verbosity == Verbosity::Full && is_root {
                Search::send_move_to_gui(refs, current_move, legal_moves_found, move_list.len());
            }

            // Create a node PV for this move.
            let mut node_pv = PrincipalVariation::new();

            // We just made a move. We are not yet at one of the leaf
            // nodes, so if the position is not a draw, we must search
            // deeper. Initially, assume the position is a draw.
            let mut score = DRAW;

            // If it isn't a draw, we must search.
            if !refs.board.is_draw() {
                // The previous move in our move list was a PV move.
                // Because of this, we can now search the current and
                // upcoming moves in the list with a zero-width window.
                // This will make searching very fast, because we are just
                // trying to prove that the move will not improve alpha and
                // thus doesn't have to be searched further.
                if found_pv_move {
                    // This is the zero-width search.
                    score = -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);

                    // If the zero-width *did* improve alpha and stayed
                    // below beta, we found a new, better PV move.
                    // Therefore we failed the zero-width search and have
                    // to search again with the normal alpha-beta window.
                    if (score > alpha) && (score < beta) {
                        score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            }

            // Take back the move, and decrease ply accordingly.
            refs.board.unmake();
            refs.search_info.ply -= 1;

            // If we improved our best score, we save it, and the current
            // move that goes along with it. Then we check if the score
            // beats beta or alpha.
            if score > best_score {
                best_score = score;
                best_move = current_move;

                // A move that is so strong for our opponent that we want to
                // avoid it at all cost, is called a beta cutoff. We don't
                // search along that path and abort ("cut") the search.
                if score >= beta {
                    hash_flag = HashFlag::Beta;

                    // If the current move is not a capture but still causes a
                    // beta-cutoff, then store it as a killer.
                    if current_move.captured() == Pieces::NONE {
                        Search::store_killer_move(current_move, refs);
                    }

                    // Perform the cutoff. Break the move loop. Many
                    // implementations of alpha-beta just return beta here
                    // (fail-hard), or best_score (fail-soft). This is fine,
                    // but we *definitely* want to save this score and move in
                    // the TT. This would require us to have the transposition
                    // table store code duplicated right here where this
                    // comment is. Therefore I prefer to save the current
                    // values we need to store as best_score, best_move, and
                    // hash_flag and break the loop instead of returning. We
                    // store the values in the TT when finishing up the
                    // alpha-beta run after the move loop.
                    break;
                }

                // We found a better move for us: the score is higher than
                // Alpha, but NOT equal or higher than beta.
                if score > alpha {
                    // This is an exact move score. It's a PV move, with a
                    // score between alpha and beta.
                    hash_flag = HashFlag::Exact;

                    // Save our better evaluation score as the new alpha.
                    alpha = score;

                    // Update the Principal Variation. These are moves that
                    // improved alpha but did not cause a beta-cutoff.
                    found_pv_move = true;
                    pv.clear();
                    pv.push(current_move);
                    pv.append(&mut node_pv);
                }
            }
        }

        // === This is where we end up if we either finish the move loop,
        // or we've broken it because of a beta-cutoff. We now have our
        // best_score, which is either alpha (if we finished the move-loop)
        // or beta (if we have a beta-cutoff). We also have the best_move,
        // which was responsible for our best_score. === //

        // === Finish up the alpha-beta run below === //

        // If we exit the loop without legal moves being found, the
        // side to move is either in checkmate or stalemate.
        if legal_moves_found == 0 {
            if is_check {
                // The return value is minus CHECKMATE, because if we have
                // no legal moves and are in check, it's game over.
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        // We save the best score and move (if any) in the TT.
        refs.tt.lock().expect(ErrFatal::LOCK).insert(
            refs.board.game_state.zobrist_key,
            SearchData::create(
                depth,
                refs.search_info.ply,
                hash_flag,
                best_score,
                best_move.to_short_move(),
            ),
        );

        // We traversed the entire move list or broken it due to a beta
        // cutoff. We return the best score of the run, which is thus
        // either alpha if we finished the list, or beta if we've broken
        // the loop.
        best_score
    }
}
