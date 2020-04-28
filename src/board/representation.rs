// TODO: Update comments
mod gamestate;
mod history;

use super::{
    fen,
    zobrist::{ZobristKey, ZobristRandoms},
    BB_SQUARES,
};
use crate::defs::{
    Bitboard, Piece, Side, Square, BLACK, EACH_SIDE, EMPTY, FEN_START_POSITION, NR_OF_PIECES,
    NR_OF_SQUARES, PNONE, WHITE,
};
use crate::evaluation::{evaldefs::PIECE_VALUES, material};
use crate::movegen::{movelist::MoveList, MoveGenerator};
use crate::utils::bits;
use gamestate::GameState;
use history::History;
use std::sync::Arc;

#[derive(Clone)]
pub struct Board {
    pub bb_side: [[Bitboard; NR_OF_PIECES as usize]; EACH_SIDE as usize],
    pub bb_pieces: [Bitboard; EACH_SIDE as usize],
    pub game_state: GameState,
    pub history: History,
    pub piece_list: [Piece; NR_OF_SQUARES as usize],
    pub material_count: [u16; EACH_SIDE as usize],
    zobrist_randoms: Arc<ZobristRandoms>,
    move_generator: Arc<MoveGenerator>,
}

impl Board {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new(zr: Arc<ZobristRandoms>, mg: Arc<MoveGenerator>) -> Self {
        Self {
            bb_side: [[EMPTY; NR_OF_PIECES as usize]; EACH_SIDE as usize],
            bb_pieces: [EMPTY; EACH_SIDE as usize],
            game_state: GameState::new(),
            history: History::new(),
            piece_list: [PNONE; NR_OF_SQUARES as usize],
            material_count: [0; EACH_SIDE as usize],
            zobrist_randoms: zr,
            move_generator: mg,
        }
    }

    // Reads either the passedn FEN-string or the starting position into the board.
    pub fn fen_read(&mut self, fen: Option<&str>) -> Result<(), u8> {
        if let Some(f) = fen {
            fen::read(self, f)
        } else {
            fen::read(self, FEN_START_POSITION)
        }
    }

    // After reading the FEN-string, piece bitboards and lists must be initialized.
    pub fn init(&mut self) {
        let piece_bitboards = self.init_piece_bitboards();
        self.bb_pieces[WHITE] = piece_bitboards.0;
        self.bb_pieces[BLACK] = piece_bitboards.1;

        self.piece_list = self.init_piece_list();
        self.game_state.zobrist_key = self.init_zobrist_key();

        let material = material::count(&self);
        self.material_count[WHITE] = material.0;
        self.material_count[BLACK] = material.1;
    }

    // Reset the board.
    pub fn reset(&mut self) {
        self.bb_side = [[0; NR_OF_PIECES as usize]; EACH_SIDE as usize];
        self.bb_pieces = [EMPTY; EACH_SIDE as usize];
        self.piece_list = [PNONE; NR_OF_SQUARES as usize];
        self.game_state = GameState::new();
        self.history.clear();
    }

    // Return a bitboard with locations of a certain piece type for one of the sides.
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_side[side][piece]
    }

    // Return a bitboard containing all the pieces on the board.
    pub fn occupancy(&self) -> Bitboard {
        self.bb_pieces[WHITE] | self.bb_pieces[BLACK]
    }

    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.piece_list[square as usize] = PNONE;
        self.material_count[side] -= PIECE_VALUES[piece];
        self.zobrist_piece(side, piece, square);
        self.bb_side[side][piece] ^= BB_SQUARES[square as usize];
        self.bb_pieces[side] ^= BB_SQUARES[square as usize];
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_side[side][piece] |= BB_SQUARES[square as usize];
        self.bb_pieces[side] |= BB_SQUARES[square as usize];
        self.zobrist_piece(side, piece, square);
        self.material_count[side] += PIECE_VALUES[piece];
        self.piece_list[square as usize] = piece;
    }

    // Remove a piece from the from-square, and put it onto the to-square.
    pub fn move_piece(&mut self, side: Side, piece: Piece, from: u8, to: u8) {
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: Square) {
        self.zobrist_en_passant();
        self.game_state.en_passant = Some(square);
        self.zobrist_en_passant();
    }

    // Clear the ep-square. (If the ep-square is None already, nothing changes.)
    pub fn clear_ep_square(&mut self) {
        self.zobrist_en_passant();
        self.game_state.en_passant = None;
        self.zobrist_en_passant();
    }

    // Swap side from WHITE <==> BLACK
    pub fn swap_side(&mut self) {
        self.zobrist_side();
        self.game_state.active_color ^= 1;
        self.zobrist_side();
    }

    // This function creates bitboards per side, containing all the pieces of that side.
    fn init_piece_bitboards(&self) -> (Bitboard, Bitboard) {
        let mut white: Bitboard = 0;
        let mut black: Bitboard = 0;

        for (bb_w, bb_b) in self.bb_side[WHITE].iter().zip(self.bb_side[BLACK].iter()) {
            white |= *bb_w;
            black |= *bb_b;
        }

        (white, black)
    }

    // Build initial piece list with piece locations.
    fn init_piece_list(&self) -> [Piece; NR_OF_SQUARES as usize] {
        let bb_w = self.bb_side[WHITE]; // White bitboards
        let bb_b = self.bb_side[BLACK]; // Black bitboards
        let mut piece_list: [Piece; NR_OF_SQUARES as usize] = [PNONE; NR_OF_SQUARES as usize];

        for (p, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white = *w; // White pieces of type "p"
            let mut black = *b; // Black pieces of type "p"

            while white > 0 {
                let square = bits::next(&mut white);
                piece_list[square as usize] = p;
            }

            while black > 0 {
                let square = bits::next(&mut black);
                piece_list[square as usize] = p;
            }
        }

        piece_list
    }

    // Create the initial Zobirst Key.
    fn init_zobrist_key(&self) -> ZobristKey {
        let mut key: u64 = 0;
        let zr = &self.zobrist_randoms;
        let bb_w = self.bb_side[WHITE];
        let bb_b = self.bb_side[BLACK];

        for (piece, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white = *w;
            let mut black = *b;

            while white > 0 {
                let square = bits::next(&mut white);
                key ^= zr.piece(WHITE, piece, square);
            }

            while black > 0 {
                let square = bits::next(&mut black);
                key ^= zr.piece(BLACK, piece, square);
            }
        }

        key ^= zr.castling(self.game_state.castling);
        key ^= zr.side(self.game_state.active_color as usize);
        key ^= zr.en_passant(self.game_state.en_passant);

        key
    }

    // ========== Move Generator passthrough ==========

    pub fn gen_all_moves(&self, ml: &mut MoveList) {
        self.move_generator.gen_all_moves(self, ml);
    }

    pub fn get_non_slider_attacks(&self, piece: Piece, square: Square) -> Bitboard {
        self.move_generator.get_non_slider_attacks(piece, square)
    }

    pub fn get_slider_attacks(&self, piece: Piece, square: Square, occ: Bitboard) -> Bitboard {
        self.move_generator.get_slider_attacks(piece, square, occ)
    }

    pub fn get_pawn_attacks(&self, side: Side, square: Square) -> Bitboard {
        self.move_generator.get_pawn_attacks(side, square)
    }

    // ========== Zobrist Randoms passthrough ==========

    pub fn zobrist_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
    }

    pub fn zobrist_castling(&mut self) {
        let gs_c = self.game_state.castling;
        self.game_state.zobrist_key ^= self.zobrist_randoms.castling(gs_c);
    }

    pub fn zobrist_en_passant(&mut self) {
        let gs_ep = self.game_state.en_passant;
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(gs_ep);
    }

    pub fn zobrist_side(&mut self) {
        let gs_side = self.game_state.active_color as usize;
        self.game_state.zobrist_key ^= self.zobrist_randoms.side(gs_side);
    }
}
