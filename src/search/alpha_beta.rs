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
    defs::{SearchTerminate, CHECKMATE, DRAW, STALEMATE, UPDATE_STATS},
    Search, SearchRefs,
};
use crate::{
    defs::MAX_MOVE_RULE,
    movegen::defs::{Move, MoveList, MoveType},
};

impl Search {
    pub fn alpha_beta(depth: u8, mut alpha: i16, beta: i16, refs: &mut SearchRefs) -> i16 {
        // Check if termination condition is met.
        if Search::is_checkpoint(refs) {
            Search::check_for_termination(refs);
        }

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth == 0 {
            return Search::quiescence(alpha, beta, refs);
        }

        // Return a draw score if this position is a repetition, or if the
        // number of moves without captures or pawn moves was exceeded.
        let max_move_rule = refs.board.game_state.halfmove_clock >= MAX_MOVE_RULE;
        if Search::is_repetition(refs.board) || max_move_rule {
            return DRAW;
        }

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

        // We created a new node which we'll search, so count it.
        refs.search_info.nodes += 1;

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
                Search::send_updated_current_move(refs, current_move, legal_moves_found);
            }

            // Send search stats to the engine, every time the node count
            // has counted UPDATE_STATS number of nodes. These are stats
            // such as nodes, speed, TT full, etc.
            if refs.search_info.nodes >= refs.search_info.last_stats + UPDATE_STATS {
                Search::send_updated_stats(refs);
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
        alpha
    }
}
