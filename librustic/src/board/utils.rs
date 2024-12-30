use crate::{
    board::defs::{Location, Ranks},
    board::Board,
    defs::{Side, Sides, Square},
    evaluation::defs::EvalParams,
};

impl Board {
    // Compute on which file and rank a given square is.
    pub fn square_on_file_rank(square: Square) -> Location {
        let file = (square % 8) as u8; // square mod 8
        let rank = (square / 8) as u8; // square div 8

        Location { file, rank }
    }

    // Compute if a given square is or isn't on the given rank.
    pub fn square_on_rank(square: Square, rank: Square) -> bool {
        let start = (rank) * 8;
        let end = start + 7;
        (start..=end).contains(&square)
    }

    pub const fn fourth_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R4
        } else {
            Ranks::R5
        }
    }

    pub const fn promotion_rank(side: Side) -> usize {
        if side == Sides::WHITE {
            Ranks::R8
        } else {
            Ranks::R1
        }
    }

    pub fn is_white_square(square: Square) -> bool {
        let rank = square / 8;
        let even_square = (square & 1) == 0;
        let even_rank = (rank & 1) == 0;

        (even_rank && !even_square) || (!even_rank && even_square)
    }

    pub const fn pawn_direction(side: Side) -> i8 {
        const UP: i8 = 8;
        const DOWN: i8 = -8;

        if side == Sides::WHITE {
            UP
        } else {
            DOWN
        }
    }

    pub const fn flip(side: Side, square: Square) -> usize {
        if side == Sides::WHITE {
            EvalParams::FLIP[square]
        } else {
            square
        }
    }

    pub fn is_white_to_move(&self) -> bool {
        self.us() == Sides::WHITE
    }
}
