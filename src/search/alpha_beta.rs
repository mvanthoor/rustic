use super::{
    defs::{SearchControl, SearchTerminate, CHECKMATE, CHECKPOINT, STALEMATE},
    Search, SearchRefs,
};
use crate::{
    evaluation,
    movegen::defs::{Move, MoveList, MoveType},
};

impl Search {
    pub fn alpha_beta(depth: u8, mut alpha: i16, beta: i16, refs: &mut SearchRefs) -> i16 {
        // Check for stop or quit commands.
        // ======================================================================

        let checkpoint = refs.search_info.nodes % CHECKPOINT == 0;

        if checkpoint {
            let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
            match cmd {
                SearchControl::Stop => refs.search_info.termination = SearchTerminate::Stop,
                SearchControl::Quit => refs.search_info.termination = SearchTerminate::Quit,
                _ => (),
            };
        }

        // ======================================================================

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth == 0 {
            return evaluation::evaluate_position(refs.board);
        }

        // Temporary variables.
        let mut current_best_move = Move::new(0);
        let old_alpha = alpha;

        // Search a new node, so we increase the node counter.
        refs.search_info.nodes += 1;

        // Generate the moves in this position
        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg
            .generate_moves(refs.board, &mut move_list, MoveType::All);

        // Iterate over the moves.
        for i in 0..move_list.len() {
            if refs.search_info.termination != SearchTerminate::Nothing {
                break;
            }

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
                current_best_move = current_move;
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
            refs.search_info.best_move = current_best_move;
        }

        // We have traversed the entire move list and found the best
        // possible move/eval_score for us at this depth. We can't improve
        // this any further, so return the result. This called "fail-low".
        return alpha;
    }
}
