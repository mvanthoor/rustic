use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::defs::{
    Piece, Side, Square, EACH_SIDE, EMPTY, NR_OF_CASTLING_PERMISSIONS, NR_OF_PIECES, NR_OF_SQUARES,
};

const ALL_SQUARES: usize = NR_OF_SQUARES;
const ALL_PIECES: usize = NR_OF_PIECES as usize;
const ALL_SIDES: usize = EACH_SIDE as usize;
const ALL_CASTLING_PERMISSIONS: usize = NR_OF_CASTLING_PERMISSIONS as usize;

/* Random number for all sides for all pieces on all squares */
type PieceRandoms = [[[u64; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES];
type CastlingRandoms = [u64; ALL_CASTLING_PERMISSIONS];
type SideRandoms = [u64; ALL_SIDES];
type EpRandoms = [u64; ALL_SQUARES + 1];

pub type ZobristKey = u64;

pub struct ZobristRandoms {
    rnd_pieces: PieceRandoms,
    rnd_castling: CastlingRandoms,
    rnd_sides: SideRandoms,
    rnd_en_passant: EpRandoms,
}

impl ZobristRandoms {
    pub fn new() -> Self {
        let mut random = SmallRng::from_seed([125; 16]);
        let mut zobrist_randoms = Self {
            rnd_pieces: [[[EMPTY; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES],
            rnd_castling: [EMPTY; ALL_CASTLING_PERMISSIONS],
            rnd_sides: [EMPTY; ALL_SIDES],
            rnd_en_passant: [EMPTY; ALL_SQUARES + 1],
        };

        zobrist_randoms.rnd_pieces.iter_mut().for_each(|side| {
            side.iter_mut().for_each(|piece| {
                piece
                    .iter_mut()
                    .for_each(|square| *square = random.gen::<u64>())
            })
        });

        zobrist_randoms
            .rnd_castling
            .iter_mut()
            .for_each(|permission| *permission = random.gen::<u64>());

        zobrist_randoms
            .rnd_sides
            .iter_mut()
            .for_each(|side| *side = random.gen::<u64>());

        zobrist_randoms
            .rnd_en_passant
            .iter_mut()
            .for_each(|ep| *ep = random.gen::<u64>());

        zobrist_randoms
    }

    pub fn piece(&self, side: Side, piece: Piece, square: Square) -> ZobristKey {
        self.rnd_pieces[side][piece][square]
    }

    pub fn castling(&self, castling_permissions: u8) -> ZobristKey {
        self.rnd_castling[castling_permissions as usize]
    }

    pub fn side(&self, side: Side) -> u64 {
        self.rnd_sides[side]
    }

    pub fn en_passant(&self, en_passant: Option<u8>) -> ZobristKey {
        match en_passant {
            Some(ep) => self.rnd_en_passant[ep as usize],
            None => self.rnd_en_passant[ALL_SQUARES],
        }
    }
}
