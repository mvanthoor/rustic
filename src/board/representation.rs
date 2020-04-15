// TODO: Update comments
mod gamestate;
mod history;

use super::fen;
use super::zobrist::{ZobristKey, ZobristRandoms};
use crate::defs::{
    Bitboard, Piece, Side, BITBOARDS_FOR_PIECES, BITBOARDS_PER_SIDE, BLACK, EMPTY,
    FEN_START_POSITION, NR_OF_PIECES, NR_OF_SQUARES, PNONE, WHITE,
};
use crate::evaluation::{evaldefs::PIECE_VALUES, material};
use crate::movegen::{movedefs::MoveList, MoveGenerator};
use crate::utils::bits;
use gamestate::GameState;
use history::History;

#[derive(Clone)]
pub struct Board<'a> {
    pub bb_side: [[Bitboard; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize],
    pub bb_pieces: [Bitboard; BITBOARDS_FOR_PIECES as usize],
    pub game_state: GameState,
    pub history: History,
    pub zobrist_randoms: &'a ZobristRandoms,
    pub piece_list: [Piece; NR_OF_SQUARES as usize],
    move_generator: &'a MoveGenerator,
}

impl<'a> Board<'a> {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new(zr: &'a ZobristRandoms, mg: &'a MoveGenerator, fen: Option<&str>) -> Board<'a> {
        let mut board = Board {
            bb_side: [[EMPTY; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize],
            bb_pieces: [EMPTY; BITBOARDS_FOR_PIECES as usize],
            game_state: GameState::new(),
            history: History::new(),
            zobrist_randoms: zr,
            piece_list: [PNONE; NR_OF_SQUARES as usize],
            move_generator: mg,
        };
        if let Some(f) = fen {
            board.setup_fen(f);
        } else {
            board.setup_fen(FEN_START_POSITION);
        }
        board.piece_list = board.create_piece_list();
        board.game_state.zobrist_key = board.create_zobrist_key();

        // Count material after board setup finished.
        let material = material::count(&board);
        board.game_state.material[WHITE] = material.0;
        board.game_state.material[BLACK] = material.1;

        board
    }

    // Reset the board.
    pub fn reset(&mut self) {
        self.bb_side = [[0; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize];
        self.bb_pieces = [EMPTY; BITBOARDS_FOR_PIECES as usize];
        self.game_state = GameState::new();
        self.history.clear();
        self.piece_list = [PNONE; NR_OF_SQUARES as usize];
    }

    // Call the fen-reader function and create the piece bitboards to do the setup.
    pub fn setup_fen(&mut self, fen: &str) {
        fen::read(fen, self);
        self.create_piece_bitboards();
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
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: u8) {
        self.piece_list[square as usize] = PNONE;
        self.game_state.material[side] -= PIECE_VALUES[piece];
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        bits::clear_bit(&mut self.bb_side[side][piece], square);
        bits::clear_bit(&mut self.bb_pieces[side], square);
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: u8) {
        bits::set_bit(&mut self.bb_side[side][piece], square);
        bits::set_bit(&mut self.bb_pieces[side], square);
        self.game_state.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        self.game_state.material[side] += PIECE_VALUES[piece];
        self.piece_list[square as usize] = piece;
    }

    pub fn move_piece(&mut self, side: Side, piece: Piece, from: u8, to: u8) {
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: u8) {
        self.game_state.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = Some(square);
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
        let us = self.game_state.active_color as usize;
        let opponent = us ^ 1;

        self.game_state.zobrist_key ^= self.zobrist_randoms.side(us);
        self.game_state.zobrist_key ^= self.zobrist_randoms.side(opponent);
        self.game_state.active_color = opponent as u8;
    }

    // Passthrough functions for move generator
    pub fn gen_all_moves(&self, ml: &mut MoveList) {
        self.move_generator.gen_all_moves(self, ml);
    }

    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        self.move_generator.get_non_slider_attacks(piece, square)
    }

    pub fn get_slider_attacks(&self, piece: Piece, square: u8, occ: Bitboard) -> Bitboard {
        self.move_generator.get_slider_attacks(piece, square, occ)
    }

    pub fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        self.move_generator.get_pawn_attacks(side, square)
    }

    // This function creates bitboards per side, containing all the pieces of that side.
    fn create_piece_bitboards(&mut self) {
        for (bb_w, bb_b) in self.bb_side[WHITE].iter().zip(self.bb_side[BLACK].iter()) {
            self.bb_pieces[WHITE] |= *bb_w;
            self.bb_pieces[BLACK] |= *bb_b;
        }
    }

    // Build initial piece list with piece locations.
    fn create_piece_list(&mut self) -> [Piece; NR_OF_SQUARES as usize] {
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
    fn create_zobrist_key(&mut self) -> ZobristKey {
        let mut key: u64 = 0;
        let zr = self.zobrist_randoms;
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
}
