use super::{defs::SearchRefs, Search};
use crate::defs::Sides;

pub const SAFEGUARD: f64 = 100.0; // msecs
const GAME_LENGTH: usize = 30; // moves
const MAX_USAGE: f64 = 0.8; // percentage
const NO_TIME: u128 = 0;

impl Search {
    // Determine if allocated search time has been used up.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        refs.search_info.timer_elapsed() > refs.search_info.allocated_time
    }

    // Calculates the time the engine allocates for searching a single
    // move. This depends on the number of moves still to go in the game.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs) as f64;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime } as f64;
        let increment = if white { gt.winc } else { gt.binc } as f64;
        let base_time = clock - SAFEGUARD;

        // return a time slice.
        if base_time <= 0.0 {
            if increment > 0.0 {
                (increment * MAX_USAGE).round() as u128
            } else {
                NO_TIME
            }
        } else {
            ((base_time * MAX_USAGE / mtg) + increment).round() as u128
        }
    }

    // Here we try to come up with some sort of sensible value for "moves
    // to go", if this value is not supplied.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // Default to GAME_LENGTH moves if movestogo was not provided
        refs.search_params
            .game_time
            .moves_to_go
            .unwrap_or(GAME_LENGTH)
    }
}
