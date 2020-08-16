pub mod defs;
mod fen;
mod gamestate;
mod history;
mod playmove;
mod utils;
mod zobrist;

use self::{
    defs::{Pieces, BB_SQUARES},
    gamestate::GameState,
    history::History,
    zobrist::{ZobristKey, ZobristRandoms},
};
use crate::{
    defs::{Bitboard, NrOf, Piece, Side, Sides, Square, EMPTY},
    evaluation::{defs::PIECE_VALUES, material},
    misc::bits,
};
use std::sync::Arc;

// TODO: Update comments
#[derive(Clone)]
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub game_state: GameState,
    pub history: History,
    pub piece_list: [Piece; NrOf::SQUARES],
    pub material_count: [u16; Sides::BOTH],
    zobrist_randoms: Arc<ZobristRandoms>,
}

// Public functions for use by other modules.
impl Board {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            game_state: GameState::new(),
            history: History::new(),
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            material_count: [0; Sides::BOTH],
            zobrist_randoms: Arc::new(ZobristRandoms::new()),
        }
    }

    // Return a bitboard with locations of a certain piece type for one of the sides.
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    // Return a bitboard containing all the pieces on the board.
    pub fn occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    pub fn us(&self) -> usize {
        self.game_state.active_color as usize
    }

    pub fn opponent(&self) -> usize {
        (self.game_state.active_color ^ 1) as usize
    }

    pub fn king_square(&self, side: Side) -> Square {
        self.bb_pieces[side][Pieces::KING].trailing_zeros() as Square
    }

    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.piece_list[square] = Pieces::NONE;
        self.material_count[side] -= PIECE_VALUES[piece];
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        self.material_count[side] += PIECE_VALUES[piece];
        self.piece_list[square] = piece;
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

// Private board functions (for initializating on startup)
impl Board {
    fn reset(&mut self) {
        self.bb_pieces = [[0; NrOf::PIECE_TYPES]; Sides::BOTH];
        self.bb_side = [EMPTY; Sides::BOTH];
        self.game_state = GameState::new();
        self.history.clear();
        self.piece_list = [Pieces::NONE; NrOf::SQUARES];
        self.material_count = [0; Sides::BOTH];
    }

    fn init(&mut self) {
        let piece_bitboards = self.init_piece_bitboards();
        self.bb_side[Sides::WHITE] = piece_bitboards.0;
        self.bb_side[Sides::BLACK] = piece_bitboards.1;

        self.piece_list = self.init_piece_list();
        self.game_state.zobrist_key = self.init_zobrist_key();

        let material = material::count(&self);
        self.material_count[Sides::WHITE] = material.0;
        self.material_count[Sides::BLACK] = material.1;
    }

    fn init_piece_bitboards(&self) -> (Bitboard, Bitboard) {
        let mut bb_white: Bitboard = 0;
        let mut bb_black: Bitboard = 0;

        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        (bb_white, bb_black)
    }

    fn init_piece_list(&self) -> [Piece; NrOf::SQUARES] {
        let bb_w = self.bb_pieces[Sides::WHITE]; // White bitboards
        let bb_b = self.bb_pieces[Sides::BLACK]; // Black bitboards
        let mut piece_list: [Piece; NrOf::SQUARES] = [Pieces::NONE; NrOf::SQUARES];

        // piece_type is enumerated, from 0 to 6.
        // 0 = KING, 1 = QUEEN, and so on, as defined in board::defs.
        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                piece_list[square] = piece_type;
            }

            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                piece_list[square] = piece_type;
            }
        }

        piece_list
    }

    // TODO: Write comments.
    fn init_zobrist_key(&self) -> ZobristKey {
        let mut key: u64 = 0;
        let zr = &self.zobrist_randoms;
        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                key ^= zr.piece(Sides::WHITE, piece_type, square);
            }

            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                key ^= zr.piece(Sides::BLACK, piece_type, square);
            }
        }

        key ^= zr.castling(self.game_state.castling);
        key ^= zr.side(self.game_state.active_color as usize);
        key ^= zr.en_passant(self.game_state.en_passant);

        key
    }
}
