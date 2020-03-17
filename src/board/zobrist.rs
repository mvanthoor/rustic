use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::defs::{Piece, Side, EMPTY, NR_OF_SQUARES};

const ALL_SQUARES: usize = NR_OF_SQUARES as usize;
const ALL_PIECES: usize = 6;
const ALL_SIDES: usize = 2;
const ALL_CASTLING_PERMISSIONS: usize = 16;
const ALL_EN_PASSANT_SQUARES: usize = 16;

/* Random number for all sides for all pieces on all squares */
type PieceRandoms = [[[u64; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES];
type CastlingRandoms = [u64; ALL_CASTLING_PERMISSIONS];
type EnPassantRandoms = [u64; ALL_EN_PASSANT_SQUARES];
type SideRandoms = [u64; ALL_SIDES];

pub type ZobristKey = u64;

pub struct ZobristRandoms {
    rnd_pieces: PieceRandoms,
    rnd_castling: CastlingRandoms,
    rnd_en_passant: EnPassantRandoms,
    rnd_sides: SideRandoms,
}

impl ZobristRandoms {
    pub fn new() -> ZobristRandoms {
        let mut random = SmallRng::from_seed([128; 16]);
        let mut zobrist_randoms = ZobristRandoms {
            rnd_pieces: [[[EMPTY; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES],
            rnd_castling: [EMPTY; ALL_CASTLING_PERMISSIONS],
            rnd_en_passant: [EMPTY; ALL_EN_PASSANT_SQUARES],
            rnd_sides: [EMPTY; ALL_SIDES],
        };

        for side in 0..ALL_SIDES {
            for piece in 0..ALL_PIECES {
                for square in 0..ALL_SQUARES {
                    zobrist_randoms.rnd_pieces[side][piece][square] = random.gen::<u64>();
                }
            }
        }

        for permission in 0..ALL_CASTLING_PERMISSIONS {
            zobrist_randoms.rnd_castling[permission] = random.gen::<u64>();
        }

        for square in 0..ALL_EN_PASSANT_SQUARES {
            zobrist_randoms.rnd_en_passant[square] = random.gen::<u64>();
        }

        for side in 0..ALL_SIDES {
            zobrist_randoms.rnd_sides[side] = random.gen::<u64>();
        }

        zobrist_randoms
    }

    pub fn piece(&self, side: Side, piece: Piece, square: u8) -> u64 {
        self.rnd_pieces[side][piece][square as usize]
    }
}
