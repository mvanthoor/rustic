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
    defs::{
        SearchTerminate, CHECKMATE, CHECK_TERMINATION, DRAW, MIN_TIME_CURR_MOVE, MIN_TIME_STATS,
        SEND_STATS, STALEMATE,
    },
    Search, SearchRefs,
};
use crate::{
    defs::MAX_DEPTH,
    evaluation,
    movegen::defs::{Move, MoveList, MoveType},
};

impl Search {
    pub fn alpha_beta(
        mut depth: u8,
        mut alpha: i16,
        beta: i16,
        pv: &mut Vec<Move>,
        refs: &mut SearchRefs,
    ) -> i16 {
        // If quiet, don't send intermediate stats updates.
        let quiet = refs.search_params.quiet;

        // If we haven't made any moves yet, we're at the root.
        let is_root = refs.search_info.ply == 0;

        // Check if termination condition is met.
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        // We have arrived at the leaf node. Evaluate the position and
        // return the result.
        if depth == 0 {
            return Search::quiescence(alpha, beta, pv, refs);
        }

        // Stop going deeper if we hit MAX_DEPTH.
        if refs.search_info.ply >= MAX_DEPTH {
            return evaluation::evaluate_position(refs.board);
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

        /*=== Actual searching starts here ===*/

        // Temporary variables.
        let mut best_move = Move::new(0);

        // Generate the moves in this position
        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg
            .generate_moves(refs.board, &mut move_list, MoveType::All);

        // Do move scoring, so the best move will be searched first.
        Search::score_moves(&mut move_list);

        // We created a new node which we'll search, so count it.
        refs.search_info.nodes += 1;

        // After SEND_STATS nodes have been searched, check if the
        // MIN_TIME_STATS has been exceeded; if so, sne dthe current
        // statistics to the GUI.
        if !quiet && (refs.search_info.nodes & SEND_STATS == 0) {
            let elapsed = refs.search_info.timer_elapsed();
            let last_stats = refs.search_info.last_stats_sent;
            if elapsed >= last_stats + MIN_TIME_STATS {
                Search::send_stats(refs);
                refs.search_info.last_stats_sent = elapsed;
            }
        }

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

            // We found a legal move.
            legal_moves_found += 1;
            refs.search_info.ply += 1;

            // Update seldepth if searching deeper than specified depth.
            if refs.search_info.ply > refs.search_info.depth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Send currently searched move to GUI.
            if !quiet && is_root {
                let elapsed = refs.search_info.timer_elapsed();
                let lcm = refs.search_info.last_curr_move_sent;
                if elapsed >= lcm + MIN_TIME_CURR_MOVE {
                    Search::send_current_move(refs, current_move, legal_moves_found);
                    refs.search_info.last_curr_move_sent = elapsed;
                }
            }

            // Create a node PV for this move.
            let mut node_pv: Vec<Move> = Vec::new();

            //We just made a move. We are not yet at one of the leaf nodes,
            //so we must search deeper. We do this by calling alpha/beta
            //again to go to the next ply, but ONLY if this move is NOT
            //causing a draw by repetition or 50-move rule. If it is, we
            //don't have to search anymore: we can just assign DRAW as the
            //eval_score.
            let eval_score = if !Search::is_draw(refs) {
                -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs)
            } else {
                DRAW
            };

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
                best_move = current_move;

                pv.clear();
                pv.push(best_move);
                pv.append(&mut node_pv);
            }
        }

        // If we exit the loop without legal moves being found, the
        // side to move is either in checkmate or stalemate.
        if legal_moves_found == 0 {
            if is_check {
                // The return value is minus CHECKMATE (negative), because
                // if we have no legal moves AND are in check, we have
                // lost. This is a very negative outcome.
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        // We store our best move and best PV.
        refs.search_info.best_move = best_move;
        refs.search_info.pv = pv.clone();

        // We have traversed the entire move list and found the best
        // possible move/eval_score for us at this depth. We can't improve
        // this any further, so return the result. This called "fail-low".
        alpha
    }
}
