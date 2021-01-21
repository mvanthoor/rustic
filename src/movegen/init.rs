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

use super::{
    magics::{Magic, BISHOP_MAGIC_NRS, ROOK_MAGIC_NRS},
    MoveGenerator, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE,
};
use crate::{
    board::defs::{Files, Pieces, RangeOf, Ranks, BB_FILES, BB_RANKS, BB_SQUARES},
    defs::{Piece, Sides, EMPTY},
};

impl MoveGenerator {
    /**
     * Generate all the possible king moves for each square.
     * Exampe: Generate a bitboard for the square the king is on.
     * Generate a move to Up-Left, if the king is not on the A file, and not on the last rank.
     * Generate a move to Up, if the king is not on the last rank.
     * ... and so on. All the moves are combined in the bb_move bitboard.
     * Do this for each square.
     */
    #[rustfmt::skip]
    pub fn init_king(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let bb_moves =
            (bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R8]) << 7
            | (bb_square & !BB_RANKS[Ranks::R8]) << 8
            | (bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8]) << 9
            | (bb_square & !BB_FILES[Files::H]) << 1
            | (bb_square & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1]) >> 7
            | (bb_square & !BB_RANKS[Ranks::R1]) >> 8
            | (bb_square & !BB_FILES[Files::A] & !BB_RANKS[Ranks::R1]) >> 9
            | (bb_square & !BB_FILES[Files::A]) >> 1;
            self.king[sq as usize] = bb_moves;
        }
    }

    /**
     * Generate all the possible knight moves for each square. Works
     * exactly the same as the king move generation, but obviously,
     * it uses the directions and file/rank restrictions for a knight
     * instead of those for the king.
     */
    #[rustfmt::skip]
    pub fn init_knight(&mut self) {
    for sq in RangeOf::SQUARES {
        let bb_square = BB_SQUARES[sq];
        let bb_moves =
            (bb_square & !BB_RANKS[Ranks::R8] & !BB_RANKS[Ranks::R7] & !BB_FILES[Files::A]) << 15
            | (bb_square & !BB_RANKS[Ranks::R8] & !BB_RANKS[Ranks::R7] & !BB_FILES[Files::H]) << 17
            | (bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R8]) << 6
            | (bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R8]) << 10
            | (bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::A]) >> 17
            | (bb_square & !BB_RANKS[Ranks::R1] & !BB_RANKS[Ranks::R2] & !BB_FILES[Files::H]) >> 15
            | (bb_square & !BB_FILES[Files::A] & !BB_FILES[Files::B] & !BB_RANKS[Ranks::R1]) >> 10
            | (bb_square & !BB_FILES[Files::G] & !BB_FILES[Files::H] & !BB_RANKS[Ranks::R1]) >> 6;
        self.knight[sq as usize] = bb_moves;
    }
}

    /**
     * Generate all the possible pawn capture targets for each square.
     * Same again... generate a move to up-left/up-right, or down-left down-right
     * if the location of the pawn makes that move possible.
     */
    pub fn init_pawns(&mut self) {
        for sq in RangeOf::SQUARES {
            let bb_square = BB_SQUARES[sq];
            let w = (bb_square & !BB_FILES[Files::A]) << 7 | (bb_square & !BB_FILES[Files::H]) << 9;
            let b = (bb_square & !BB_FILES[Files::A]) >> 9 | (bb_square & !BB_FILES[Files::H]) >> 7;
            self.pawns[Sides::WHITE][sq as usize] = w;
            self.pawns[Sides::BLACK][sq as usize] = b;
        }
    }

    /** This is the main part of the module: it indexes all of the atack boards
     * using the magic numbers from the "magics" module. This builds an attack database
     * for sliding pieces, for each square and each combination of blocker boards. A
     * blocker is a piece that is "in the way", causing the slider to not be able to
     * 'see' beyond that piece.
     * Step by step description:
     * This function is used for generating and indexing the rook and bishop attacks.
     * Set the viewpoint to either rook or bishop, using is_rook.
     * We need to keep track of the offset within the attack table; each square starts at a
     * certain offset. We also keep track of the total permutations calculated, for checking
     * purposes.
     * Then we start the main loop: for each square in all squares...
     * Get the piece mask for that square (where it can move on an empty board)
     * Calculate the number of permutations that this mask is going to generate.
     * Calculate the end offset of this square in the attack table.
     * Generate all the blocker borads for this piece on this square.
     * Generate all the attacker boards for this piece on this square.
     * Create e new magic.
     * Fill the new magic with the calculated information, but pick the magic number
     * for this square from the pre-generated magics in the magics module. See the function
     * "find_magics()" in that module. (That function is very similar to this one.)
     * We're still on the same square...
     * Now start iterating through the permutations of the blocker boards.
     * Calculte the magic index in the attack table, and then put the *ATTACK* board there.
     * (Every blocker board has only one attack board.)
     * Obviously use either the rook or the bishop attack table.
     * There is also some code to check if the index is within the expected offset,
     * and that the slot found in the attack table is actually empty. If one of these is not true,
     * then there is something wrong with the pre-generated magic numbers.
     * If everything is OK (the program has not paniced), insert the magic into either the rook
     * or bishop magics table. Then go to the next permutation.
     * After the permutations and square loops are done, do a final check. This is a perfect hash
     * (every spot in the table is filled, with no collisions), so we expect the number of handled
     * permutations to be exaclty the same as the table's length. If not, there's something wrong.
     *
     * Now, it's possible to build a blocker board for any slider
     * on any board (slider_mask & board_occupancy), and then use this blocker board and the magic
     * information to calculate the index of the attack board for this piece within the attack
     * table.
     */
    pub fn init_magics(&mut self, piece: Piece) {
        let ok = piece == Pieces::ROOK || piece == Pieces::BISHOP;
        assert!(ok, "Illegal piece: {}", piece);

        let is_rook = piece == Pieces::ROOK;
        let mut offset = 0;

        for sq in RangeOf::SQUARES {
            let r_mask = MoveGenerator::rook_mask(sq);
            let b_mask = MoveGenerator::bishop_mask(sq);
            let mask = if is_rook { r_mask } else { b_mask };

            let bits = mask.count_ones(); // Number of set bits in the mask
            let permutations = 2u64.pow(bits); // Number of blocker boards to be indexed.
            let end = offset + permutations - 1; // End point in the attack table.
            let blocker_boards = MoveGenerator::blocker_boards(mask);

            let r_ab = MoveGenerator::rook_attack_boards(sq, &blocker_boards);
            let b_ab = MoveGenerator::bishop_attack_boards(sq, &blocker_boards);
            let attack_boards = if is_rook { r_ab } else { b_ab };

            let mut magic: Magic = Default::default();
            let r_magic_nr = ROOK_MAGIC_NRS[sq as usize];
            let b_magic_nr = BISHOP_MAGIC_NRS[sq as usize];

            magic.mask = mask;
            magic.shift = (64 - bits) as u8;
            magic.offset = offset;
            magic.nr = if is_rook { r_magic_nr } else { b_magic_nr };

            for i in 0..permutations {
                let next = i as usize;
                let index = magic.get_index(blocker_boards[next]);
                let rook_table = &mut self.rook[..];
                let bishop_table = &mut self.bishop[..];
                let table = if is_rook { rook_table } else { bishop_table };

                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error. Error in Magics.");
                    table[index] = attack_boards[next];
                } else {
                    panic!("Attack table index not empty. Error in Magics.");
                }
            }

            // No failures  during indexing. Store this magic.
            if is_rook {
                self.rook_magics[sq as usize] = magic;
            } else {
                self.bishop_magics[sq as usize] = magic;
            }

            // Do the next magic.
            offset += permutations;
        }

        // All permutations (blocker boards) should have been indexed.
        let r_ts = ROOK_TABLE_SIZE as u64;
        let b_ts = BISHOP_TABLE_SIZE as u64;
        let expectation = if is_rook { r_ts } else { b_ts };
        const ERROR: &str = "Initializing magics failed. Check magic numbers.";

        assert!(offset == expectation, ERROR);
    }
}
