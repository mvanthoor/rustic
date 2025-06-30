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

use super::{defs::SearchRefs, Search};
use crate::defs::Sides;
use super::defs::{
    GamePhase, TimeControl, MoveQuality, TimeStats,
    OPENING_PLY_THRESHOLD, EARLY_MIDDLEGAME_PLY_THRESHOLD, LATE_MIDDLEGAME_PLY_THRESHOLD, ENDGAME_PIECE_THRESHOLD,
    EMERGENCY_TIME_THRESHOLD, EMERGENCY_MAX_DEPTH, EMERGENCY_TIME_FACTOR
};
use crate::defs::MAX_PLY;

pub const OVERHEAD: i128 = 50; // msecs
const GAME_LENGTH: usize = 25; // moves
const MOVES_BUFFER: usize = 5; //moves
const CRITICAL_TIME: u128 = 1_000; // msecs
const OK_TIME: u128 = CRITICAL_TIME * 5; // msecs

impl Search {
    // Determine if allocated search time has been used up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.timer_elapsed();
        let allocated = refs.search_info.allocated_time;

        // Calculate a factor with which it is allowed to overshoot the
        // allocated search time. The more time the engine has, the larger
        // the overshoot-factor can be.
        let overshoot_factor = match allocated {
            x if x > OK_TIME => 1.0,                       // Allow large overshoot.
            x if x > CRITICAL_TIME && x <= OK_TIME => 1.5, // Low on time. Reduce overshoot.
            x if x <= CRITICAL_TIME => 1.0,                // Critical time. Don't overshoot.
            _ => 1.0,                                      // This case shouldn't happen.
        };

        elapsed >= (overshoot_factor * allocated as f64).round() as u128
    }

    pub fn time_up(refs: &mut SearchRefs) -> bool {
        Search::out_of_time(refs) || refs.search_info.interrupted()
    }

    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        // Calculate the time slice step by step.
        let gt = &refs.search_params.game_time;
        let mtg = Search::adaptive_moves_to_go(refs);
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let increment = if white { gt.winc } else { gt.binc } as i128;
        let base_time = ((clock as f64) / (mtg as f64)).round() as i128;
        let time_slice = base_time + increment - OVERHEAD;

        // Make sure we're never sending less than 0 msecs of available time.
        if time_slice > 0 {
            // Just send the calculated slice.
            time_slice as u128
        } else if (base_time + increment) > (OVERHEAD / 5) {
            // Don't substract GUI lag protection (overhead) if this leads
            // to a negative time allocation.
            (base_time + increment) as u128
        } else {
            // We actually don't have any time.
            0
        }
    }

    // Determine a factor for how much of the available time for a move
    // should actually be used. The idea is to spend more time when there
    // is plenty on the clock and reduce thinking time in critical stages.
    pub fn dynamic_time_factor(refs: &SearchRefs) -> f64 {
        let gt = &refs.search_params.game_time;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime } as f64;

        // Up to one minute on the clock scales linearly between 0.3 and 0.6.
        let base = 0.3_f64;
        let max_add = 0.3_f64;
        let max_clock = 60_000_f64; // cap at a minute
        let capped = if clock > max_clock { max_clock } else { clock };

        base + (capped / max_clock) * max_add
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was supplied, then use this.
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            x
        } else {
            // Guess moves to go if not supplied.
            let white = refs.board.us() == Sides::WHITE;
            let ply = refs.board.history.len();
            let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };

            GAME_LENGTH - (moves_made % GAME_LENGTH) + MOVES_BUFFER
        }
    }

    // Adaptive moves-to-go based on game phase
    pub fn adaptive_moves_to_go(refs: &SearchRefs) -> usize {
        if let Some(mtg) = refs.search_params.game_time.moves_to_go {
            return mtg;
        }
        
        let ply = refs.board.history.len();
        let piece_count = refs.board.piece_count();
        
        // Adaptive estimation based on game phase
        // This is incredibly basic and needs to be improved, but should be good enough 
        if ply <= OPENING_PLY_THRESHOLD {
            30  // Opening: more moves expected
        } else if ply <= EARLY_MIDDLEGAME_PLY_THRESHOLD {
            if piece_count >= 20 {
                25  // Early middlegame
            } else if piece_count >= 10 {
                20  // Late middlegame
            } else {
                15  // Late game with pieces
            }
        } else {
            if piece_count >= 10 {
                15  // Late game with pieces
            } else {
                10  // Endgame
            }
        }
    }

    // Determine game phase based on ply and piece count
    pub fn determine_game_phase(refs: &SearchRefs) -> GamePhase {
        let ply = refs.board.history.len();
        let piece_count = refs.board.piece_count();
        
        if ply <= OPENING_PLY_THRESHOLD {
            GamePhase::Opening
        } else if ply <= EARLY_MIDDLEGAME_PLY_THRESHOLD {
            GamePhase::EarlyMiddlegame
        } else if ply <= LATE_MIDDLEGAME_PLY_THRESHOLD {
            GamePhase::LateMiddlegame
        } else if piece_count <= ENDGAME_PIECE_THRESHOLD {
            GamePhase::Endgame
        } else {
            GamePhase::LateMiddlegame
        }
    }

    // Classify time control based on total time available
    // This does not include increment time so isn't perfect
    pub fn classify_time_control(refs: &SearchRefs) -> TimeControl {
        let gt = &refs.search_params.game_time;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let total_time = clock / 1000; // Convert to seconds
        
        if total_time < 60 {
            TimeControl::Bullet     // < 1 minute
        } else if total_time < 300 {
            TimeControl::Blitz      // 5 minutes
        } else if total_time < 1800 {
            TimeControl::Rapid      // 30 minutes
        } else {
            TimeControl::Classical  // > 30 minutes
        }
    }

    // Emergency time management
    pub fn emergency_time_management(refs: &mut SearchRefs) -> bool {
        let gt = &refs.search_params.game_time;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let moves_to_go = Search::adaptive_moves_to_go(refs);
        
        // Emergency mode: less than 2 seconds per move
        if clock < (moves_to_go as u128) * EMERGENCY_TIME_THRESHOLD {
            refs.search_info.emergency_mode = true;
            refs.search_info.max_depth = EMERGENCY_MAX_DEPTH;
            true
        } else {
            refs.search_info.emergency_mode = false;
            refs.search_info.max_depth = MAX_PLY;
            false
        }
    }

    // Assess move quality based on root analysis
    pub fn assess_move_quality(refs: &SearchRefs) -> MoveQuality {
        if refs.search_info.root_analysis.is_empty() {
            return MoveQuality::Acceptable;
        }
        
        let analysis = &refs.search_info.root_analysis;
        let best_eval = analysis[0].eval;
        let second_eval = if analysis.len() > 1 { analysis[1].eval } else { best_eval };
        let eval_diff = (best_eval - second_eval).abs();
        
        // Check if we're in check
        let in_check = refs.board.in_check();
        
        match (eval_diff, in_check) {
            (0..30, false) => MoveQuality::Acceptable,  // Close evaluation
            (30..100, false) => MoveQuality::Good,      // Clear advantage
            (100.., false) => MoveQuality::Excellent,   // Large advantage
            (0..50, true) => MoveQuality::Critical,     // In check, close evaluation
            (50.., true) => MoveQuality::Poor,          // In check, large difference
            _ => MoveQuality::Acceptable,
        }
    }

    // Quality-based time allocation
    pub fn quality_based_time_allocation(refs: &SearchRefs) -> u128 {
        let base_time = Search::calculate_time_slice(refs);
        let move_quality = Search::assess_move_quality(refs);
        
        match move_quality {
            MoveQuality::Excellent => base_time * 70 / 100,  // 30% less time
            MoveQuality::Good => base_time * 85 / 100,       // 15% less time
            MoveQuality::Acceptable => base_time,            // Normal time
            MoveQuality::Poor => base_time * 120 / 100,      // 20% more time
            MoveQuality::Critical => base_time * 150 / 100,  // 50% more time
        }
    }

    // Time control specific allocation
    pub fn time_control_specific_allocation(refs: &SearchRefs) -> u128 {
        let base_time = Search::calculate_time_slice(refs);
        let time_control = Search::classify_time_control(refs);
        
        match time_control {
            TimeControl::Bullet => base_time * 80 / 100,     // 20% less time
            TimeControl::Blitz => base_time * 90 / 100,      // 10% less time
            TimeControl::Rapid => base_time,                 // Normal time
            TimeControl::Classical => base_time * 110 / 100, // 10% more time
        }
    }

    // Enhanced time slice calculation with all improvements
    pub fn calculate_enhanced_time_slice(refs: &SearchRefs) -> u128 {
        let base_time = Search::calculate_time_slice(refs);
        
        // Apply emergency time management
        if refs.search_info.emergency_mode {
            return (base_time as f64 * EMERGENCY_TIME_FACTOR) as u128;
        }
        
        // Apply quality-based allocation
        let quality_time = Search::quality_based_time_allocation(refs);
        
        // Apply time control specific allocation
        let control_time = Search::time_control_specific_allocation(refs);
        
        // Use the more conservative of the two
        std::cmp::min(quality_time, control_time)
    }

    // Update time statistics
    pub fn update_time_statistics(refs: &mut SearchRefs, time_used: u128, success: bool) {
        let phase = Search::determine_game_phase(refs);
        refs.search_info.time_stats.update(time_used, success, phase);
    }

    // Display time management statistics
    pub fn display_time_statistics(refs: &SearchRefs) -> String {
        let stats = &refs.search_info.time_stats;
        let phase = Search::determine_game_phase(refs);
        let time_control = Search::classify_time_control(refs);
        let emergency = refs.search_info.emergency_mode;
        
        format!(
            "Time Stats: Total={}, Success={}, Rate={:.1}%, Avg={}ms, Phase={:?}, Control={:?}, Emergency={}",
            stats.total_moves,
            stats.successful_allocations,
            stats.success_rate() * 100.0,
            stats.average_time_per_move,
            phase,
            time_control,
            emergency
        )
    }
}
