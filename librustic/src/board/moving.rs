use crate::{
    board::defs::{Pieces, BB_SQUARES},
    board::Board,
    defs::{Piece, Side, Square},
    evaluation::defs::EvalParams,
};

impl Board {
    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
        self.piece_list[square] = Pieces::NONE;
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        self.game_state.phase_value -= EvalParams::PHASE_VALUES[piece];
        let square = Board::flip(side, square);
        self.game_state.psqt_value[side].sub(EvalParams::PSQT_SET[piece][square]);
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.piece_list[square] = piece;
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        self.game_state.phase_value += EvalParams::PHASE_VALUES[piece];
        let square = Board::flip(side, square);
        self.game_state.psqt_value[side].add(EvalParams::PSQT_SET[piece][square]);
    }

    // Remove a piece from the from-square, and put it onto the to-square.
    pub fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: Square) {
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = Some(square as u8);
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
    }

    // Clear the ep-square. (If the ep-square is None already, nothing changes.)
    pub fn clear_ep_square(&mut self) {
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = None;
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
    }

    // Swap side from WHITE <==> BLACK
    pub fn swap_side(&mut self) {
        self.game_state.zobrist_key ^= self
            .zobrist_randoms
            .side(self.game_state.active_color as usize);
        self.game_state.active_color ^= 1;
        self.game_state.zobrist_key ^= self
            .zobrist_randoms
            .side(self.game_state.active_color as usize);
    }

    // Update castling permissions and take Zobrist-key into account.
    pub fn update_castling_permissions(&mut self, new_permissions: u8) {
        self.game_state.zobrist_key ^= self.zobrist_randoms.castling(self.game_state.castling);
        self.game_state.castling = new_permissions;
        self.game_state.zobrist_key ^= self.zobrist_randoms.castling(self.game_state.castling);
    }
}
