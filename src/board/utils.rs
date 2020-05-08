use super::defs::Location;
use crate::defs::Square;

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
