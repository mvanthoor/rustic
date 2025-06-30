/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
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
    defs::{
        RootMoveAnalysis, SearchTerminate, CHECKMATE, CHECK_TERMINATION,
        INF, SEND_STATS, STALEMATE, NULL_MOVE_REDUCTION,
        MULTICUT_DEPTH, MULTICUT_REDUCTION, MULTICUT_CUTOFFS, MULTICUT_MOVES,
    },
    Search, SearchRefs,
};
use crate::{
    defs::MAX_PLY,
    engine::defs::{ErrFatal, HashFlag, SearchData},
    evaluation,
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
        let quiet = refs.search_params.quiet;
        let is_root = refs.search_info.ply == 0;

        // Update thread-local node count
        refs.thread_local_data.increment_nodes();

        if refs.search_info.nodes & CHECK_TERMINATION == 0 {
            Search::check_termination(refs);
        }

        if refs.search_info.terminate != SearchTerminate::Nothing {
            return 0;
        }

        if refs.search_info.ply >= MAX_PLY {
            return evaluation::evaluate_position(refs.board);
        }

        let is_check = refs.mg.square_attacked(
            refs.board,
            refs.board.opponent(),
            refs.board.king_square(refs.board.us()),
        );

        if is_check {
            depth += 1;
        }

        if depth <= 0 {
            return Search::quiescence(alpha, beta, pv, refs);
        }

        refs.search_info.nodes += 1;

        let mut tt_value: Option<i16> = None;
        let mut tt_move: ShortMove = ShortMove::new(0);

        // First check thread-local TT cache to reduce global TT access
        if refs.tt_enabled {
            if let Some(data) = refs.thread_local_data.local_tt_cache.probe(refs.board.game_state.zobrist_key) {
                let tt_result = data.get(depth, refs.search_info.ply, alpha, beta);
                tt_value = tt_result.0;
                tt_move = tt_result.1;
            } else {
                // Fall back to global TT only if not found in local cache
                if let Some(data) = refs
                    .tt
                    .read()
                    .expect(ErrFatal::LOCK)
                    .probe(refs.board.game_state.zobrist_key)
                {
                    let tt_result = data.get(depth, refs.search_info.ply, alpha, beta);
                    tt_value = tt_result.0;
                    tt_move = tt_result.1;
                    
                    // Cache the result locally for future access
                    refs.thread_local_data.local_tt_cache.insert(
                        refs.board.game_state.zobrist_key,
                        *data,
                    );
                }
            }
        }

        if let Some(v) = tt_value {
            if !is_root {
                return v;
            }
        }

        // cut off branches early when a null move proves sufficient
        if !is_root
            && depth > NULL_MOVE_REDUCTION
            && !is_check
            && !Search::is_insufficient_material(refs)
        {
            refs.board.make_null_move();
            refs.search_info.ply += 1;
            let mut tmp_pv: Vec<Move> = Vec::new();
            let score = -Search::alpha_beta(
                depth - 1 - NULL_MOVE_REDUCTION,
                -beta,
                -beta + 1,
                &mut tmp_pv,
                refs,
            );
            refs.board.unmake_null_move();
            refs.search_info.ply -= 1;

            if score >= beta {
                return beta;
            }
        }

        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg.generate_moves(refs.board, &mut move_list, MoveType::All);

        Search::score_moves(&mut move_list, tt_move, refs);

        if !is_root && depth >= MULTICUT_DEPTH && !is_check {
            let max_moves = std::cmp::min(MULTICUT_MOVES as usize, move_list.len() as usize);
            let mut cutoffs = 0;
            for j in 0..max_moves {
                Search::pick_move(&mut move_list, j as u8);
                let mcut = move_list.get_move(j as u8);
                if !refs.board.make(mcut, refs.mg) {
                    continue;
                }
                refs.search_info.ply += 1;
                let mut tmp_pv: Vec<Move> = Vec::new();
                let score = -Search::alpha_beta(
                    depth - 1 - MULTICUT_REDUCTION,
                    -beta,
                    -beta + 1,
                    &mut tmp_pv,
                    refs,
                );
                refs.board.unmake();
                refs.search_info.ply -= 1;
                if score >= beta {
                    cutoffs += 1;
                    if cutoffs >= MULTICUT_CUTOFFS as usize {
                        return beta;
                    }
                }
            }
        }

        if !quiet && (refs.search_info.nodes & SEND_STATS == 0) {
            Search::send_stats_to_gui(refs);
        }

        let mut best_eval_score = -INF;
        let mut hash_flag = HashFlag::Alpha;
        let mut best_move: ShortMove = ShortMove::new(0);

        // Store evaluated root moves so sharp sequences can be collected later.
        let mut root_analysis: Vec<RootMoveAnalysis> = Vec::new();

        for i in 0..move_list.len() as usize {
            if Search::time_up(refs) {
                break;
            }

            Search::pick_move(&mut move_list, i as u8);
            let current_move = move_list.get_move(i as u8);

            if !refs.board.make(current_move, refs.mg) {
                continue;
            }

            refs.search_info.ply += 1;
            legal_moves_found += 1;

            let mut tmp_pv: Vec<Move> = Vec::new();
            let mut score: i16;

            if legal_moves_found > 1 {
                score = -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut tmp_pv, refs);
                if score > alpha && score < beta {
                    score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut tmp_pv, refs);
                }
            } else {
                score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut tmp_pv, refs);
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            if refs.search_info.terminate != SearchTerminate::Nothing {
                break;
            }

            if score > best_eval_score {
                best_eval_score = score;
                best_move = current_move.to_short_move();

                if score > alpha {
                    hash_flag = HashFlag::Exact;
                    alpha = score;
                    pv.clear();
                    pv.push(current_move);
                    pv.extend(tmp_pv);

                    if is_root {
                        refs.thread_local_data.update_best_move(current_move);
                    }

                    if score >= beta {
                        hash_flag = HashFlag::Beta;
                        break;
                    }
                }
            }

            if is_root {
                let mut good_replies = 0;
                let mut reply: Option<Move> = None;
                let mut reply_sequence: Vec<Move> = Vec::new();

                if score > alpha - refs.search_params.sharp_margin {
                    (good_replies, reply, reply_sequence) = Search::collect_sharp_sequence(
                        depth - 1,
                        -beta,
                        -alpha + refs.search_params.sharp_margin,
                        refs,
                    );
                }

                root_analysis.push(RootMoveAnalysis {
                    mv: current_move,
                    eval: score,
                    good_replies,
                    reply,
                    reply_sequence,
                });
            }
        }

        if legal_moves_found == 0 {
            if is_check {
                return -CHECKMATE + refs.search_info.ply as i16;
            } else {
                return STALEMATE;
            }
        }

        // Store position in TT using thread-local batching
        if refs.tt_enabled {
            let tt_data = SearchData::create(
                depth,
                refs.search_info.ply,
                hash_flag,
                best_eval_score,
                best_move,
            );

            // Add to thread-local batch instead of immediate global TT write
            refs.thread_local_data.tt_batch.add(
                refs.board.game_state.zobrist_key,
                tt_data,
            );

            // Flush batch if it's full
            if refs.thread_local_data.tt_batch.is_full() {
                Search::flush_tt_batch(refs);
            }
        }

        if is_root {
            refs.search_info.root_analysis = root_analysis;
        }

        best_eval_score
    }

    /// Flush thread-local TT batch to global TT
    pub fn flush_tt_batch(refs: &mut SearchRefs) {
        if refs.thread_local_data.tt_batch.len() > 0 {
            if let Ok(mut tt_write) = refs.tt.write() {
                for update in &refs.thread_local_data.tt_batch.updates {
                    tt_write.insert(update.zobrist_key, update.data);
                }
            }
            refs.thread_local_data.tt_batch.clear();
        }
    }

    fn collect_sharp_sequence(
        depth: i8,
        alpha: i16,
        beta: i16,
        refs: &mut SearchRefs,
    ) -> (usize, Option<Move>, Vec<Move>) {
        let mut move_list = MoveList::new();
        refs.mg.generate_moves(refs.board, &mut move_list, MoveType::All);

        let mut evals: Vec<(Move, i16)> = Vec::new();
        let mut best_eval = INF;
        let mut best_move: Option<Move> = None;

        for i in 0..move_list.len() {
            if Search::time_up(refs) {
                break;
            }

            let mv = move_list.get_move(i);
            if refs.board.make(mv, refs.mg) {
                refs.search_info.ply += 1;
                let mut node_pv: Vec<Move> = Vec::new();
                let score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                if Search::time_up(refs) {
                    refs.board.unmake();
                    refs.search_info.ply -= 1;
                    return (0, None, Vec::new());
                }
                refs.board.unmake();
                refs.search_info.ply -= 1;

                if score < best_eval {
                    best_eval = score;
                    best_move = Some(mv);
                }
                evals.push((mv, score));
            }
        }

        let good: Vec<Move> = evals
            .iter()
            .filter(|(_, e)| *e <= best_eval + refs.search_params.sharp_margin)
            .map(|(m, _)| *m)
            .collect();

        let reply = if good.len() == 1 { Some(good[0]) } else { best_move };

        if good.len() != 1 || depth <= 1 || reply.is_none() {
            return (good.len(), reply, Vec::new());
        }

        let forced = good[0];
        let mut sequence: Vec<Move> = vec![forced];

        if refs.board.make(forced, refs.mg) {
            refs.search_info.ply += 1;
            let mut pv: Vec<Move> = Vec::new();
            Search::alpha_beta(depth - 1, alpha, beta, &mut pv, refs);
            if Search::time_up(refs) {
                refs.board.unmake();
                refs.search_info.ply -= 1;
                return (0, None, sequence);
            }

            if depth > 2 {
                if let Some(my_move) = pv.get(0).cloned() {
                    if refs.board.make(my_move, refs.mg) {
                        refs.search_info.ply += 1;
                        let (_, _, mut next_seq) =
                            Search::collect_sharp_sequence(depth - 2, alpha, beta, refs);
                        if Search::time_up(refs) {
                            refs.board.unmake();
                            refs.search_info.ply -= 1;
                            return (0, Some(forced), sequence);
                        }
                        sequence.append(&mut next_seq);
                        refs.board.unmake();
                        refs.search_info.ply -= 1;
                    }
                }
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;
        }

        (good.len(), reply, sequence)
    }
}
