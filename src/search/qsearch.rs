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
    defs::{SearchTerminate, CHECK_TERMINATION, MIN_TIME_STATS, SEND_STATS},
    Search, SearchRefs,
};
use crate::{
    defs::MAX_DEPTH,
    evaluation,
    movegen::defs::{Move, MoveList, MoveType},
};

impl Search {
    pub fn quiescence(mut alpha: i16, beta: i16, pv: &mut Vec<Move>, refs: &mut SearchRefs) -> i16 {
        // No intermediate stats updates if quiet.
        let quiet = refs.search_params.quiet;

        // Check if search needs to be terminated.
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        if refs.search_info.ply >= MAX_DEPTH {
            return evaluation::evaluate_position(refs.board);
        }

        // Do a stand-pat here: Check how we're doing, even before we make
        // a move. If the evaluation score is larger than beta, then we're
        // already so bad we don't need to search any further. Just return
        // the beta score.
        let eval_score = evaluation::evaluate_position(refs.board);
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

        // Temporary variables.
        let mut best_move = Move::new(0);

        // Generate only capture moves.
        let mut move_list = MoveList::new();
        let mtc = MoveType::Capture;
        refs.mg.generate_moves(refs.board, &mut move_list, mtc);

        // Do move scoring, so the best move will be searched first.
        Search::score_moves(&mut move_list);

        // We created a new node which we'll search, so count it.
        refs.search_info.nodes += 1;

        // Update search stats in the GUI. Check every SEND_STATS nodes if
        // the minium MIN_TIME_STATS has elapsed before sending.
        if !quiet && (refs.search_info.nodes & SEND_STATS == 0) {
            let elapsed = refs.search_info.timer_elapsed();
            let last_stats = refs.search_info.last_stats_sent;
            if elapsed >= last_stats + MIN_TIME_STATS {
                Search::send_stats(refs);
                refs.search_info.last_stats_sent = elapsed;
            }
        }

        // Iterate over the capture moves.
        for i in 0..move_list.len() {
            // Break the loop if a termination condition was met.
            if refs.search_info.terminate != SearchTerminate::Nothing {
                break;
            }

            // Pick the next moves with the higest score.
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
            if refs.search_info.ply > refs.search_info.depth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Create a PV for this node.
            let mut node_pv: Vec<Move> = Vec::new();

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
                best_move = current_move;
                pv.clear();
                pv.push(best_move);
                pv.append(&mut node_pv);
            }
        }

        // Store the best move and best PV we found.
        refs.search_info.best_move = best_move;
        refs.search_info.pv = pv.clone();

        // We have traversed the entire move list and found the best score for us,
        // so we return this.
        alpha
    }
}
