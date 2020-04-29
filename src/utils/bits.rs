use crate::defs::{Bitboard, Square};

// Get the next set bit from a bitboard and unset it. When given a piece
// bitboard, this provides the location/square of the next piece of that type.
pub fn next(bitboard: &mut Bitboard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}

// Clear the given bit in the bitboard.
#[allow(dead_code)]
pub fn clear_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard &= !(1u64 << bit);
}

// Set the given bit in the bitboard.
#[allow(dead_code)]
pub fn set_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard |= 1u64 << bit;
}
