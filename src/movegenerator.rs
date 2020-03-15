pub mod blockatt;
mod gen;
mod information;
mod init;
pub mod magics;
pub mod masks;
pub mod movedefs;
mod rays;

// TODO: Rewrite comments for move generator
extern crate rand;
use crate::board::representation::Board;
use crate::board::{create_bb_files, create_bb_ranks};
use crate::defs::{Bitboard, Piece, Side, BISHOP, KING, KNIGHT, NR_OF_SQUARES, QUEEN, ROOK};
use init::{init_king, init_knight, init_magics, init_pawns};
use magics::Magics;
use movedefs::MoveList;

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

// impl Default for MoveGenerator {}

impl MoveGenerator {
    pub fn new() -> MoveGenerator {
        let magics: Magics = Default::default();
        let files = create_bb_files();
        let ranks = create_bb_ranks();
        let mut mg = MoveGenerator {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
            _rook: vec![EMPTY; ROOK_TABLE_SIZE],
            _bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            _rook_magics: [magics; NSQ],
            _bishop_magics: [magics; NSQ],
        };
        init_king(&mut mg, &files, &ranks);
        init_knight(&mut mg, &files, &ranks);
        init_pawns(&mut mg, &files);
        init_magics(&mut mg, ROOK);
        init_magics(&mut mg, BISHOP);
        mg
    }

    //** This function takes a board, and generates all moves for the side that is to move. */
    pub fn gen_all_moves(&self, board: &Board, ml: &mut MoveList) {
        gen::all_moves(board, self, ml);
    }

    // ===== Private functions for use by submodules ===== //

    /** Return non-slider (King, Knight) attacks for the given square. */
    fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        let sq = square as usize;

        match piece {
            KING => self._king[sq],
            KNIGHT => self._knight[sq],
            _ => 0,
        }
    }

    /** Return slider attacsk for Rook, Bishop and Queen using Magic. */
    fn get_slider_attacks(&self, piece: Piece, square: u8, occupancy: Bitboard) -> Bitboard {
        let sq = square as usize;

        match piece {
            ROOK => {
                let index = self._rook_magics[sq].get_index(occupancy);
                self._rook[index]
            }
            BISHOP => {
                let index = self._bishop_magics[sq].get_index(occupancy);
                self._bishop[index]
            }
            QUEEN => {
                let r_index = self._rook_magics[sq].get_index(occupancy);
                let b_index = self._bishop_magics[sq].get_index(occupancy);
                self._rook[r_index] ^ self._bishop[b_index]
            }
            _ => 0,
        }
    }

    /** Return pawn attacks for the given square. */
    fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        self._pawns[side][square as usize]
    }
}
