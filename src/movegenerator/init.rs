/** Magics.rs implements the generation of magic bitboards for sliders, and the attack tables
 * for the non-slider squares. After all the bitboards are generated, using the three functions
 * "get_non_slider_attacks()", "get_slider_attacks()" and "get_pawn_attacks()" will "magically"
 * return all the possible attacks from the given square on the current board.
 * All possible moves for all pieces on all board squares (taking into account all possible
 * combination of blockers for the sliders) are calculated at the start of the engine, which
 * saves tremendous amounts of time in the move generator.
 */
extern crate rand;

use super::blockatt::{
    create_bishop_attack_boards, create_blocker_boards, create_rook_attack_boards,
};
use super::magics::{Magics, BISHOP_MAGICS, ROOK_MAGICS};
use super::masks::{create_bishop_mask, create_rook_mask};
use crate::definitions::{
    Bitboard, Piece, Side, ALL_SQUARES, BISHOP, BLACK, FILE_A, FILE_B, FILE_G, FILE_H, KING,
    KNIGHT, NR_OF_SQUARES, PAWN_SQUARES, QUEEN, RANK_1, RANK_2, RANK_7, RANK_8, ROOK, WHITE,
};
use crate::utils::{create_bb_files, create_bb_ranks};

const WHITE_BLACK: usize = 2;
const NSQ: usize = NR_OF_SQUARES as usize;
pub const EMPTY: Bitboard = 0;
pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

pub type BlockerBoards = Vec<Bitboard>;
pub type AttackBoards = Vec<Bitboard>;

/**
 * The struct "Magics" will hold all of the attack tables for each piece on each square.
 * The _rook and _bishop arrays hold the attack tables for the sliders. _rook_info and
 * _bishop_info hold the magic information, to get the correct attack board from the
 * respective attack table and return it. These tables and info are initialized in the
 * init_magics() function.
*/
pub struct Movements {
    _king: [Bitboard; NSQ],
    _knight: [Bitboard; NSQ],
    _pawns: [[Bitboard; NSQ]; WHITE_BLACK],
    _rook: Vec<Bitboard>,
    _bishop: Vec<Bitboard>,
    _rook_magics: [Magics; NSQ],
    _bishop_magics: [Magics; NSQ],
}

impl Default for Movements {
    fn default() -> Movements {
        let magics: Magics = Default::default();
        Movements {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
            _rook: vec![EMPTY; ROOK_TABLE_SIZE],
            _bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            _rook_magics: [magics; NSQ],
            _bishop_magics: [magics; NSQ],
        }
    }
}

impl Movements {
    pub fn initialize(&mut self) {
        let files = create_bb_files();
        let ranks = create_bb_ranks();

        self.init_king(&files, &ranks);
        self.init_knight(&files, &ranks);
        self.init_pawns(&files);
        self.init_magics(ROOK);
        self.init_magics(BISHOP);
    }

    /** Return non-slider (King, Knight) attacks for the given square. */
    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        match piece {
            KING => self._king[square as usize],
            KNIGHT => self._knight[square as usize],
            _ => 0,
        }
    }

    /** Return slider attacsk for Rook, Bishop and Queen using Magic. */
    pub fn get_slider_attacks(&self, piece: Piece, square: u8, occupancy: Bitboard) -> Bitboard {
        match piece {
            ROOK => {
                let index = self._rook_magics[square as usize].get_index(occupancy);
                self._rook[index]
            }
            BISHOP => {
                let index = self._bishop_magics[square as usize].get_index(occupancy);
                self._bishop[index]
            }
            QUEEN => {
                let r_index = self._rook_magics[square as usize].get_index(occupancy);
                let b_index = self._bishop_magics[square as usize].get_index(occupancy);
                self._rook[r_index] ^ self._bishop[b_index]
            }
            _ => 0,
        }
    }

    /** Return pawn attacks for the given square. */
    pub fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        self._pawns[side][square as usize]
    }

    /**
     * Generate all the possible king moves for each square.
     * Exampe: Generate a bitboard for the square the king is on.
     * Generate a move to Up-Left, if the king is not on the A file, and not on the last rank.
     * Generate a move to Up, if the king is not on the last rank.
     * ... and so on. All the moves are combined in the bb_move bitboard.
     * Do this for each square.
     */
    fn init_king(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let bb_square = 1u64 << sq;
            let bb_moves = (bb_square & !files[FILE_A] & !ranks[RANK_8]) << 7
                | (bb_square & !ranks[RANK_8]) << 8
                | (bb_square & !files[FILE_H] & !ranks[RANK_8]) << 9
                | (bb_square & !files[FILE_H]) << 1
                | (bb_square & !files[FILE_H] & !ranks[RANK_1]) >> 7
                | (bb_square & !ranks[RANK_1]) >> 8
                | (bb_square & !files[FILE_A] & !ranks[RANK_1]) >> 9
                | (bb_square & !files[FILE_A]) >> 1;
            self._king[sq as usize] = bb_moves;
        }
    }

    /**
     * Generate all the possible knight moves for each square. Works
     * exactly the same as the king move generation, but obviously,
     * it uses the directions and file/rank restrictions for a knight
     * instead of those for the king.
     */
    fn init_knight(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let bb_square = 1u64 << sq;
            let bb_moves = (bb_square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_A]) << 15
                | (bb_square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_H]) << 17
                | (bb_square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_8]) << 6
                | (bb_square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_8]) << 10
                | (bb_square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_A]) >> 17
                | (bb_square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_H]) >> 15
                | (bb_square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_1]) >> 10
                | (bb_square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_1]) >> 6;
            self._knight[sq as usize] = bb_moves;
        }
    }

    /**
     * Generate all the possible pawn capture targets for each square.
     * Same again... generate a move to up-left/up-right, or down-left down-right
     * if the location of the pawn makes that move possible.
     */
    fn init_pawns(&mut self, files: &[Bitboard; 8]) {
        for sq in PAWN_SQUARES {
            let bb_square = 1u64 << sq;
            let w = (bb_square & !files[FILE_A]) << 7 | (bb_square & !files[FILE_H]) << 9;
            let b = (bb_square & !files[FILE_A]) >> 9 | (bb_square & !files[FILE_H]) >> 7;
            self._pawns[WHITE][sq as usize] = w;
            self._pawns[BLACK][sq as usize] = b;
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
    fn init_magics(&mut self, piece: Piece) {
        assert!(piece == ROOK || piece == BISHOP, "Illegal piece: {}", 0);
        let is_rook = piece == ROOK;
        let mut offset = 0;
        let mut total_permutations = 0;
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
            let mut mcurrent: Magics = Default::default();

            mcurrent.mask = mask;
            mcurrent.shift = (64 - bits) as u8;
            mcurrent.offset = offset;
            mcurrent.magic = if is_rook {
                ROOK_MAGICS[sq as usize]
            } else {
                BISHOP_MAGICS[sq as usize]
            };
            total_permutations += permutations;
            for i in 0..permutations {
                let next = i as usize;
                let index = mcurrent.get_index(blocker_boards[next]);
                let table: &mut [Bitboard] = if is_rook {
                    &mut self._rook[..]
                } else {
                    &mut self._bishop[..]
                };
                if table[index] == EMPTY {
                    let fail_low = index < offset as usize;
                    let fail_high = index > end as usize;
                    assert!(!fail_low && !fail_high, "Indexing error. Error in Magics.");
                    table[index] = attack_boards[next];
                } else {
                    panic!("Attack table index not empty. Error in Magics.");
                }
            }
            // No failures  during indexing.
            // Store this magic, then do the next one.
            if is_rook {
                self._rook_magics[sq as usize] = mcurrent;
            } else {
                self._bishop_magics[sq as usize] = mcurrent
            }
            offset += permutations;
        }
        // All permutations (blocker boards) should have been indexed.
        assert!(
            (total_permutations as usize)
                == if is_rook {
                    ROOK_TABLE_SIZE
                } else {
                    BISHOP_TABLE_SIZE
                },
            "Initializing magics failed. Check magic numbers."
        );
    }
}
