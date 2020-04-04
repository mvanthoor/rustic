use crate::defs::Bitboard;

/**
 * Get the next set bit from a bitboard.
 * This is used to get the square locations of each piece.
 * For example, the PAWNS bitboard could have 8 bits set.
 * This function returns the index (= square) from that bitboard,
 * and then removes the bit. All pieces/squares (whatver is in
 * the bitboard) have been handled when the bitboard becomes 0.
 * */
pub fn next(bitboard: &mut Bitboard) -> u8 {
    let location = bitboard.trailing_zeros();
    *bitboard ^= 1u64 << location;
    location as u8
}

pub fn clear_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard &= !(1u64 << bit);
}

pub fn set_bit(bitboard: &mut Bitboard, bit: u8) {
    *bitboard |= 1u64 << bit;
}