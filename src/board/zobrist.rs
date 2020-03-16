use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::defs::NR_OF_SQUARES;

const NSQ: usize = NR_OF_SQUARES as usize;
const EACH_PIECE: usize = 6;
const EACH_SIDE: usize = 2;

/* Random number for each side's pieces on each square */
type PieceRandoms = [[[u64; NSQ]; EACH_PIECE]; EACH_SIDE];

pub struct ZobristRandoms {
    piece_randoms: PieceRandoms,
}

impl ZobristRandoms {
    pub fn new() -> ZobristRandoms {
        let mut random = SmallRng::from_entropy();
        let mut zobrist_randoms = ZobristRandoms {
            piece_randoms: [[[0; NSQ]; EACH_PIECE]; EACH_SIDE],
        };
        // Fill piece_randoms for each side, for each piece, on each square.
        for side in 0..EACH_SIDE {
            for piece in 0..EACH_PIECE {
                for square in 0..NSQ {
                    let key = random.gen::<u64>();
                    zobrist_randoms.piece_randoms[side][piece][square] = key;
                }
            }
        }
        zobrist_randoms
    }
}
