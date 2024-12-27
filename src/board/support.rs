use crate::board::{defs::Pieces, Board};
use crate::{
    defs::{Bitboard, NrOf, Piece, Side, Sides, Square, EMPTY},
    misc::bits,
};

impl Board {
    pub fn reset(&mut self) {
        self.bb_pieces = [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH];
        self.bb_side = [EMPTY; Sides::BOTH];
        self.game_state.clear();
        self.history.clear();
        self.piece_list = [Pieces::NONE; NrOf::SQUARES];
    }

    // Return a bitboard with locations of a certain piece type for one of the sides.
    pub fn get_pieces(&self, side: Side, piece: Piece) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    pub fn get_bitboards(&self, side: Side) -> &[u64; NrOf::PIECE_TYPES] {
        &self.bb_pieces[side]
    }

    // Return a bitboard containing all the pieces on the board.
    pub fn occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    // Returns the side to move.
    pub fn us(&self) -> usize {
        self.game_state.active_color as usize
    }

    // Returns the side that is NOT moving.
    pub fn opponent(&self) -> usize {
        (self.game_state.active_color ^ 1) as usize
    }

    // Returns the square the king is currently on.
    pub fn king_square(&self, side: Side) -> Square {
        self.bb_pieces[side][Pieces::KING].trailing_zeros() as Square
    }

    pub fn has_bishop_pair(&self, side: Side) -> bool {
        let mut bishops = self.get_pieces(side, Pieces::BISHOP);
        let mut white_square = 0;
        let mut black_square = 0;

        if bishops.count_ones() >= 2 {
            while bishops > 0 {
                let square = bits::next(&mut bishops);

                if Board::is_white_square(square) {
                    white_square += 1;
                } else {
                    black_square += 1;
                }
            }
        }

        white_square >= 1 && black_square >= 1
    }
}
