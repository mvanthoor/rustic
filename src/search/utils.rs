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
    defs::{
        SearchControl, SearchCurrentMove, SearchMode, SearchRefs, SearchReport, SearchStats,
        SearchTerminate, MAX_KILLER_MOVES, MIN_TIME_CURR_MOVE, MIN_TIME_STATS,
    },
    Search,
};
use crate::{
    board::{defs::Pieces, Board},
    defs::{Sides, MAX_MOVE_RULE},
    engine::defs::{ErrFatal, Information},
    movegen::defs::Move,
};

impl Search {
    // This function calculates the number of nodes per second.
    pub fn nodes_per_second(nodes: usize, msecs: u128) -> usize {
        let mut nps: usize = 0;
        let seconds = msecs as f64 / 1000f64;
        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }
        nps
    }

    // Send intermediate statistics to GUI.
    pub fn send_stats_to_gui(refs: &mut SearchRefs) {
        let elapsed = refs.search_info.timer_elapsed();
        let last_stats = refs.search_info.last_stats_sent;

        if elapsed >= last_stats + MIN_TIME_STATS {
            let hash_full = refs.tt.lock().expect(ErrFatal::LOCK).hash_full();
            let msecs = refs.search_info.timer_elapsed();
            let nps = Search::nodes_per_second(refs.search_info.nodes, msecs);
            let stats = SearchStats::new(msecs, refs.search_info.nodes, nps, hash_full);
            let stats_report = SearchReport::SearchStats(stats);
            let information = Information::Search(stats_report);

            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
            refs.search_info.last_stats_sent = elapsed;
        }
    }

    // Send currently processed move to GUI.
    pub fn send_move_to_gui(refs: &mut SearchRefs, current_move: Move, count: u8) {
        let elapsed = refs.search_info.timer_elapsed();
        let lcm = refs.search_info.last_curr_move_sent;

        if elapsed >= lcm + MIN_TIME_CURR_MOVE {
            let scm = SearchCurrentMove::new(current_move, count);
            let scm_report = SearchReport::SearchCurrentMove(scm);
            let information = Information::Search(scm_report);

            refs.report_tx.send(information).expect(ErrFatal::CHANNEL);
            refs.search_info.last_curr_move_sent = elapsed;
        }
    }

    // This function checks termination conditions and sets the termination
    // flag if this is required.
    pub fn check_termination(refs: &mut SearchRefs) {
        // Terminate search if stop or quit command is received.
        let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        match cmd {
            SearchControl::Stop => refs.search_info.terminate = SearchTerminate::Stop,
            SearchControl::Quit => refs.search_info.terminate = SearchTerminate::Quit,
            SearchControl::Start(_) | SearchControl::Nothing => (),
        };

        // Terminate search if certain conditions are met.
        let search_mode = refs.search_params.search_mode;
        match search_mode {
            SearchMode::Depth => {
                if refs.search_info.depth > refs.search_params.depth {
                    refs.search_info.terminate = SearchTerminate::Stop
                }
            }
            SearchMode::MoveTime => {
                let elapsed = refs.search_info.timer_elapsed();
                if elapsed >= refs.search_params.move_time {
                    refs.search_info.terminate = SearchTerminate::Stop
                }
            }
            SearchMode::Nodes => {
                if refs.search_info.nodes >= refs.search_params.nodes {
                    refs.search_info.terminate = SearchTerminate::Stop
                }
            }
            SearchMode::GameTime => {
                if Search::out_of_time(refs) {
                    refs.search_info.terminate = SearchTerminate::Stop
                }
            }
            SearchMode::Infinite => (), // Handled by a direct 'stop' command
            SearchMode::Nothing => (),  // We're not searching. Nothing to do.
        }
    }

    // Returns true if the position should be evaluated as a draw.
    pub fn is_draw(refs: &SearchRefs) -> bool {
        let is_max_move_rule = refs.board.game_state.halfmove_clock >= MAX_MOVE_RULE;
        Search::is_insufficient_material(refs)
            || Search::is_repetition(refs.board) > 0
            || is_max_move_rule
    }

    // Detects position repetitions in the game's history.
    pub fn is_repetition(board: &Board) -> u8 {
        let mut count = 0;
        let mut stop = false;
        let mut i = board.history.len() - 1;

        // Search the history list.
        while i != 0 && !stop {
            let historic = board.history.get_ref(i);

            // If the historic zobrist key is equal to the one of the board
            // passed into the function, then we found a repetition.
            if historic.zobrist_key == board.game_state.zobrist_key {
                count += 1;
            }

            // If the historic HMC is 0, it indicates that this position
            // was created by a capture or pawn move. We don't have to
            // search further back, because before this, we can't ever
            // repeat. After all, the capture or pawn move can't be
            // reverted or repeated.
            stop = historic.halfmove_clock == 0;

            // Search backwards.
            i -= 1;
        }
        count
    }
}

// This is in its own block so rustfmt::skip can be applied. Otherwhise
// the layout of this function becomes very messy.
#[rustfmt::skip]
impl Search {
    pub fn is_insufficient_material(refs: &SearchRefs) -> bool {
        // It's not a draw if: ...there are still pawns.
        let w_p = refs.board.get_pieces(Pieces::PAWN, Sides::WHITE).count_ones() > 0;     
        let b_p = refs.board.get_pieces(Pieces::PAWN, Sides::BLACK).count_ones() > 0;        
        // ...there's a major piece on the board.
        let w_q = refs.board.get_pieces(Pieces::QUEEN, Sides::WHITE).count_ones() > 0;
        let b_q = refs.board.get_pieces(Pieces::QUEEN, Sides::BLACK).count_ones() > 0;
        let w_r = refs.board.get_pieces(Pieces::ROOK, Sides::WHITE).count_ones() > 0;
        let b_r = refs.board.get_pieces(Pieces::ROOK, Sides::BLACK).count_ones() > 0;
        // ...or two bishops for one side.
        // FIXME : Bishops must be on squares of different color
        let w_b = refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 1;
        let b_b = refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 1;
        // ... or a bishop+knight for at least one side.
        let w_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::WHITE).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::WHITE).count_ones() > 0;
        let b_bn =
            refs.board.get_pieces(Pieces::BISHOP, Sides::BLACK).count_ones() > 0 &&
            refs.board.get_pieces(Pieces::KNIGHT, Sides::BLACK).count_ones() > 0;
         
        // If one of the conditions above is true, we still have enough
        // material for checkmate, so insufficient_material returns false.
        !(w_p || b_p || w_q || b_q || w_r || b_r || w_b || b_b ||  w_bn || b_bn)
    }
}

// Killer moves and history heuristics.
impl Search {
    // This function stores a move in the list of killer moves. Normally we
    // store two killer moves per ply. By checking that the move we want to
    // store is not the same as the first killer move in the list, we make sure
    // that both moves are always different. It is possible to store three or
    // more killer moves, but experience shows that checking for ALL of them to
    // be unique costs more time than the extra killer moves could save.
    pub fn store_killer_move(current_move: Move, refs: &mut SearchRefs) {
        let ply = refs.search_info.ply as usize;
        let first_killer = refs.search_info.killer_moves[0][ply];

        // First killer must not be the same as the move being stored.
        if first_killer.get_move() != current_move.get_move() {
            // Shift all the moves one index upward...
            for i in (1..MAX_KILLER_MOVES).rev() {
                let n = i as usize;
                let previous = refs.search_info.killer_moves[n - 1][ply];
                refs.search_info.killer_moves[n][ply] = previous;
            }

            // and add the new killer move in the first spot.
            refs.search_info.killer_moves[0][ply] = current_move.to_short_move();
        }
    }

    // This function updates a move's history heuristic. The depth * depth
    // part makes sure that moves closer to the root (which have a higher
    // depth value) have more value than moves closer to the leavses.
    pub fn update_history_heuristic(m: Move, depth: i8, refs: &mut SearchRefs) {
        let piece = m.piece();
        let to = m.to();
        let value = depth as u32 * depth as u32;
        refs.search_info.history_heuristic[refs.board.us()][piece][to] += value;
    }
}
