extern crate rand;

use super::blockatt::{create_blocker_boards, create_rook_attack_boards};
use super::masks::create_rook_mask;
use super::ROOK_TABLE_SIZE;
use crate::defines::{Bitboard, Piece, SQUARE_NAME};
use crate::print;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Copy, Clone)]
pub struct Magics {
    pub mask: Bitboard,
    pub shift: u8,
    pub magic: u64,
    pub offset: u32,
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            mask: 0,
            shift: 0,
            magic: 0,
            offset: 0,
        }
    }
}

impl Magics {
    pub fn index(&self, occupancy: Bitboard) -> usize {
        let blockerboard = occupancy & self.mask;
        let i = (blockerboard.wrapping_mul(self.magic) >> self.shift) as usize;
        i
    }
}

pub fn find_magics(sq: u8, piece: Piece) {
    let mut rook_table: [Bitboard; ROOK_TABLE_SIZE] = [0; ROOK_TABLE_SIZE];
    let mut previous: Magics = Default::default();
    let mut current: Magics = Default::default();
    let mask = create_rook_mask(sq);
    let bits = mask.count_ones();
    let permutations = 2u64.pow(bits);
    let blocker_boards = create_blocker_boards(mask);
    let attack_boards = create_rook_attack_boards(sq, &blocker_boards);
    let mut found = false;
    let mut hi: usize = 0;

    // Set up the current magic
    current.mask = mask;
    current.shift = (64 - bits) as u8;
    while !found {
        // Assume it will be found in this run
        found = true;
        current.magic = random_few_bits();
        for i in 0..permutations {
            let index = current.index(blocker_boards[i as usize]);
            if rook_table[index] == 0 {
                rook_table[index] = attack_boards[index];
                hi = if index > hi { index } else { hi };
            } else {
                rook_table = [0; ROOK_TABLE_SIZE];
                hi = 0;
                found = false;
                break;
            }
        }
    }

    println!(
        "Magic found for {} - {} - hi: {}",
        SQUARE_NAME[sq as usize], current.magic, hi
    );
}

/**
 * It is required for a magic number to have a low number of set bits.
 * This is accomplished by AND-ing three random numbers; this will
 * cancel out any bits that don't occur in all three numbers. By
 * canceling out bits, the bit-count is lowered.
 */
fn random_few_bits() -> u64 {
    let mut random = SmallRng::from_entropy();

    random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>()
}
