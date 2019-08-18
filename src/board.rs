use crate::defs::*;
use crate::fen;
use crate::masks;

pub struct Board {
    pub bb_w: [Bitboard; NR_OF_BB_NORMAL],
    pub bb_b: [Bitboard; NR_OF_BB_NORMAL],
    pub bb_pieces: [Bitboard; NR_OF_BB_PIECES],
    pub bb_files: [Bitboard; NR_OF_BB_FILES],
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
            bb_pieces: [0; NR_OF_BB_PIECES],
            bb_files: [0; NR_OF_BB_FILES],
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
    fn create_piece_bitboards(&mut self) {
        for (bb_w, bb_b) in self.bb_w.iter().zip(self.bb_b.iter()) {
            self.bb_pieces[BB_PIECES_W] ^= bb_w;
            self.bb_pieces[BB_PIECES_B] ^= bb_b;
        }
        self.bb_pieces[BB_PIECES_ALL] ^= self.bb_pieces[BB_PIECES_W] ^ self.bb_pieces[BB_PIECES_B];
        self.bb_pieces[BB_PIECES_PAWNS] ^= self.bb_w[BB_P] ^ self.bb_b[BB_P];
    }

    fn create_file_bitboards(&mut self) {
        // Set LSB of each byte to 1 to create file 0.
        let file: u64 = 0x0101_0101_0101_0101;
        for (i, f) in self.bb_files.iter_mut().enumerate() {
            *f = file << i;
        }
    }

    pub fn reset(&mut self) {
        self.bb_w = [0; NR_OF_BB_NORMAL];
        self.bb_b = [0; NR_OF_BB_NORMAL];
        self.bb_pieces = [0; NR_OF_BB_PIECES];
        self.bb_files = [0; NR_OF_BB_FILES];
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
        self.create_piece_bitboards();
        self.create_file_bitboards();
        masks::create(self);
    }
}
