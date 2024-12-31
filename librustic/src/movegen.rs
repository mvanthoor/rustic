mod create;
pub mod defs;
mod generate;
mod init;
mod magics;
mod movelist;

use crate::board::defs::Pieces;
use crate::defs::{Bitboard, NrOf, Sides, EMPTY};
use crate::movegen::magics::{Magic, BISHOP_TABLE_SIZE, ROOK_TABLE_SIZE};

// The move generator struct holds the attack table for each piece, and the
// tables with magic numbers for the rook and bishop.
pub struct MoveGenerator {
    king: [Bitboard; NrOf::SQUARES],
    knight: [Bitboard; NrOf::SQUARES],
    pawns: [[Bitboard; NrOf::SQUARES]; Sides::BOTH],
    rook: Vec<Bitboard>,
    bishop: Vec<Bitboard>,
    rook_magics: [Magic; NrOf::SQUARES],
    bishop_magics: [Magic; NrOf::SQUARES],
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveGenerator {
    pub fn new() -> Self {
        let magics: Magic = Default::default();
        let mut mg = Self {
            king: [EMPTY; NrOf::SQUARES],
            knight: [EMPTY; NrOf::SQUARES],
            pawns: [[EMPTY; NrOf::SQUARES]; Sides::BOTH],
            rook: vec![EMPTY; ROOK_TABLE_SIZE],
            bishop: vec![EMPTY; BISHOP_TABLE_SIZE],
            rook_magics: [magics; NrOf::SQUARES],
            bishop_magics: [magics; NrOf::SQUARES],
        };
        mg.init_king();
        mg.init_knight();
        mg.init_pawns();
        mg.init_magics(Pieces::ROOK);
        mg.init_magics(Pieces::BISHOP);
        mg
    }
}
