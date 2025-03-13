use crate::board::Board;
use crate::board::defs::{Pieces, BB_RANKS};
use crate::defs::{Bitboard, Sides};

/// A hash function that monotonically decreases whenever a pawn moves, a piece is captured, or a
/// player castles or gives up the right to castle. Used to quickly eliminate unreachable positions
/// from the transposition table.
impl Board {
    pub fn monotonic_hash(&self) -> u32 {
        let white_pawns = self.get_pieces(Pieces::PAWN, Sides::WHITE);
        let black_pawns = self.get_pieces(Pieces::PAWN, Sides::BLACK);
        let mut pawns_key = 0u32;
        for ranks_advanced in 0..=5 {
            let pawn_rank_key = (white_pawns & BB_RANKS[1 + ranks_advanced]) >> (8 * ranks_advanced + 8)
                | (black_pawns & BB_RANKS[6 - ranks_advanced]) >> (8 * (5 - ranks_advanced));
            pawns_key += (pawn_rank_key << (3 * (5 - ranks_advanced))) as u32;
        }
        const DARK_SQUARES: Bitboard = 0xAA55AA55AA55AA55;
        const LIGHT_SQUARES: Bitboard = !DARK_SQUARES;
        let mut pieces_keys = [0; 2];
        for side in [Sides::WHITE, Sides::BLACK] {
            let pawns = [white_pawns, black_pawns][side as usize].count_ones();
            let knights_and_pawns = pawns + self.get_pieces(Pieces::KNIGHT, side).count_ones();
            let bishops = self.get_pieces(Pieces::BISHOP, side);
            let light_bishops_and_pawns = pawns + (bishops & LIGHT_SQUARES).count_ones();
            let dark_bishops_and_pawns = pawns + (bishops & DARK_SQUARES).count_ones();
            let rooks_and_pawns = pawns + self.get_pieces(Pieces::ROOK, side).count_ones();
            let queens_and_pawns = pawns + self.get_pieces(Pieces::QUEEN, side).count_ones();
            pieces_keys[side as usize] = knights_and_pawns | light_bishops_and_pawns << 4 | dark_bishops_and_pawns << 8 | rooks_and_pawns << 12 | queens_and_pawns << 16;
        }
        pawns_key + pieces_keys[0] + (pieces_keys[1] << 7) + (1 << 19 - 1) * (self.game_state.castling as u32)
    }
}