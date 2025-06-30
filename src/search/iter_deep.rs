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

        // Initialize thread-local data for this search
        refs.thread_local_data.start_search();

        if is_game_time {
            // Apply emergency time management first
            Search::emergency_time_management(refs);
            
            // Use enhanced time slice calculation
            let time_slice = Search::calculate_enhanced_time_slice(refs);
            let factor = Search::dynamic_time_factor(refs);

            if time_slice > 0 {
                refs.search_info.allocated_time = (time_slice as f64 * factor).round() as u128;
            } else {
                refs.search_params.search_mode = SearchMode::Depth;
                refs.search_params.depth = 1;
            }
        }

        refs.search_info.timer_start();
        
        // Clear TT caches at the start of a new search
        Search::clear_tt_caches(refs);
        
        while (depth <= refs.search_info.max_depth) && (depth <= refs.search_params.depth) && !stop {
            refs.search_info.depth = depth;
            refs.thread_local_data.search_depth = depth;
            refs.search_info.root_analysis.clear();

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

            // Always update best_move if we have a valid PV, even if interrupted
            if !root_pv.is_empty() {
                best_move = root_pv[0];
                refs.thread_local_data.update_best_move(best_move);
            } else if !refs.search_info.root_analysis.is_empty() {
                // Fallback: if we have evaluated moves but no PV (interrupted early), 
                // use the first evaluated move as best move
                best_move = refs.search_info.root_analysis[0].mv;
                refs.thread_local_data.update_best_move(best_move);
            } else if best_move.get_move() == 0 && !refs.search_info.root_analysis.is_empty() {
                // Additional fallback: if best_move is still null but we have root analysis,
                // use the first move from root analysis
                best_move = refs.search_info.root_analysis[0].mv;
                refs.thread_local_data.update_best_move(best_move);
            }

            if !refs.search_info.interrupted() {
                let elapsed = refs.search_info.timer_elapsed();
                let nodes = refs.search_info.nodes;
                let hash_full = refs.tt.read().expect(ErrFatal::LOCK).hash_full();

                let forced_lines: Vec<(Move, Vec<Move>)> = refs
                    .search_info
                    .root_analysis
                    .iter()
                    .filter(|a| a.good_replies == 1)
                    .map(|a| (a.mv, a.reply_sequence.clone()))
                    .collect();

                let pv_to_send = root_pv.clone();

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

                // Enhanced sharp move logging
                if !refs.search_info.root_analysis.is_empty() {
                    // Check if the best move is a sharp line
                    let best_move_analysis = refs.search_info.root_analysis
                        .iter()
                        .find(|a| a.mv == best_move);
                    
                    if let Some(best_analysis) = best_move_analysis {
                        if best_analysis.good_replies == 1 && !best_analysis.reply_sequence.is_empty() {
                            // The best move is a sharp line - log it with top alternatives
                            let mut sorted_analysis = refs.search_info.root_analysis.clone();
                            sorted_analysis.sort_by(|a, b| b.eval.cmp(&a.eval));
                            
                            let sequence_str = best_analysis.reply_sequence
                                .iter()
                                .map(|m| m.as_string())
                                .collect::<Vec<String>>()
                                .join(" ");
                            
                            let mut msg = format!(
                                "Sharp line chosen: {} (eval: {}) -> {}", 
                                best_move.as_string(), 
                                best_analysis.eval, 
                                sequence_str
                            );
                            
                            // Add top 3 alternatives (excluding the best move)
                            let alternatives: Vec<_> = sorted_analysis
                                .iter()
                                .filter(|a| a.mv != best_move)
                                .take(3)
                                .collect();
                            
                            if !alternatives.is_empty() {
                                msg.push_str(" | Alternatives: ");
                                let alt_strs: Vec<String> = alternatives
                                    .iter()
                                    .map(|a| format!("{} ({})", a.mv.as_string(), a.eval))
                                    .collect();
                                msg.push_str(&alt_strs.join(", "));
                            }
                            
                            let report = SearchReport::InfoString(msg);
                            let information = Information::Search(report);
                            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
                        }
                    }
                    
                    // Also log any other sharp lines that weren't chosen (legacy behaviour)
                    let other_sharp_lines: Vec<_> = forced_lines
                        .iter()
                        .filter(|(mv, _)| *mv != best_move)
                        .collect();
                    
                    if !other_sharp_lines.is_empty() {
                        let mut parts: Vec<String> = Vec::new();
                        for (mv, seq) in other_sharp_lines.iter() {
                            let seq_str = seq
                                .iter()
                                .map(|m| m.as_string())
                                .collect::<Vec<String>>()
                                .join(" ");
                            parts.push(format!("{} -> {}", mv.as_string(), seq_str));
                        }
                        let msg = format!("Other sharp lines available: {}", parts.join(" | "));
                        let report = SearchReport::InfoString(msg);
                        let information = Information::Search(report);
                        refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
                    }
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

        // Flush any remaining TT updates before finishing
        Search::flush_tt_batch(refs);

        // Update time statistics
        if is_game_time {
            let time_used = refs.search_info.timer_elapsed();
            // Success is determined by whether we found a valid move, not by time usage
            let success = best_move.get_move() != 0 && !refs.search_info.interrupted();
            Search::update_time_statistics(refs, time_used, success);
            
            // Send time management statistics to GUI for monitoring
            let stats_msg = Search::display_time_statistics(refs);
            let report = SearchReport::InfoString(stats_msg);
            let information = Information::Search(report);
            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
        }

        // Final fallback: if we still don't have a valid move, generate moves and use the first legal one
        if best_move.get_move() == 0 {
            let mut move_list = crate::movegen::defs::MoveList::new();
            refs.mg.generate_moves(refs.board, &mut move_list, crate::movegen::defs::MoveType::All);
            
            for i in 0..move_list.len() {
                let mv = move_list.get_move(i);
                if refs.board.make(mv, refs.mg) {
                    refs.board.unmake();
                    best_move = mv;
                    refs.thread_local_data.update_best_move(best_move);
                    break;
                }
            }
        }

        (best_move, refs.search_info.terminate)
    }
}
