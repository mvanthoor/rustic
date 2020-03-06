/**
 * This module provicdes "Magics", for working with sliding pieces in the move generator.
*/
extern crate rand; // Random number generator for creating magics.
use super::blockatt::{
    create_bishop_attack_boards, create_blocker_boards, create_rook_attack_boards,
};
use super::masks::{create_bishop_mask, create_rook_mask};
use super::{BISHOP_TABLE_SIZE, EMPTY, ROOK_TABLE_SIZE};
use crate::defines::{Bitboard, Piece, ALL_SQUARES, BISHOP, ROOK};
use crate::print;
use crate::print::PIECE_NAME;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/**
 * Magics contain the following data:
 * mask: A Rook or Bishop mask for the square the magic belongs to.
 * shift: This number is needed to create the magic index. It's "64 - (nr. of bits set 1 in mask)"
 * offset: this is the offset in the attack table, where the attacks for the magic's square begin.
 * magic: the magic number itself, used to create the index.
*/
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

/**
 * get_index() is the actual function that gets the magic index into the attack table.
 * The attack table is a perfect hash. This means the following.
 * - A rook on A1 has 7 squares vertical and 7 squares horizontal movement.
 * - This is a total of 14 bits. However, if there are no pieces on A2-A6, or B1-G1, the rook
 *      can always see A8 and H1. This means that if there are no blockers on the file or rank,
 *      the rook can 'see' the square at the edge of the board. Therefore, the bits marking the
 *      edge of a ray are not counted. Thus, the rook on A1 has actually 12 bits set.
 * - These bits along the rank and file denote the possible position of blocking pieces.
 * - For 12 bits, there are 4096 possible configuration of blockers (2 to the power of 12).
 * - Thus, square A1 has 4096 blocker boards.
 * - The get_index() function receives a board occupancy when called.
 * - occupancy * self.mask (the mask for the piece on the square the magic belongs to) yields
 *      a blocker board.
 * - Each blocker board (configuration of blockers) goes with one attack board (the squares the)
 *      piece can actually attack). This attack board is in the attack table.
 * - The formula calculates WHERE in the attack table the blocker board is:
 *      (blockerboard * magic number) >> (64 - bits in mask) + offset
 * - For the rook on A1 the outcome will be an index of 0 - 4095:
 *      0 - 4095 because of 4096 possible blocker (and thus, attack board) permutations
 *      0 for offset, because A1 is the first square.
 * - So the index for a rook on B1 will start at 4096, and so on.
 * - The "magic number" is called magic, because it generates a UNIQUE index for each each
 *      attack board in the attack table, without any collisions; so the entire table is exactly
 *      filled. Therefore it's called a perfect hash.
 * - Finding the magics is a process of just trying random numbers, with the formula below, over
 * and over again until a number is found that generates unique indexes for all of the permutations
 * of blockers of the piece on a particular square. See the explanation for find_magics().
 */
impl Magics {
    pub fn get_index(&self, occupancy: Bitboard) -> usize {
        let blockerboard = occupancy & self.mask;
        ((blockerboard.wrapping_mul(self.magic) >> self.shift) + self.offset) as usize
    }
}

/*
    TODO: even more comments here.
    Finding magic numbers step by step.
    fail_low and fail_high: check if the generated index is within the expected offset.
    If this is not the case, there's a bug somewhere in this code, or in get_index().
*/
pub fn find_magics(piece: Piece) {
    assert!(piece == ROOK || piece == BISHOP, "Illegal piece: {}", 0);
    let is_rook = piece == ROOK;
    let mut rook_table: Vec<Bitboard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<Bitboard> = vec![EMPTY; BISHOP_TABLE_SIZE];
    let mut random = SmallRng::from_entropy();
    let mut offset = 0;
    let mut total_permutations = 0;

    println!("Finding magics for: {}", PIECE_NAME[piece]);
    for sq in ALL_SQUARES {
        let mask = if is_rook {
            create_rook_mask(sq)
        } else {
            create_bishop_mask(sq)
        };
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let end = offset + permutations - 1; // End point in the attack table.
        let blocker_boards = create_blocker_boards(mask);
        let attack_boards = if is_rook {
            create_rook_attack_boards(sq, &blocker_boards)
        } else {
            create_bishop_attack_boards(sq, &blocker_boards)
        };
        let mut try_this: Magics = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
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
            for i in 0..permutations {
                let next = i as usize;
                let index = try_this.get_index(blocker_boards[next]);
                let table: &mut [Bitboard] = if is_rook {
                    &mut rook_table[..]
                } else {
                    &mut bishop_table[..]
                };
                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error.");
                    table[index] = attack_boards[next];
                } else {
                    for i in offset..=end {
                        table[i as usize] = EMPTY;
                    }
                    found = false;
                    break;
                }
            }
        }
        print::found_magic(sq, try_this, offset, end, attempts);
        offset += permutations; // Set offset for next square.
    }
    let expected = if is_rook {
        ROOK_TABLE_SIZE
    } else {
        BISHOP_TABLE_SIZE
    };
    assert!((total_permutations as usize) == expected);
}
