pub mod attackboards;
pub mod blockerboards;
pub mod defs;
mod gen;
pub mod info;
mod init;
pub mod magics;
pub mod masks;
pub mod movelist;
mod rays;

// TODO: Rewrite comments for move generator
use crate::board::{defs::Pieces, representation::Board};
use crate::defs::{Bitboard, Piece, Side, Square, EMPTY, NR_OF_SQUARES};
use init::{init_king, init_knight, init_magics, init_pawns};
use magics::Magics;
use movelist::MoveList;

const WHITE_BLACK: usize = 2;
pub const ROOK_TABLE_SIZE: usize = 102_400; // Total permutations of all rook blocker boards.
pub const BISHOP_TABLE_SIZE: usize = 5_248; // Total permutations of all bishop blocker boards.

/**
 * The struct "Magics" will hold all of the attack tables for each piece on each square.
 * The _rook and _bishop arrays hold the attack tables for the sliders. _rook_info and
 * _bishop_info hold the magic information, to get the correct attack board from the
 * respective attack table and return it. These tables and info are initialized in the
 * init_magics() function.
*/
pub struct MoveGenerator {
    king: [Bitboard; NR_OF_SQUARES],
    knight: [Bitboard; NR_OF_SQUARES],
    pawns: [[Bitboard; NR_OF_SQUARES]; WHITE_BLACK],
    rook: Vec<Bitboard>,
    bishop: Vec<Bitboard>,
    rook_magics: [Magics; NR_OF_SQUARES],
    bishop_magics: [Magics; NR_OF_SQUARES],
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magics = Default::default();
        let mut mg = Self {
            king: [EMPTY; NR_OF_SQUARES],
            knight: [EMPTY; NR_OF_SQUARES],
            pawns: [[EMPTY; NR_OF_SQUARES]; WHITE_BLACK],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NR_OF_SQUARES],
            bishop_magics: [magics; NR_OF_SQUARES],
        };
        init_king(&mut mg);
        init_knight(&mut mg);
        init_pawns(&mut mg);
        init_magics(&mut mg, Pieces::ROOK);
        init_magics(&mut mg, Pieces::BISHOP);
        mg
    }

    //** This function takes a board, and generates all moves for the side that is to move. */
    pub fn gen_all_moves(&self, board: &Board, ml: &mut MoveList) {
        gen::all_moves(board, ml);
    }

    // ===== Private functions for use by submodules ===== //

    /** Return non-slider (King, Knight) attacks for the given square. */
    pub fn get_non_slider_attacks(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => self.king[square],
            Pieces::KNIGHT => self.knight[square],
            _ => panic!("Not a king or a knight: {}", piece),
        }
    }

    /** Return slider attacsk for Rook, Bishop and Queen using Magic. */
    pub fn get_slider_attacks(
        &self,
        piece: Piece,
        square: Square,
        occupancy: Bitboard,
    ) -> Bitboard {
        match piece {
            Pieces::ROOK => {
                let index = self.rook_magics[square].get_index(occupancy);
                self.rook[index]
            }
            Pieces::BISHOP => {
                let index = self.bishop_magics[square].get_index(occupancy);
                self.bishop[index]
            }
            Pieces::QUEEN => {
                let r_index = self.rook_magics[square].get_index(occupancy);
                let b_index = self.bishop_magics[square].get_index(occupancy);
                self.rook[r_index] ^ self.bishop[b_index]
            }
            _ => panic!("Not a sliding piece: {}", piece),
        }
    }

    /** Return pawn attacks for the given square. */
    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.pawns[side][square]
    }
}
