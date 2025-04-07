use crate::board::Board;
use crate::board::defs::{Pieces, BB_RANKS};
use crate::defs::{Bitboard, Sides};

impl Board {
    /// A hash function that monotonically decreases whenever a pawn moves, a piece is captured, or a
    /// player castles or gives up the right to castle. Used to quickly eliminate unreachable positions
    /// from the transposition table.
    pub fn monotonic_hash(&self) -> u32 {
        // Decreasing multipliers for stronger pieces
        // Each side can have 0..=8 pawns and usually has 0..=1 or 0..=2 of each other piece,
        // which works out to only 419_903 reasonably likely values for each side. But "unreasonable"
        // numbers of all pieces except pawns can happen because of pawn promotions in excess of
        // those needed to replace captured pieces. To give these unreasonable numbers a chance of
        // not colliding with reasonable ones, we round up each multiplier generated on this basis
        // to the next prime. (The "raw" values in the comments below are the values we'd be using
        // with no such rounding.)
        //
        // The maximum sum of both pieces keys is
        // 46_703*8+5197*8+1733*2+577*2+293+149+73+37+13*2+5*2+3+1 == 420_412.
        const PAWN_MULT: [u32; 2] = [5197, 46703]; // usually 0..=8; raw: [5184, 46656]
        const KNIGHT_MULT: [u32; 2] = [1733, 577]; // usually 0..=2; raw: [1728, 576]
        const LIGHT_BISHOP_MULT: [u32; 2] = [149, 293]; // usually 0..=1; raw: [144, 288]
        const DARK_BISHOP_MULT: [u32; 2] = [73, 37]; // usually 0..=1; raw: [72, 36]
        const ROOK_MULT: [u32; 2] = [5, 13]; // usually 0..=2; raw: [4,12]
        const QUEEN_MULT: [u32; 2] = [3, 1]; // usually 0..=1; raw: [2,1]

        const DARK_SQUARES: Bitboard = 0xAA55AA55AA55AA55;
        const LIGHT_SQUARES: Bitboard = !DARK_SQUARES;
        let mut pieces_keys = [0u32; 2];

        let white_pawns = self.get_pieces(Pieces::PAWN, Sides::WHITE);
        let black_pawns = self.get_pieces(Pieces::PAWN, Sides::BLACK);
        for side in [Sides::WHITE, Sides::BLACK] {
            let pawns = [white_pawns, black_pawns][side as usize].count_ones();
            let knights = self.get_pieces(Pieces::KNIGHT, side).count_ones();
            let bishops = self.get_pieces(Pieces::BISHOP, side);
            let light_bishops = (bishops & LIGHT_SQUARES).count_ones();
            let dark_bishops = (bishops & DARK_SQUARES).count_ones();
            let rooks = self.get_pieces(Pieces::ROOK, side).count_ones();
            let queens = self.get_pieces(Pieces::QUEEN, side).count_ones();

            // Simply add weighted piece counts
            pieces_keys[side as usize] =
                pawns * PAWN_MULT[side as usize] +
                    knights * KNIGHT_MULT[side as usize] +
                    light_bishops * LIGHT_BISHOP_MULT[side as usize] +
                    dark_bishops * DARK_BISHOP_MULT[side as usize] +
                    rooks * ROOK_MULT[side as usize] +
                    queens * QUEEN_MULT[side as usize];
        }

        // Maximum castling key is 420_271 * 15 because only the lower 4 bits are used.
        let castling_key = self.game_state.castling as u32 * 420271;

        // Maximum en passant key is 97, since en passant captures only happen on ranks 3 and 6.
        // It increases to 127 if we've read an invalid target from FEN input.
        // Must be smaller than the difference between either side's pawn_key multipliers for their
        // second and fourth ranks.
        let en_passant_key = self.game_state.en_passant.map_or(0, |ep| ((ep as u32) << 1) + 1);

        // Use multipliers for rank weights
        // Largest possible rank value is 255 for each side
        // and the piece and castling keys leave a maximum value of 4288243248 for the pawn key,
        // so the sum of the 2 largest multipliers should be less than 4288243248/255.
        // The ones used here are the nearest primes to powers of the positive root of
        // x.pow(10) + x.pow(11) == 4288243248.0/255.0, which is
        // 4.455443274968434891800999910616226042276, excluding primes already used as multipliers
        // above and adjusting the largest few to minimize the residual.
        // Maximum pawn key is (13_734_121 + 3_082_517)*255 == 4_288_242_690
        let mut pawns_key = 0u32;

        // Both multipliers decrease as the pawns advance.
        const WHITE_RANK_MULTIPLIERS: [u32; 7] = [0, 13734121, 155291, 34849, 397, 89, 2];
        const BLACK_RANK_MULTIPLIERS: [u32; 7] = [0, 7, 19, 1753, 7823, 691878, 3082517];
        for rank in 1..=6 {
            let white_pawn_rank_key = (white_pawns & BB_RANKS[rank]) as u32
                * WHITE_RANK_MULTIPLIERS[rank];
            let black_pawn_rank_key = (black_pawns & BB_RANKS[rank]) as u32
                * BLACK_RANK_MULTIPLIERS[rank];
            pawns_key += white_pawn_rank_key + black_pawn_rank_key;
        }

        // Maximum total key is 4_288_242_690 + 420_412 + 420_271 * 15 + 97 == 4_294_967_264
        // == u32::MAX - 31
        pawns_key + pieces_keys[0] + pieces_keys[1] + castling_key + en_passant_key
    }
}


