use super::{defs::Location, Board};
use crate::{
    board::defs::Ranks,
    defs::{Side, Sides, Square},
};

impl Board {
    // Compute on which file and rank a given square is.
    pub fn square_on_file_rank(square: Square) -> Location {
        let file = (square % 8) as u8; // square mod 8
        let rank = (square / 8) as u8; // square div 8
        (file, rank)
    }

    // Compute if a given square is or isn't on the given rank.
    pub fn square_on_rank(square: Square, rank: Square) -> bool {
        let start = (rank) * 8;
        let end = start + 7;
        (start..=end).contains(&square)
    }

    pub fn fourth_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R4
        } else {
            Ranks::R5
        }
    }

    pub fn promotion_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R8
        } else {
            Ranks::R1
        }
    }
}
