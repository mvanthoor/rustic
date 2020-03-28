use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::defs::{Piece, Side, EMPTY, NR_OF_SQUARES};

const ALL_SQUARES: usize = NR_OF_SQUARES as usize;
const ALL_PIECES: usize = 6;
const ALL_SIDES: usize = 2;
const EP_NO_YES: usize = 2;
const EP_NO: usize = 0;
const EP_YES: usize = 1;
const ALL_CASTLING_PERMISSIONS: usize = 16;

/* Random number for all sides for all pieces on all squares */
type PieceRandoms = [[[u64; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES];
type CastlingRandoms = [u64; ALL_CASTLING_PERMISSIONS];
type SideRandoms = [u64; ALL_SIDES];
type EpRandoms = [u64; EP_NO_YES];

pub type ZobristKey = u64;

pub struct ZobristRandoms {
    rnd_pieces: PieceRandoms,
    rnd_castling: CastlingRandoms,
    rnd_sides: SideRandoms,
    rnd_en_passant: EpRandoms,
}

impl ZobristRandoms {
    pub fn new() -> ZobristRandoms {
        // let mut random = SmallRng::from_entropy();
        let mut random = rand::thread_rng();
        let mut zobrist_randoms = ZobristRandoms {
            rnd_pieces: [[[EMPTY; ALL_SQUARES]; ALL_PIECES]; ALL_SIDES],
            rnd_castling: [EMPTY; ALL_CASTLING_PERMISSIONS],
            rnd_sides: [EMPTY; ALL_SIDES],
            rnd_en_passant: [EMPTY; EP_NO_YES],
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

        for side in 0..ALL_SIDES {
            zobrist_randoms.rnd_sides[side] = random.gen::<u64>();
        }

        for has_ep in 0..EP_NO_YES {
            zobrist_randoms.rnd_en_passant[has_ep] = random.gen::<u64>();
        }

        zobrist_randoms
    }

    pub fn piece(&self, side: Side, piece: Piece, square: u8) -> u64 {
        self.rnd_pieces[side][piece][square as usize]
    }

    pub fn castling(&self, castling_permissions: u8) -> u64 {
        self.rnd_castling[castling_permissions as usize]
    }

    pub fn side(&self, side: Side) -> u64 {
        self.rnd_sides[side]
    }

    pub fn en_passant(&self, en_passant: Option<u8>) -> u64 {
        let x = if en_passant.is_some() { EP_YES } else { EP_NO };
        self.rnd_en_passant[x]
    }
}
