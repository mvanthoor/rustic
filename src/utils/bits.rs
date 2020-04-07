use crate::defs::Bitboard;

// Get the next set bit from a bitboard and unset it.
// When given a piece bitboard, this provides the
// location/square of the next piece of that type.
pub fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1u64 << location;
    location as u8
}

// Clear the given bit in the bitboard.
pub fn clear_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard &= !(1u64 << bit);
}

// Set the given bit in the bitboard.
pub fn set_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard |= 1u64 << bit;
}
