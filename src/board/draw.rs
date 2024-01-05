use crate::{
    board::{Board, Pieces},
    defs::{Sides, MAX_MOVE_RULE},
};

impl Board {
    // Returns true if the position should be evaluated as a draw.
    pub fn is_draw(&self) -> bool {
        (!self.can_force_checkmate())
            || self.draw_by_repetition_rule() > 0
            || self.draw_by_fifty_move_rule()
    }

    // Checks for the 50-move rule.
    pub fn draw_by_fifty_move_rule(&self) -> bool {
        self.game_state.halfmove_clock >= MAX_MOVE_RULE
    }

    // This function returns true if the amount of material on the board is
    // not sufficient to deliver checkmate, in ANY position, using ANY
    // sequence of legal moves, EVEN if the losing side is trying to ASSIST
    // in getting checkmated. In such a position a draw can officially be
    // claimed under FIDE rules.
    pub fn draw_by_insufficient_material_rule(&self) -> bool {
        // Get the piece bitboards for white and black.
        let w = self.get_bitboards(Sides::WHITE);
        let b = self.get_bitboards(Sides::BLACK);

        // Determine if at least one side has either a Queen, a Rook or a
        // pawn (qrp). If is the case, a draw by rule is not possible.
        let qrp = w[Pieces::QUEEN] != 0
            || w[Pieces::ROOK] != 0
            || w[Pieces::PAWN] != 0
            || b[Pieces::QUEEN] != 0
            || b[Pieces::ROOK] != 0
            || b[Pieces::PAWN] != 0;

        // No queens, rooks or pawns. We may have a draw.
        if !qrp {
            // King vs. King
            let kk = w[Pieces::BISHOP] == 0
                && w[Pieces::KNIGHT] == 0
                && b[Pieces::BISHOP] == 0
                && b[Pieces::KNIGHT] == 0;
            // King/Bishop vs. King
            let kbk = w[Pieces::BISHOP].count_ones() == 1
                && w[Pieces::KNIGHT] == 0
                && b[Pieces::BISHOP] == 0
                && b[Pieces::KNIGHT] == 0;
            // King/Knight vs. King
            let knk = w[Pieces::BISHOP] == 0
                && w[Pieces::KNIGHT].count_ones() == 1
                && b[Pieces::BISHOP] == 0
                && b[Pieces::KNIGHT] == 0;
            // King vs. King/Bishop
            let kkb = w[Pieces::BISHOP] == 0
                && w[Pieces::KNIGHT] == 0
                && b[Pieces::BISHOP].count_ones() == 1
                && b[Pieces::KNIGHT] == 0;
            // King vs. King/Knight
            let kkn = w[Pieces::BISHOP] == 0
                && w[Pieces::KNIGHT] == 0
                && b[Pieces::BISHOP] == 0
                && b[Pieces::KNIGHT].count_ones() == 1;
            // King/Bishop vs. King/Bishop
            let kbkb = w[Pieces::BISHOP].count_ones() == 1
                && w[Pieces::KNIGHT] == 0
                && b[Pieces::BISHOP].count_ones() == 1
                && b[Pieces::KNIGHT] == 0;
            // Both bishops have to be on the same colored square for a
            // draw to be claimable.
            let same_color_sq = if kbkb {
                let wb_sq = w[Pieces::BISHOP].trailing_zeros() as usize;
                let bb_sq = b[Pieces::BISHOP].trailing_zeros() as usize;

                Board::is_white_square(wb_sq) == Board::is_white_square(bb_sq)
            } else {
                false
            };

            if kk || kbk || knk || kkb || kkn || (kbkb && same_color_sq) {
                return true;
            }
        }

        // All other cases cannot be claimed as a draw.
        false
    }

    // Detects position repetitions in the game's history and returns the
    // of times a position was repeated.
    pub fn draw_by_repetition_rule(&self) -> u8 {
        let mut count = 0;
        let mut stop = false;
        let mut i = (self.history.len() - 1) as i16;

        // Search the history list.
        while i >= 0 && !stop {
            let historic = self.history.get_ref(i as usize);

            // If the historic zobrist key is equal to the one of the board
            // passed into the function, then we found a repetition.
            if historic.zobrist_key == self.game_state.zobrist_key {
                count += 1;
            }

            // If the historic HMC is 0, it indicates that this position
            // was created by a capture or pawn move. We don't have to
            // search further back, because before this, we can't ever
            // repeat. After all, the capture or pawn move can't be
            // reverted or repeated.
            stop = historic.halfmove_clock == 0;

            // Search backwards.
            i -= 1;
        }
        count
    }

    // This function returns true if there is enough material available for
    // at least one of the sides to force a checkmate, even with best play
    // from the losing side. If mate cannot be forced then this function
    // returns false. (In some situations it may be possible to achieve a
    // mate if the losing side plays very poorly or even assists in getting
    // mated, but this function will still return false if mate can not be
    // forced.)
    pub fn can_force_checkmate(&self) -> bool {
        let w = self.get_bitboards(Sides::WHITE);
        let b = self.get_bitboards(Sides::BLACK);

        w[Pieces::QUEEN] > 0
            || b[Pieces::QUEEN] > 0
            || w[Pieces::ROOK] > 0
            || b[Pieces::ROOK] > 0
            || w[Pieces::PAWN] > 0
            || b[Pieces::PAWN] > 0
            || self.has_bishop_pair(Sides::WHITE)
            || self.has_bishop_pair(Sides::BLACK)
            || (w[Pieces::BISHOP] > 0 && w[Pieces::KNIGHT] > 0)
            || (b[Pieces::BISHOP] > 0 && b[Pieces::KNIGHT] > 0)
            || w[Pieces::KNIGHT].count_ones() >= 3
            || b[Pieces::KNIGHT].count_ones() >= 3
    }
}