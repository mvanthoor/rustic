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
        RootMoveAnalysis, SearchTerminate, CHECKMATE, CHECK_TERMINATION, DRAW,
        INF, SEND_STATS, STALEMATE, NULL_MOVE_REDUCTION, LMR_REDUCTION, LMR_MOVE_THRESHOLD,
        LMR_LATE_THRESHOLD, LMR_LATE_REDUCTION, RECAPTURE_EXTENSION,
        MULTICUT_DEPTH, MULTICUT_REDUCTION, MULTICUT_CUTOFFS, MULTICUT_MOVES,
        SHARP_SEQUENCE_DEPTH_CAP,
    },
    Search, SearchRefs,
};
use crate::{
    board::defs::Pieces,
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
        let mut do_pvs = false;

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

        if refs.tt_enabled {
            if let Some(data) = refs
                .tt
                .read()
                .expect(ErrFatal::LOCK)
                .probe(refs.board.game_state.zobrist_key)
            {
                let tt_result = data.get(depth, refs.search_info.ply, alpha, beta);
                tt_value = tt_result.0;
                tt_move = tt_result.1;
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
        let mut best_index: usize = 0;

        // Store evaluated root moves so sharp sequences can be collected later.
        let mut root_moves: Vec<(Move, i16, i16)> = Vec::new();
        let mut root_analysis: Vec<RootMoveAnalysis> = Vec::new();

        for i in 0..move_list.len() {
            Search::pick_move(&mut move_list, i);
            let current_move = move_list.get_move(i);

            if !refs.board.make(current_move, refs.mg) {
                continue;
            }

            legal_moves_found += 1;
            refs.search_info.ply += 1;

            if refs.search_info.ply > refs.search_info.seldepth {
                refs.search_info.seldepth = refs.search_info.ply;
            }

            if !quiet && is_root {
                Search::send_move_to_gui(refs, current_move, legal_moves_found);
            }

            let mut node_pv: Vec<Move> = Vec::new();
            let mut eval_score = DRAW;

            if !Search::is_draw(refs) {

                let is_quiet = current_move.captured() == Pieces::NONE;
                let apply_lmr = !is_root
                    && depth > 2
                    && !is_check
                    && is_quiet
                    && i >= LMR_MOVE_THRESHOLD;
                
                let mut r = if apply_lmr { LMR_REDUCTION } else { 0 };
                if apply_lmr && i >= LMR_LATE_THRESHOLD {
                    r = LMR_LATE_REDUCTION;
                }

                let mut ext = 0;
                if refs.board.history.len() > 0 {
                    let prev = refs.board.history.get_ref(refs.board.history.len() - 1).next_move;
                    if prev.captured() != Pieces::NONE
                        && current_move.captured() != Pieces::NONE
                        && prev.to() == current_move.to()
                    {
                        ext = RECAPTURE_EXTENSION;
                    }
                }

                if do_pvs {
                    eval_score = -Search::alpha_beta(depth - 1 - r + ext, -alpha - 1, -alpha, &mut node_pv, refs);

                    if (eval_score > alpha) && (eval_score < beta) {

                        eval_score = -Search::alpha_beta(depth - 1 + ext, -beta, -alpha, &mut node_pv, refs);
                    } else if apply_lmr && eval_score > alpha {
                        eval_score = -Search::alpha_beta(depth - 1 + ext, -beta, -alpha, &mut node_pv, refs);
                    } 
                } else {
                    eval_score = -Search::alpha_beta(depth - 1 - r + ext, -beta, -alpha, &mut node_pv, refs);
                    if apply_lmr && eval_score > alpha {
                        eval_score = -Search::alpha_beta(depth - 1 + ext, -beta, -alpha, &mut node_pv, refs);
                    }
                }
            }

            if is_root {
                root_moves.push((current_move, eval_score, alpha));
                root_analysis.push(RootMoveAnalysis {
                    mv: current_move,
                    eval: eval_score,
                    good_replies: 0,
                    reply: None,
                    reply_sequence: Vec::new(),
                });
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_move = current_move.to_short_move();
                best_index = root_moves.len() - 1;
            }

            if eval_score >= beta {
                refs.tt.write().expect(ErrFatal::LOCK).insert(
                    refs.board.game_state.zobrist_key,
                    SearchData::create(depth, refs.search_info.ply, HashFlag::Beta, beta, best_move),
                );

                if current_move.captured() == Pieces::NONE {
                    Search::store_killer_move(current_move, refs);
                    Search::update_history_heuristic(current_move, depth, refs);
                }

                if refs.board.history.len() > 0 {
                    let prev = refs.board.history.get_ref(refs.board.history.len() - 1).next_move;
                    Search::store_counter_move(prev, current_move, refs);
                }

                return beta;
            }

            if eval_score > alpha {
                alpha = eval_score;
                hash_flag = HashFlag::Exact;
                do_pvs = true;
                pv.clear();
                pv.push(current_move);
                pv.append(&mut node_pv);
            }
        }

        if is_root && depth > 1 {
            let seq_depth = std::cmp::min(depth - 1, SHARP_SEQUENCE_DEPTH_CAP);
            if depth <= SHARP_SEQUENCE_DEPTH_CAP {
                for (idx, (mv, _, a)) in root_moves.iter().enumerate() {
                    if refs.board.make(*mv, refs.mg) {
                        refs.search_info.ply += 1;
                        let (gr, reply, seq) =
                            Search::collect_sharp_sequence(seq_depth, *a, beta, refs);
                        refs.board.unmake();
                        refs.search_info.ply -= 1;
                        root_analysis[idx].good_replies = gr;
                        root_analysis[idx].reply = reply;
                        root_analysis[idx].reply_sequence = seq;
                    }
                }
            } else if let Some((mv, _, a)) = root_moves.get(best_index) {
                if refs.board.make(*mv, refs.mg) {
                    refs.search_info.ply += 1;
                    let (gr, reply, seq) =
                        Search::collect_sharp_sequence(seq_depth, *a, beta, refs);
                    refs.board.unmake();
                    refs.search_info.ply -= 1;
                    root_analysis[best_index].good_replies = gr;
                    root_analysis[best_index].reply = reply;
                    root_analysis[best_index].reply_sequence = seq;
                }
            }
            refs.search_info.root_analysis.append(&mut root_analysis);
        }

        if legal_moves_found == 0 {
            if is_check {
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        refs.tt.write().expect(ErrFatal::LOCK).insert(
            refs.board.game_state.zobrist_key,
            SearchData::create(depth, refs.search_info.ply, hash_flag, alpha, best_move),
        );

        alpha
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
            let mv = move_list.get_move(i);
            if refs.board.make(mv, refs.mg) {
                refs.search_info.ply += 1;
                let mut node_pv: Vec<Move> = Vec::new();
                let score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
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

            if depth > 2 {
                if let Some(my_move) = pv.get(0).cloned() {
                    if refs.board.make(my_move, refs.mg) {
                        refs.search_info.ply += 1;
                        let (_, _, mut next_seq) =
                            Search::collect_sharp_sequence(depth - 2, alpha, beta, refs);
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
