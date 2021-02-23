/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2021, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

use crate::movegen::MoveGenerator;
use crate::{
    board::defs::{Pieces, RangeOf, PIECE_NAME, SQUARE_NAME},
    defs::{Bitboard, Piece, Square, EMPTY},
    movegen::{defs::Magic, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE},
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;

// The find_magics function can be used by compiling the "wizardry" module
// into the engine, and then adding the "-w" option on the command line.
// This function generates magic numbers for the rooks and bishops. A queen
// is a combination of a rook and a bishop, so she does not have her own
// magic numbers.
pub fn find_magics(piece: Piece) {
    // First check if we're actually dealing with a rook or a bishop.
    let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
    assert!(ok, "Illegal piece: {}", piece);

    // Create working variables.
    let is_rook = piece == Pieces::ROOK;
    let mut rook_table: Vec<Bitboard> = vec![EMPTY; ROOK_TABLE_SIZE];
    let mut bishop_table: Vec<Bitboard> = vec![EMPTY; BISHOP_TABLE_SIZE];
    let mut random = ChaChaRng::from_entropy();
    let mut offset = 0;

    println!("Finding magics for: {}", PIECE_NAME[piece]);
    for sq in RangeOf::SQUARES {
        // Create the mask for either the rook or bishop.
        let r_mask = MoveGenerator::rook_mask(sq);
        let b_mask = MoveGenerator::bishop_mask(sq);
        let mask = if is_rook { r_mask } else { b_mask };

        // Precalculate needed values.
        let bits = mask.count_ones(); // Number of set bits in the mask
        let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
        let end = offset + permutations - 1; // End index in the attack table.

        // Create blocker boards for the current mask.
        let blocker_boards = MoveGenerator::blocker_boards(mask);

        // Create attack boards for the current square/blocker combo (either
        // rook or bishop).
        let r_ab = MoveGenerator::rook_attack_boards(sq, &blocker_boards);
        let b_ab = MoveGenerator::bishop_attack_boards(sq, &blocker_boards);
        let attack_boards = if is_rook { r_ab } else { b_ab };

        // Done calculating needed data. Create a new magic.
        let mut try_this: Magic = Default::default(); // New magic
        let mut found = false; // While loop breaker if magic works;
        let mut attempts = 0; // Track needed attempts to find the magic.

        // Set up the new magic with the values we already know.
        try_this.mask = mask;
        try_this.shift = (64 - bits) as u8;
        try_this.offset = offset;

        // Start finding a magic that works for this square, for all permuations.
        while !found {
            attempts += 1; // Next attempt to find magic.
            found = true; // Assume this new magic will work.

            // Create a random magic number to test.
            try_this.nr = random.gen::<u64>() & random.gen::<u64>() & random.gen::<u64>();

            // Now try all possible permutations of blocker boards on this square.
            for i in 0..permutations {
                // Get the index where the magic for this blocker board
                // needs to go (if it works.)
                let next = i as usize;
                let index = try_this.get_index(blocker_boards[next]);

                // Use either a reference to the rook or bishop table.
                let r_table = &mut rook_table[..];
                let b_table = &mut bishop_table[..];
                let table: &mut [Bitboard] = if is_rook { r_table } else { b_table };

                // If the table at this index is empty...
                if table[index] == EMPTY {
                    // Check if we're within the expected range
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error.");

                    // We found a working magic.
                    table[index] = attack_boards[next];
                } else {
                    // The table at this index is not empty. We have a
                    // collision. This magic doesn't work. Wipe the part of
                    // the table we are working with. Try a new number.
                    for wipe_index in offset..=end {
                        table[wipe_index as usize] = EMPTY;
                    }
                    found = false;
                    break;
                }
            }
        }

        // We got out of the loop and found a random magic number that can
        // index all the attack boards for a rook/bishop for a single
        // square without a collision. Report this number.
        found_magic(sq, try_this, offset, end, attempts);

        // Set table offset for next magic.
        offset += permutations;
    }

    // Check if the entire table is correct. The offset should be equal to
    // the size of the table. If it isn't, we skipped permuations and thus
    // have some sort of error in our code above.
    let r_ts = ROOK_TABLE_SIZE as u64;
    let b_ts = BISHOP_TABLE_SIZE as u64;
    let expected = if is_rook { r_ts } else { b_ts };
    const ERROR: &str = "Creating magics failed. Permutations were skipped.";

    assert!(offset == expected, ERROR);
}

// Print the magic number.
fn found_magic(square: Square, m: Magic, offset: u64, end: u64, attempts: u64) {
    println!(
        "{}: {:24}u64 (offset: {:6}, end: {:6}, attempts: {})",
        SQUARE_NAME[square], m.nr, offset, end, attempts
    );
}
