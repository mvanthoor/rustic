use crate::{
    board::{defs::Pieces, zobrist::ZobristKey, Board},
    defs::{Bitboard, NrOf, Piece, Sides},
    evaluation::defs::EvalParams,
    evaluation::Evaluation,
    misc::bits,
    movegen::defs::Move,
};

// Initialization functions
impl Board {
    // Main initialization function. This is used to initialize the "other"
    // bit-boards that are not set up by the FEN-reader function.
    pub fn init(&mut self) {
        // Gather all the pieces of a side into one bitboard; one bitboard
        // with all the white pieces, and one with all black pieces.
        let pieces_per_side_bitboards = self.init_pieces_per_side_bitboards();
        self.bb_side[Sides::WHITE] = pieces_per_side_bitboards.0;
        self.bb_side[Sides::BLACK] = pieces_per_side_bitboards.1;

        // Initialize incrementally updated values.
        self.piece_list = self.init_piece_list();
        self.game_state.zobrist_key = self.init_zobrist_key();
        self.game_state.phase_value = Evaluation::count_phase(self);
        self.game_state.next_move = Move::new(0);

        // Set initial PST values: also incrementally updated.
        let pst_values = Evaluation::psqt_apply(self, &EvalParams::PSQT_SET);
        self.game_state.psqt_value[Sides::WHITE] = pst_values.0;
        self.game_state.psqt_value[Sides::BLACK] = pst_values.1;
    }

    // Gather the pieces for each side into their own bitboard.
    fn init_pieces_per_side_bitboards(&self) -> (Bitboard, Bitboard) {
        let mut bb_white: Bitboard = 0;
        let mut bb_black: Bitboard = 0;

        // Iterate over the bitboards of every piece type.
        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        // Return a bitboard with all white pieces, and a bitboard with all
        // black pieces.
        (bb_white, bb_black)
    }

    // Initialize the piece list. This list is used to quickly determine
    // which piece type (rook, knight...) is on a square without having to
    // loop through the piece bitboards.
    fn init_piece_list(&self) -> [Piece; NrOf::SQUARES] {
        let bb_w = self.bb_pieces[Sides::WHITE]; // White piece bitboards
        let bb_b = self.bb_pieces[Sides::BLACK]; // Black piece bitboards
        let mut piece_list: [Piece; NrOf::SQUARES] = [Pieces::NONE; NrOf::SQUARES];

        // piece_type is enumerated, from 0 to 6.
        // 0 = KING, 1 = QUEEN, and so on, as defined in board::defs.
        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            // Put white pieces into the piece list.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                piece_list[square] = piece_type;
            }

            // Put black pieces into the piece list.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                piece_list[square] = piece_type;
            }
        }

        piece_list
    }

    // Initialize the zobrist hash. This hash will later be updated incrementally.
    pub fn init_zobrist_key(&self) -> ZobristKey {
        // Keep the key here.
        let mut key: u64 = 0;

        // Same here: "bb_w" is shorthand for
        // "self.bb_pieces[Sides::WHITE]".
        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        // Iterate through all piece types, for both white and black.
        // "piece_type" is enumerated, and it'll start at 0 (KING), then 1
        // (QUEEN), and so on.
        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            // Assume the first iteration; piece_type will be 0 (KING). The
            // following two statements will thus get all the pieces of
            // type "KING" for white and black. (This will obviously only
            // be one king, but with rooks, there will be two in the
            // starting position.)
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            // Iterate through all the piece locations of the current piece
            // type. Get the square the piece is on, and then hash that
            // square/piece combination into the zobrist key.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                key ^= self.zobrist_randoms.piece(Sides::WHITE, piece_type, square);
            }

            // Same for black.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                key ^= self.zobrist_randoms.piece(Sides::BLACK, piece_type, square);
            }
        }

        // Hash the castling, active color, and en-passant state.
        key ^= self.zobrist_randoms.castling(self.game_state.castling);
        key ^= self
            .zobrist_randoms
            .side(self.game_state.active_color as usize);
        key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);

        // Done; return the key.
        key
    }
}
