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
use super::{create_bb_files, create_bb_ranks};
use crate::defs::{
    Bitboard, Piece, Side, BB_FOR_FILES, BB_FOR_RANKS, BITBOARDS_FOR_PIECES, BITBOARDS_PER_SIDE,
    BLACK, EMPTY, FEN_START_POSITION, PNONE, WHITE,
};
use crate::movegen::movedefs::Move;
use crate::utils::next;

const MAX_GAME_MOVES: u16 = 2048;

#[derive(Clone)]
pub struct UnMakeInfo {
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_key: u64,
    pub this_move: Move,
}

impl UnMakeInfo {
    pub fn new(ac: u8, c: u8, ep: Option<u8>, hmc: u8, fmn: u16, zk: u64, m: Move) -> UnMakeInfo {
        UnMakeInfo {
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

pub type UnMakeList = Vec<UnMakeInfo>;

#[derive(Clone)]
pub struct Board<'a> {
    pub bb_w: [Bitboard; BITBOARDS_PER_SIDE as usize],
    pub bb_b: [Bitboard; BITBOARDS_PER_SIDE as usize],
    pub bb_pieces: [Bitboard; BITBOARDS_FOR_PIECES as usize],
    pub bb_files: [Bitboard; BB_FOR_FILES as usize],
    pub bb_ranks: [Bitboard; BB_FOR_RANKS as usize],
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: Option<u8>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub unmake_list: UnMakeList,
    pub zobrist_key: ZobristKey,
    pub zobrist_randoms: &'a ZobristRandoms,
}

impl<'a> Board<'a> {
    /**
     * This function creates a new board. If an FEN-position is passed, then use that for
     * setting up the board. If None is passed, use the normal starting position.
     */
    pub fn new(zr: &'a ZobristRandoms, fen: Option<&str>) -> Board<'a> {
        let mut board = Board {
            bb_w: [EMPTY; BITBOARDS_PER_SIDE as usize],
            bb_b: [EMPTY; BITBOARDS_PER_SIDE as usize],
            bb_pieces: [EMPTY; BITBOARDS_FOR_PIECES as usize],
            bb_files: [EMPTY; BB_FOR_FILES as usize],
            bb_ranks: [EMPTY; BB_FOR_RANKS as usize],
            active_color: WHITE as u8,
            castling: 0,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 0,
            unmake_list: Vec::with_capacity(MAX_GAME_MOVES as usize),
            zobrist_key: EMPTY,
            zobrist_randoms: zr,
        };
        board.bb_files = create_bb_files();
        board.bb_ranks = create_bb_ranks();
        if let Some(f) = fen {
            board.setup_fen(f);
        } else {
            board.setup_fen(FEN_START_POSITION);
        }
        board.build_zobrist_key();

        board
    }

    /** Reset the board. */
    pub fn reset(&mut self) {
        self.bb_w = [0; BITBOARDS_PER_SIDE as usize];
        self.bb_b = [0; BITBOARDS_PER_SIDE as usize];
        self.bb_pieces = [EMPTY; BITBOARDS_FOR_PIECES as usize];
        self.active_color = WHITE as u8;
        self.castling = 0;
        self.en_passant = None;
        self.halfmove_clock = 0;
        self.fullmove_number = 0;
        self.unmake_list.clear();
        self.zobrist_key = EMPTY;
    }

    pub fn setup_fen(&mut self, fen: &str) {
        fen::read(fen, self);
        self.create_piece_bitboards();
    }

    /** Get the pieces of a certain type, for one of the sides. */
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        match side {
            WHITE => self.bb_w[piece],
            BLACK => self.bb_b[piece],
            _ => 0,
        }
    }

    /** Return which piece is on a given square, or return PNONE (no piece) */
    pub fn which_piece(&self, square: u8) -> Piece {
        let inspect = 1u64 << square as u64;
        for (piece, (white, black)) in self.bb_w.iter().zip(self.bb_b.iter()).enumerate() {
            if (*white & inspect > 0) || (*black & inspect > 0) {
                return piece;
            }
        }
        PNONE
    }

    /** Return a bitboard containing all the pieces on the board. */
    pub fn occupancy(&self) -> Bitboard {
        self.bb_pieces[WHITE] | self.bb_pieces[BLACK]
    }

    /**
     * This function iterates through all the white and black bitboards
     * to create the bitboard holding all of the pieces of that color.
     */
    fn create_piece_bitboards(&mut self) {
        for (bb_w, bb_b) in self.bb_w.iter().zip(self.bb_b.iter()) {
            self.bb_pieces[WHITE] |= *bb_w;
            self.bb_pieces[BLACK] |= *bb_b;
        }
    }

    /** This function builds the Zobrist key for the inital position. */
    fn build_zobrist_key(&mut self) {
        for (piece, (bb_w, bb_b)) in self.bb_w.iter().zip(self.bb_b.iter()).enumerate() {
            let mut white = *bb_w;
            let mut black = *bb_b;

            while white > 0 {
                let square = next(&mut white);
                self.zobrist_key ^= self.zobrist_randoms.piece(WHITE, piece, square);
            }

            while black > 0 {
                let square = next(&mut black);
                self.zobrist_key ^= self.zobrist_randoms.piece(BLACK, piece, square);
            }
        }

        self.zobrist_key ^= self.zobrist_randoms.castling(self.castling);
        self.zobrist_key ^= self.zobrist_randoms.side(self.active_color as usize);
        self.zobrist_key ^= self.zobrist_randoms.en_passant(self.en_passant);
    }
}
