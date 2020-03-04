/** Magics.rs implements the generation of magic bitboards for sliders, and the attack tables
 * for the non-slider squares. After all the bitboards are generated, using the three functions
 * "get_non_slider_attacks()", "get_slider_attacks()" and "get_pawn_attacks()" will "magically"
 * return all the possible attacks from the given square on the current board.
 * All possible moves for all pieces on all board squares (taking into account all possible
 * combination of blockers for the sliders) are calculated at the start of the engine, which
 * saves tremendous amounts of time in the move generator.
 */
mod blockatt;
mod masks;
extern crate rand;

use crate::defines::{
    Bitboard, Piece, Side, ALL_SQUARES, BLACK, FILE_A, FILE_B, FILE_G, FILE_H, KING, KNIGHT,
    NR_OF_SQUARES, PAWN_SQUARES, RANK_1, RANK_2, RANK_7, RANK_8, WHITE,
};
use crate::utils::{create_bb_files, create_bb_ranks};
use blockatt::{create_bishop_attack_boards, create_blocker_boards, create_rook_attack_boards};
use masks::{create_bishop_mask, create_rook_mask};

const WHITE_BLACK: usize = 2;
const EMPTY: Bitboard = 0;
const NSQ: usize = NR_OF_SQUARES as usize;

const ROOK_TABLE_SIZE: usize = 102400; // Total permutations of all rook blocker boards.
const BISHOP_TABLE_SIZE: usize = 5248; // Total permutations of all bishop blocker boards.

type BlockerBoards = Vec<Bitboard>;
type AttackBoards = Vec<Bitboard>;

#[derive(Copy, Clone)]
pub struct Magics {
    mask: Bitboard,
    shift: u8,
    magic: u64,
    offset: u32,
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
    fn index(square: u8) {}
}

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
    _rook: [Bitboard; ROOK_TABLE_SIZE],
    _rook_magics: [Magics; NSQ],
    _bishop: [Bitboard; BISHOP_TABLE_SIZE],
    _bishop_magics: [Magics; NSQ],
}

impl Default for Movements {
    fn default() -> Movements {
        let magics: Magics = Default::default();
        Movements {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
            _rook: [EMPTY; ROOK_TABLE_SIZE],
            _rook_magics: [magics; NSQ],
            _bishop: [EMPTY; BISHOP_TABLE_SIZE],
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
        self.init_magics();
    }

    /** Return non-slider (King, Knight) attacks for the given square. */
    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        match piece {
            KING => self._king[square as usize],
            KNIGHT => self._knight[square as usize],
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

    /** This is the main part of the module: it generates all the "magic" numbers and
     * bitboards for every slider, on every square, for every blocker setup.
     * A blocker is a piece that is "in the way", causing the slider to not be able to
     * 'see' beyond that piece.
     * The first part initializes magics for the rook.
     * The second part initializes magics for the bishop.
     * This way of initializatoin is not the absolute fastest, as it could have been
     * done all in one loop, and without extra functions. This is not a problem.
     * Initialization is only done once, at program startup, and so it does not have any impact
     * on the performance of the engine. Because this is quite a difficult subject to wrap one's
     * head around, and speed is unimportant in this case, clarity is preferable over cleverness.
     */
    fn init_magics(&mut self) {
        for sq in ALL_SQUARES {
            let mask = create_rook_mask(sq);
            let bits = mask.count_ones();
            let permutations = 2u64.pow(bits);
            let blocker_boards = create_blocker_boards(mask);
            let attack_boards = create_rook_attack_boards(sq, blocker_boards);
            self._rook_magics[sq as usize].mask = mask;
            self._rook_magics[sq as usize].shift = (64 - bits) as u8;
        }

        for sq in ALL_SQUARES {
            let mask = create_bishop_mask(sq);
            let bits = mask.count_ones();
            let permutations = 2u64.pow(bits);
            let blocker_boards = create_blocker_boards(mask);
            let attack_boards = create_bishop_attack_boards(sq, blocker_boards);
            self._bishop_magics[sq as usize].mask = mask;
            self._bishop_magics[sq as usize].shift = (64 - bits) as u8;
        }
    }
}
