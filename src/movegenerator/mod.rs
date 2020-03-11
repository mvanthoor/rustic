pub mod blockatt;
pub mod gen;
pub mod init;
pub mod magics;
pub mod masks;
mod rays;

// TODO: Rewrite comments for move generator
extern crate rand;
use crate::board::{create_bb_files, create_bb_ranks};
use crate::defs::{Bitboard, Piece, Side, BISHOP, KING, KNIGHT, NR_OF_SQUARES, QUEEN, ROOK};
use init::{init_king, init_knight, init_magics, init_pawns};
use magics::Magics;

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
pub struct MoveGenerator {
    _king: [Bitboard; NSQ],
    _knight: [Bitboard; NSQ],
    _pawns: [[Bitboard; NSQ]; WHITE_BLACK],
    _rook: Vec<Bitboard>,
    _bishop: Vec<Bitboard>,
    _rook_magics: [Magics; NSQ],
    _bishop_magics: [Magics; NSQ],
}

impl Default for MoveGenerator {
    fn default() -> MoveGenerator {
        let magics: Magics = Default::default();
        MoveGenerator {
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

impl MoveGenerator {
    pub fn initialize(&mut self) {
        let files = create_bb_files();
        let ranks = create_bb_ranks();

        init_king(self, &files, &ranks);
        init_knight(self, &files, &ranks);
        init_pawns(self, &files);
        init_magics(self, ROOK);
        init_magics(self, BISHOP);
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
}
