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
        RootMoveAnalysis, SHARP_MARGIN, SearchTerminate, CHECKMATE, CHECK_TERMINATION, DRAW,
        INF, SEND_STATS, STALEMATE,
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
                .lock()
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

        let mut legal_moves_found = 0;
        let mut move_list = MoveList::new();
        refs.mg.generate_moves(refs.board, &mut move_list, MoveType::All);

        Search::score_moves(&mut move_list, tt_move, refs);

        if !quiet && (refs.search_info.nodes & SEND_STATS == 0) {
            Search::send_stats_to_gui(refs);
        }

        let mut best_eval_score = -INF;
        let mut hash_flag = HashFlag::Alpha;
        let mut best_move: ShortMove = ShortMove::new(0);

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
                if do_pvs {
                    eval_score = -Search::alpha_beta(depth - 1, -alpha - 1, -alpha, &mut node_pv, refs);
                    if (eval_score > alpha) && (eval_score < beta) {
                        eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                    }
                } else {
                    eval_score = -Search::alpha_beta(depth - 1, -beta, -alpha, &mut node_pv, refs);
                }
            }

            if is_root {
                let (gr, reply) = if depth > 1 {
                    Search::count_good_replies(depth - 1, alpha, beta, refs)
                } else {
                    (0, None)
                };
                refs.search_info.root_analysis.push(RootMoveAnalysis {
                    mv: current_move,
                    eval: eval_score,
                    good_replies: gr,
                    reply,
                });
            }

            refs.board.unmake();
            refs.search_info.ply -= 1;

            if eval_score > best_eval_score {
                best_eval_score = eval_score;
                best_move = current_move.to_short_move();
            }

            if eval_score >= beta {
                refs.tt.lock().expect(ErrFatal::LOCK).insert(
                    refs.board.game_state.zobrist_key,
                    SearchData::create(depth, refs.search_info.ply, HashFlag::Beta, beta, best_move),
                );

                if current_move.captured() == Pieces::NONE {
                    Search::store_killer_move(current_move, refs);
                    Search::update_history_heuristic(current_move, depth, refs);
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

        if legal_moves_found == 0 {
            if is_check {
                return -CHECKMATE + (refs.search_info.ply as i16);
            } else {
                return STALEMATE;
            }
        }

        refs.tt.lock().expect(ErrFatal::LOCK).insert(
            refs.board.game_state.zobrist_key,
            SearchData::create(depth, refs.search_info.ply, hash_flag, alpha, best_move),
        );

        alpha
    }

    fn count_good_replies(
        depth: i8,
        alpha: i16,
        beta: i16,
        refs: &mut SearchRefs,
    ) -> (usize, Option<Move>) {
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
            .into_iter()
            .filter(|(_, e)| *e <= best_eval + SHARP_MARGIN)
            .map(|(m, _)| m)
            .collect();

        let reply = if good.len() == 1 { Some(good[0]) } else { best_move };

        (good.len(), reply)
    }
}
