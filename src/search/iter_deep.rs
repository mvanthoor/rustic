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
    defs::{SearchMode, SearchRefs, SearchResult, INF, ASPIRATION_WINDOW},
    ErrFatal, Information, Search, SearchReport, SearchSummary,
};
use crate::{defs::MAX_PLY, movegen::defs::Move};

// Actual search routines.
impl Search {
    pub fn iterative_deepening(refs: &mut SearchRefs) -> SearchResult {
        let mut depth = 1;
        let mut best_move = Move::new(0);
        let mut root_pv: Vec<Move> = Vec::new();
        let mut stop = false;
        let mut prev_eval: i16 = 0;
        let is_game_time = refs.search_params.is_game_time();

        if is_game_time {
            let time_slice = Search::calculate_time_slice(refs);
            let factor = 0.40;

            if time_slice > 0 {
                refs.search_info.allocated_time = (time_slice as f64 * factor).round() as u128;
            } else {
                refs.search_params.search_mode = SearchMode::Depth;
                refs.search_params.depth = 1;
            }
        }

        // let alpha: i16 = -INF;
        // let beta: i16 = INF;

        refs.search_info.timer_start();
        while (depth <= MAX_PLY) && (depth <= refs.search_params.depth) && !stop {
            refs.search_info.depth = depth;
            refs.search_info.root_analysis.clear();

            //let eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);

            let mut alpha = if depth > 1 {
                prev_eval - ASPIRATION_WINDOW
            } else {
                -INF
            };
            let mut beta = if depth > 1 {
                prev_eval + ASPIRATION_WINDOW
            } else {
                INF
            };

            root_pv.clear();
            let mut eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);

            if (eval <= alpha) || (eval >= beta) {
                alpha = -INF;
                beta = INF;
                root_pv.clear();
                eval = Search::alpha_beta(depth, alpha, beta, &mut root_pv, refs);
            }
            prev_eval = eval;


            if !refs.search_info.interrupted() {
                if !root_pv.is_empty() {
                    best_move = root_pv[0];
                }

                let elapsed = refs.search_info.timer_elapsed();
                let nodes = refs.search_info.nodes;
                let hash_full = refs.tt.lock().expect(ErrFatal::LOCK).hash_full();

                let forced_lines: Vec<(Move, Vec<Move>)> = refs
                    .search_info
                    .root_analysis
                    .iter()
                    .filter(|a| a.good_replies == 1)
                    .map(|a| (a.mv, a.reply_sequence.clone()))
                    .collect();

                let forced_moves: Vec<Move> = forced_lines.iter().map(|(m, _)| *m).collect();

                let pv_to_send = if !forced_moves.is_empty() {
                    forced_moves.clone()
                } else {
                    root_pv.clone()
                };

                let summary = SearchSummary {
                    depth,
                    seldepth: refs.search_info.seldepth,
                    time: elapsed,
                    cp: eval,
                    mate: 0,
                    nodes,
                    nps: Search::nodes_per_second(nodes, elapsed),
                    hash_full,
                    pv: pv_to_send,
                };

                let report = SearchReport::SearchSummary(summary);
                let information = Information::Search(report);
                refs.report_tx.send(information).expect(ErrFatal::CHANNEL);

                if !forced_lines.is_empty() {
                    let mut parts: Vec<String> = Vec::new();
                    for (mv, seq) in forced_lines.iter() {
                        let seq_str = seq
                            .iter()
                            .map(|m| m.as_string())
                            .collect::<Vec<String>>()
                            .join(" ");
                        parts.push(format!("{} -> {}", mv.as_string(), seq_str));
                    }
                    let msg = format!("sharp lines: {}", parts.join(" | "));
                    let report = SearchReport::InfoString(msg);
                    let information = Information::Search(report);
                    refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
                }

                depth += 1;
            }

            let time_up = if is_game_time {
                refs.search_info.timer_elapsed() > refs.search_info.allocated_time
            } else {
                false
            };

            stop = refs.search_info.interrupted() || time_up;
        }

        (best_move, refs.search_info.terminate)
    }
}
