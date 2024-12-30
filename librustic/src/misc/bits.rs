use crate::defs::{Bitboard, Square};

// Get the next set bit from a bitboard and unset it. When given a piece
// bitboard, this provides the location/square of the next piece of that type.
pub fn next(bitboard: &mut Bitboard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}
