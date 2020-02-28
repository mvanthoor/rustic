use crate::defines::*;

// Piece location: (file, rank)
pub type Location = (u8, u8);

const UP: bool = true;
const ACROSS: bool = false;
const NORMAL: bool = true;
const ANTI: bool = false;

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

pub fn square_on_file_rank(square: u8) -> Location {
    let file = square % 8; // square mod 8
    let rank = square / 8; // square div 8
    (file, rank)
}

pub fn square_on_rank(square: u8, rank: u8) -> bool {
    let start = (rank) * 8;
    let end = start + 7;
    (start..=end).contains(&square)
}

pub fn create_bb_diagonals() -> [Bitboard; 15] {
    let mut bb_diagonals: [Bitboard; 15] = [0; 15];
    let mut list: Vec<Bitboard> = Vec::new();

    diagonals(RANK_1 as u8, RANK_8 as u8, UP, &mut list);
    diagonals(FILE_B as u8, FILE_H as u8, ACROSS, &mut list);

    for (i, d) in list.iter().enumerate() {
        bb_diagonals[i] = *d;
    }
    bb_diagonals
}

pub fn diagonals(start: u8, end: u8, direction: bool, diagonals: &mut Vec<Bitboard>) {
    for x in start..=end {
        let current_square = if direction == UP {
            1u64 << (x * 8) // Up the ranks
        } else {
            1u64 << x // Across the files
        };
        let steps_to_take = 7 - x;
        let mut bitboard = 0;
        for step in 0..=steps_to_take {
            let landing_square = current_square << (9 * step);
            bitboard ^= landing_square;
        }
        diagonals.push(bitboard);
    }
}
