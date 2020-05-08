use crate::board::defs::{Pieces, ALL_SQUARES, PIECE_NAME};
use crate::defs::{Bitboard, Piece, EMPTY};
use crate::extra::print;
use crate::movegen::{
    attackboards, blockerboards, magics::Magics, masks, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE,
};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

/*
    TODO: even more comments here.
    Finding magic numbers step by step.
    fail_low and fail_high: check if the generated index is within the expected offset.
    If this is not the case, there's a bug somewhere in this code, or in get_index().
*/
#[allow(dead_code)]
pub fn find_magics(piece: Piece) {
    assert!(
        piece == Pieces::ROOK || piece == Pieces::BISHOP,
        "Illegal piece: {}",
        piece
    );

    let is_rook = piece == Pieces::ROOK;
    let mut rook_table: Vec<Bitboard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<Bitboard> = vec![EMPTY; BISHOP_TABLE_SIZE];
    let mut random = SmallRng::from_entropy();
    let mut offset = 0;
    let mut total_permutations = 0;

    println!("Finding magics for: {}", PIECE_NAME[piece]);
    for sq in ALL_SQUARES {
        // Create the mask for either the rook or bishop.
        let mask = if is_rook {
            masks::create_rook_mask(sq)
        } else {
            masks::create_bishop_mask(sq)
        };

        // Precalculate needed values.
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let end = offset + permutations - 1; // End index in the attack table.

        // Create blocker board for the current mask.
        let blocker_boards = blockerboards::create_blocker_boards(mask);

        // Create attack board for the current square/blocker combo (either rook or bishop).
        let attack_boards = if is_rook {
            attackboards::create_rook_attack_boards(sq, &blocker_boards)
        } else {
            attackboards::create_bishop_attack_boards(sq, &blocker_boards)
        };

        // Done calculating needed data. Create a new magic.
        let mut try_this: Magics = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
        let mut attempts = 0; // Track needed attempts to find the magic.

        // Set up the new magic with the values we already know.
        try_this.mask = mask;
        try_this.shift = (64 - bits) as u8;
        try_this.offset = offset;
        total_permutations += permutations;

        // Start finding a magic that works for this square, for all permuations.
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

    // Check if everything is correct.
    assert!(
        (total_permutations as usize)
            == if is_rook {
                ROOK_TABLE_SIZE
            } else {
                BISHOP_TABLE_SIZE
            },
        "Creating magics failed. Permutations were skipped."
    );
}
