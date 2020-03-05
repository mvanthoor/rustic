// TODO: Some more comments.
extern crate rand;

use super::blockatt::{create_blocker_boards, create_rook_attack_boards};
use super::masks::create_rook_mask;
use super::{EMPTY, ROOK_TABLE_SIZE};
use crate::defines::{Bitboard, Piece, ALL_SQUARES, BISHOP, ROOK, SQUARE_NAME};
use crate::print;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone)]
pub struct Magics {
    pub mask: Bitboard,
    pub shift: u8,
    pub offset: u64,
    pub magic: u64,
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            mask: 0,
            shift: 0,
            offset: 0,
            magic: 0,
        }
    }
}

impl Magics {
    pub fn get_index(&self, occupancy: Bitboard) -> usize {
        let blockerboard = occupancy & self.mask;
        ((blockerboard.wrapping_mul(self.magic) >> self.shift) + self.offset) as usize
    }
}

/*
    Explanation for fail_low and fail_high:
    Only try and add if the spot in the table is empty.
    First check if the index is actually legal.
    Can't be smaller than the table's current offset,
    because it would overwrite earlier entries. It can't
    be higher than offset+permutations-1, because it would
    be skipping table entries. If this happens, there is an
    error somewhere in this function, or in get_index().
*/
pub fn find_magics(piece: Piece) {
    assert!(piece == ROOK || piece == BISHOP, "Illegal piece: {}", piece);
    let mut rook_table: [Bitboard; ROOK_TABLE_SIZE] = [0; ROOK_TABLE_SIZE];
    let mut random = SmallRng::from_entropy();
    let mut offset = 0;
    let mut total_permutations = 0;

    for sq in ALL_SQUARES {
        let mask = create_rook_mask(sq);
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let blocker_boards = create_blocker_boards(mask); // All blocker boards for this mask.
        let attack_boards = create_rook_attack_boards(sq, &blocker_boards); // All attack boards.
        let mut try_this: Magics = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
        let mut low: usize = offset as usize; // Track the lowest generated magic index.
        let mut high: usize = offset as usize; // Track the highest generated magic index.
        let mut attempts = 0; // Track needed attempts to find the magic.

        // Set up the new magic.
        try_this.mask = mask;
        try_this.shift = (64 - bits) as u8;
        try_this.offset = offset;
        total_permutations += permutations;
        while !found {
            attempts += 1; // Next attempt to find magic.
            found = true; // Assume this new magic will work.
            try_this.magic = random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>();
            for i in 0..=(permutations - 1) {
                let next = i as usize; // Get the next blocker/attack board
                let index = try_this.get_index(blocker_boards[next]); // Magical index...
                if rook_table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > offset as usize + permutations as usize - 1;
                    if fail_low || fail_high {
                        panic!("Illegal index: {}", index);
                    }
                    rook_table[index] = attack_boards[next];
                    low = if index < low { index } else { low };
                    high = if index > high { index } else { high };
                } else {
                    for i in offset..=(offset + permutations - 1) {
                        rook_table[i as usize] = EMPTY;
                    }
                    low = offset as usize;
                    high = offset as usize;
                    found = false;
                    break;
                }
            }
        }
        print::found_magic(sq, try_this, offset, permutations, low, high, attempts);
        offset += permutations; // Set offset for next square.
    }
    print::magic_the_gathering_finished(total_permutations, ROOK_TABLE_SIZE);
}
