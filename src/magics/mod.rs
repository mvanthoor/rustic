mod masks;

use crate::defines::{
    Bitboard, Piece, Side, ALL_SQUARES, BLACK, FILE_A, FILE_B, FILE_G, FILE_H, KING, KNIGHT,
    NR_OF_SQUARES, PAWN_SQUARES, RANK_1, RANK_2, RANK_7, RANK_8, WHITE,
};
use crate::print;
use crate::utils::{create_bb_files, create_bb_ranks};
use masks::{create_bishop_mask, create_rook_mask};

const WHITE_BLACK: usize = 2;
const EMPTY: Bitboard = 0;
const NSQ: usize = NR_OF_SQUARES as usize;

// This is a total sum of all rook or bishop blocker permutations per square.
// const ROOK_TABLE_SIZE: u32 = 102400;
// const BISHOP_TABLE_SIZE: u32 = 5248;

pub struct Magics {
    _king: [Bitboard; NSQ],
    _knight: [Bitboard; NSQ],
    _pawns: [[Bitboard; NSQ]; WHITE_BLACK],
}

impl Default for Magics {
    fn default() -> Magics {
        Magics {
            _king: [EMPTY; NSQ],
            _knight: [EMPTY; NSQ],
            _pawns: [[EMPTY; NSQ]; WHITE_BLACK],
        }
    }
}

impl Magics {
    pub fn initialize(&mut self) {
        let files = create_bb_files();
        let ranks = create_bb_ranks();

        self.init_king(&files, &ranks);
        self.init_knight(&files, &ranks);
        self.init_pawns(&files);
        self.init_magics();
    }

    pub fn get_non_slider_attacks(&self, piece: Piece, square: u8) -> Bitboard {
        match piece {
            KING => self._king[square as usize],
            KNIGHT => self._knight[square as usize],
            _ => 0,
        }
    }

    pub fn get_pawn_attacks(&self, side: Side, square: u8) -> Bitboard {
        self._pawns[side][square as usize]
    }

    fn init_king(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let square = 1u64 << sq;
            let moves = (square & !files[FILE_A] & !ranks[RANK_8]) << 7
                | (square & !ranks[RANK_8]) << 8
                | (square & !files[FILE_H] & !ranks[RANK_8]) << 9
                | (square & !files[FILE_H]) << 1
                | (square & !files[FILE_H] & !ranks[RANK_1]) >> 7
                | (square & !ranks[RANK_1]) >> 8
                | (square & !files[FILE_A] & !ranks[RANK_1]) >> 9
                | (square & !files[FILE_A]) >> 1;
            self._king[sq as usize] = moves;
        }
    }

    fn init_knight(&mut self, files: &[Bitboard; 8], ranks: &[Bitboard; 8]) {
        for sq in ALL_SQUARES {
            let square = 1u64 << sq;
            let moves = (square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_A]) << 15
                | (square & !ranks[RANK_8] & !ranks[RANK_7] & !files[FILE_H]) << 17
                | (square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_8]) << 6
                | (square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_8]) << 10
                | (square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_A]) >> 17
                | (square & !ranks[RANK_1] & !ranks[RANK_2] & !files[FILE_H]) >> 15
                | (square & !files[FILE_A] & !files[FILE_B] & !ranks[RANK_1]) >> 10
                | (square & !files[FILE_G] & !files[FILE_H] & !ranks[RANK_1]) >> 6;
            self._knight[sq as usize] = moves;
        }
    }

    fn init_pawns(&mut self, files: &[Bitboard; 8]) {
        for sq in PAWN_SQUARES {
            let square = 1u64 << sq;
            let w = (square & !files[FILE_A]) << 7 | (square & !files[FILE_H]) << 9;
            let b = (square & !files[FILE_A]) >> 9 | (square & !files[FILE_H]) >> 7;
            self._pawns[WHITE][sq as usize] = w;
            self._pawns[BLACK][sq as usize] = b;
        }
    }

    fn init_magics(&mut self) {
        // TODO: Implement magics
        // for i in ALL_SQUARES {
        //     println!("square: {}", i);
        //     let x = create_rook_mask(i);
        //     print::bitboard(x, Some(i));
        // }

        // for i in ALL_SQUARES {
        //     println!("square: {}", i);
        //     let x = create_bishop_mask(i);
        //     print::bitboard(x, Some(i));
        // }
    }
}
