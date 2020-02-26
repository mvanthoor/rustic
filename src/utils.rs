use crate::defines::Bitboard;

pub fn create_bb_files() -> [Bitboard; 8] {
    // 0x0101_0101_0101_0101 is bits set for A1, A2...
    let mut bb_files: [Bitboard; 8] = [0; 8];
    for (i, file) in bb_files.iter_mut().enumerate() {
        *file = 0x0101_0101_0101_0101 << i;
    }
    bb_files
}

pub fn create_bb_ranks() -> [Bitboard; 8] {
    // 0xFF is all bits set for Rank 1; entire first byte of u64.
    let mut bb_ranks: [Bitboard; 8] = [0; 8];
    for (i, rank) in bb_ranks.iter_mut().enumerate() {
        *rank = 0xFF << (i * 8);
    }
    bb_ranks
}
