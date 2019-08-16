use crate::defs::*;
use crate::fen;
use crate::masks;

pub struct Board {
    pub bb_w: [Bitboard; NR_OF_BB_NORMAL],
    pub bb_b: [Bitboard; NR_OF_BB_NORMAL],
    pub bb_special: [Bitboard; NR_OF_BB_SPECIAL],
    pub bb_mask: [Mask; NR_OF_BB_MASK],
    pub turn: Color,
    pub castling: u8,
    pub en_passant: i8,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Default for Board {
    fn default() -> Board {
        Board {
            bb_w: [0; NR_OF_BB_NORMAL],
            bb_b: [0; NR_OF_BB_NORMAL],
            bb_special: [0; NR_OF_BB_SPECIAL],
            bb_mask: [[0; 64]; NR_OF_BB_MASK],
            turn: Color::WHITE,
            castling: 0,
            en_passant: -1,
            halfmove_clock: 0,
            fullmove_number: 0,
        }
    }
}

impl Board {
    fn create_special_bitboards(&mut self) {
        for i in 0..NR_OF_BB_NORMAL {
            self.bb_special[BB_SPECIAL_W] ^= self.bb_w[i];
            self.bb_special[BB_SPECIAL_B] ^= self.bb_b[i];
        }
        self.bb_special[BB_SPECIAL_ALL] ^= self.bb_special[BB_SPECIAL_W];
        self.bb_special[BB_SPECIAL_ALL] ^= self.bb_special[BB_SPECIAL_B];
    }

    pub fn reset(&mut self) {
        self.bb_w = [0; NR_OF_BB_NORMAL];
        self.bb_b = [0; NR_OF_BB_NORMAL];
        self.bb_special = [0; NR_OF_BB_SPECIAL];
        self.bb_mask = [[0; 64]; NR_OF_BB_MASK];
        self.turn = Color::WHITE;
        self.castling = 0;
        self.en_passant = -1;
        self.halfmove_clock = 0;
        self.fullmove_number = 0;
    }

    pub fn create_start_position(&mut self) {
        self.reset();
        fen::read(FEN_START_POSITION, self);
        self.create_special_bitboards();
        masks::create(self);
    }
}
