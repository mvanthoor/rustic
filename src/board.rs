use crate::defs::*;

pub struct Board {
    pub bb_w: [Bitboard; 6],
    pub bb_b: [Bitboard; 6],
    pub bb_mask: [Mask; 3],
    pub turn: Color,
    pub castling: u8,
    pub en_passant: i8,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Default for Board {
    fn default() -> Board {
        Board {
            bb_w: [0; 6],
            bb_b: [0; 6],
            bb_mask: [[0; 64]; 3],
            turn: Color::WHITE,
            castling: 0,
            en_passant: -1,
            halfmove_clock: 0,
            fullmove_number: 0,
        }
    }
}

impl Board {
    pub fn reset(&mut self) {
        self.bb_w = [0; 6];
        self.bb_b = [0; 6];
        self.turn = Color::WHITE;
        self.castling = 0;
        self.en_passant = -1;
        self.halfmove_clock = 0;
        self.fullmove_number = 0;
    }
}
