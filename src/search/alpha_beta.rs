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
    defs::{SearchTerminate, CHECKMATE, CHECK_TERMINATION, DRAW, INF, SEND_STATS, STALEMATE},
    Search, SearchRefs,
};
use crate::{
    board::defs::Pieces,
    defs::MAX_PLY,
    engine::defs::{ErrFatal, HashFlag, SearchData},
    evaluation::Evaluation,
    movegen::defs::{Move, MoveList, MoveType, ShortMove},
};

impl Search {
    pub fn alpha_beta(
        mut depth: i8,
        mut alpha: i16,
        beta: i16,
        pv: &mut Vec<Move>,
        refs: &mut SearchRefs,
    ) -> i16 {
        let quiet = refs.search_params.quiet; // If quiet, don't send intermediate stats.
        let is_root = refs.search_info.ply == 0; // At root if no moves were played.
        let mut do_pvs = false; // Used for PVS (Principal Variation Search)

        // Check if termination condition is met.
        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        // If time is up, abort. This depth won't be considered in
        // iterative deepening as it is unfinished.
        if refs.search_info.terminate != SearchTerminate::Nothing {
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
        if refs.tt_enabled {
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
        }

        // If we have a value from the TT, then return immediately.
        if let Some(v) = tt_value {
            if !is_root {
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

        // If not quiet, periodically send stats to the GUI.
        if !quiet && (refs.search_info.nodes & SEND_STATS == 0) {
            Search::send_stats_to_gui(refs);
        }

        // Workign variables for finding the best eval and move.
        let mut best_eval_score = -INF;
        let mut hash_flag = HashFlag::Alpha;
        let mut best_move: ShortMove = ShortMove::new(0);

        // Iterate over the moves.
        for i in 0..move_list.len() {
            // Pick the highest oredered move first.
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
            if refs.search_info.ply > refs.search_info.seldepth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            // Send currently searched move to GUI.
            if !quiet && is_root {
                Search::send_move_to_gui(refs, current_move, legal_moves_found);
            }

            // Create a node PV for this move.
            let mut node_pv: Vec<Move> = Vec::new();

            // We just made a move. We are not yet at one of the leaf
            // nodes, so if the position is not a draw, we must search
            // deeper. Initially, assume the position is a draw.
            let mut eval_score = DRAW;

            // If it isn't a draw, we must search.
            if !Search::is_draw(refs) {
                // Try a PVS if applicable.
                if do_pvs {
                    eval_score =
                        -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);

                    // Check if we failed the PVS.
                    if (eval_score > alpha) && (eval_score < beta) {
                        eval_score =
                            -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            }

            // Take back the move, and decrease ply accordingly.
            refs.board.unmake();
            refs.search_info.ply -= 1;

            // eval_score is better than the best we found so far, so we
            // save a new best_move that'll go into the hash table.
            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_move = current_move.to_short_move();
            }

            // Beta cutoff: this move is so good for our opponent, that we
            // do not search any further. Insert into TT and return beta.
            if eval_score >= beta {
                refs.tt.lock().expect(ErrFatal::LOCK).insert(
                    refs.board.game_state.zobrist_key,
                    SearchData::create(
                        depth,
                        refs.search_info.ply,
                        HashFlag::Beta,
                        beta,
                        best_move,
                    ),
                );

                // If the move is not a capture but still causes a
                // beta-cutoff, then store it as a killer move.
                if current_move.captured() == Pieces::NONE {
                    Search::store_killer_move(current_move, refs);
                }

                return beta;
            }

            // We found a better move for us.
            if eval_score > alpha {
                // Save our better evaluation score as alpha.
                alpha = eval_score;

                // This is an exact move score.
                hash_flag = HashFlag::Exact;

                // Update the Principal Variation.
                do_pvs = true;
                pv.clear();
                pv.push(current_move);
                pv.append(&mut node_pv);
            }
        }

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

        // We save the best move we found for us; with an ALPHA flag if we
        // didn't improve alpha, or EXACT if we did raise alpha.
        refs.tt.lock().expect(ErrFatal::LOCK).insert(
            refs.board.game_state.zobrist_key,
            SearchData::create(depth, refs.search_info.ply, hash_flag, alpha, best_move),
        );

        // We have traversed the entire move list and found the best
        // possible move/eval_score for us.
        alpha
    }
}
