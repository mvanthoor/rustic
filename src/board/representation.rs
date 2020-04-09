/**
 * board.rs holds Rustic's board representation and functions associated with it.
 * Rustic uses bitboards. This means there will be at least 6 bitboards for each side;
 * one bitboard per piece type per side.
 * In addition, there are also bitboards containing all white pieces, all black pieces
 * (so it isn't necessary to loop through the bitboards all the time), and bitboards masking
 * files or ranks. Later, more bitboards (diagonals, for exmple) may be added.
 * All other things making up a chess position such as color, castling rights, e_passant
 * and others, will also be in this struct.
*/
use super::fen;
use super::zobrist::{ZobristKey, ZobristRandoms};
use crate::defs::{
    Bitboard, Piece, Side, BB_FOR_FILES, BB_FOR_RANKS, BITBOARDS_FOR_PIECES, BITBOARDS_PER_SIDE,
    BLACK, EMPTY, FEN_START_POSITION, NR_OF_PIECES, NR_OF_SQUARES, PNONE, WHITE,
};
use crate::movegen::{
    movedefs::{Move, MoveList},
    MoveGenerator,
};
use crate::utils::bits;

const MAX_GAME_MOVES: u16 = 2048;

#[derive(Clone, Copy)]
pub struct GameState {
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub this_move: Move,
}

impl GameState {
    pub fn new(ac: u8, c: u8, ep: Option<u8>, hmc: u8, fmn: u16, zk: u64, m: Move) -> GameState {
        GameState {
            active_color: ac,
            castling: c,
            en_passant: ep,
            halfmove_clock: hmc,
            fullmove_number: fmn,
            zobrist_key: zk,
            this_move: m,
        }
    }
}

pub type UnMakeList = Vec<GameState>;

#[derive(Clone)]
pub struct Board<'a> {
    pub bb_side: [[Bitboard; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize],
    pub bb_pieces: [Bitboard; BITBOARDS_FOR_PIECES as usize],
    pub bb_files: [Bitboard; BB_FOR_FILES as usize],
    pub bb_ranks: [Bitboard; BB_FOR_RANKS as usize],
    pub game_state: GameState,
    pub unmake_list: UnMakeList,
    pub zobrist_key: ZobristKey,
    pub zobrist_randoms: &'a ZobristRandoms,
    pub piece_list: [Piece; NR_OF_SQUARES as usize],
    move_generator: &'a MoveGenerator,
}

impl<'a> Board<'a> {
    /**
     * This function creates a new board. If an FEN-position is passed, then use that for
     * setting up the board. If None is passed, use the normal starting position.
     */
    pub fn new(zr: &'a ZobristRandoms, mg: &'a MoveGenerator, fen: Option<&str>) -> Board<'a> {
        let mut board = Board {
            bb_side: [[EMPTY; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize],
            bb_pieces: [EMPTY; BITBOARDS_FOR_PIECES as usize],
            bb_files: [EMPTY; BB_FOR_FILES as usize],
            bb_ranks: [EMPTY; BB_FOR_RANKS as usize],
            game_state: GameState::new(0, 0, None, 0, 0, 0, Move { data: 0 }),
            unmake_list: Vec::with_capacity(MAX_GAME_MOVES as usize),
            zobrist_key: EMPTY,
            zobrist_randoms: zr,
            piece_list: [PNONE; NR_OF_SQUARES as usize],
            move_generator: mg,
        };
        board.bb_files = super::create_bb_files();
        board.bb_ranks = super::create_bb_ranks();
        if let Some(f) = fen {
            board.setup_fen(f);
        } else {
            board.setup_fen(FEN_START_POSITION);
        }
        board.create_piece_list();
        board.zobrist_key = board.build_zobrist_key();

        board
    }

    /** Reset the board. */
    pub fn reset(&mut self) {
        self.bb_side = [[0; NR_OF_PIECES as usize]; BITBOARDS_PER_SIDE as usize];
        self.bb_pieces = [EMPTY; BITBOARDS_FOR_PIECES as usize];
        self.game_state = GameState::new(0, 0, None, 0, 0, 0, Move { data: 0 });
        self.unmake_list.clear();
        self.zobrist_key = EMPTY;
        self.piece_list = [PNONE; NR_OF_SQUARES as usize];
    }

    // Call the fen-reader function and create the piece bitboards to do the setup.
    pub fn setup_fen(&mut self, fen: &str) {
        fen::read(fen, self);
        self.create_piece_bitboards();
    }

    /** Get the pieces of a certain type, for one of the sides. */
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_side[side][piece]
    }

    /** Return a bitboard containing all the pieces on the board. */
    pub fn occupancy(&self) -> Bitboard {
        self.bb_pieces[WHITE] | self.bb_pieces[BLACK]
    }

    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: u8) {
        self.piece_list[square as usize] = PNONE;
        self.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        bits::clear_bit(&mut self.bb_side[side][piece], square);
        bits::clear_bit(&mut self.bb_pieces[side], square);
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: u8) {
        bits::set_bit(&mut self.bb_side[side][piece], square);
        bits::set_bit(&mut self.bb_pieces[side], square);
        self.zobrist_key ^= self.zobrist_randoms.piece(side, piece, square);
        self.piece_list[square as usize] = piece;
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: u8) {
        self.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = Some(square);
        self.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
    }

    // Clear the ep-square. (If the ep-square is None already, nothing changes.)
    pub fn clear_ep_square(&mut self) {
        self.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = None;
        self.zobrist_key ^= self.zobrist_randoms.en_passant(self.game_state.en_passant);
    }

    // Swap side from WHITE <==> BLACK
    pub fn swap_side(&mut self) {
        let us = self.game_state.active_color as usize;
        let opponent = us ^ 1;

        self.zobrist_key ^= self.zobrist_randoms.side(us);
        self.zobrist_key ^= self.zobrist_randoms.side(opponent);
        self.game_state.active_color = opponent as u8;
    }

    // This function creates bitboards per side, containing all the pieces of that side.
    fn create_piece_bitboards(&mut self) {
        for (bb_w, bb_b) in self.bb_side[WHITE].iter().zip(self.bb_side[BLACK].iter()) {
            self.bb_pieces[WHITE] |= *bb_w;
            self.bb_pieces[BLACK] |= *bb_b;
        }
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

    // This function builds the initial piece list (which piece type on which square).
    fn create_piece_list(&mut self) {
        let bb_w = self.bb_side[WHITE]; // White bitboards
        let bb_b = self.bb_side[BLACK]; // Black bitboards
        for (p, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white = *w; // White pieces of type "p"
            let mut black = *b; // Black pieces of type "p"

            while white > 0 {
                let square = bits::next(&mut white);
                self.piece_list[square as usize] = p;
            }

            while black > 0 {
                let square = bits::next(&mut black);
                self.piece_list[square as usize] = p;
            }
        }
    }

    /** This function builds the Zobrist key for the inital position. */
    pub fn build_zobrist_key(&mut self) -> ZobristKey {
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
