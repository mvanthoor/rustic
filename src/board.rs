use crate::defines::*;
use crate::fen;

pub struct Board {
    pub bb_w: [Bitboard; BITBOARDS_PER_SIDE as usize],
    pub bb_b: [Bitboard; BITBOARDS_PER_SIDE as usize],
    pub bb_pieces: [Bitboard; BITBOARDS_FOR_PIECES as usize],
    pub bb_files: [Bitboard; BITBOARDS_PER_FILE as usize],
    pub active_color: u8,
    pub castling: u8,
    pub en_passant: i8,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Default for Board {
    fn default() -> Board {
        Board {
            bb_w: [0; BITBOARDS_PER_SIDE as usize],
            bb_b: [0; BITBOARDS_PER_SIDE as usize],
            bb_pieces: [0; BITBOARDS_FOR_PIECES as usize],
            bb_files: [0; BITBOARDS_PER_FILE as usize],
            active_color: WHITE as u8,
            castling: 0,
            en_passant: -1,
            halfmove_clock: 0,
            fullmove_number: 0,
        }
    }
}

impl Board {
    pub fn create_piece_bitboards(&mut self) {
        let white = WHITE as usize;
        let black = BLACK as usize;
        // Iterate through all white and black bitboards.
        for (bb_w, bb_b) in self.bb_w.iter().zip(self.bb_b.iter()) {
            // Combine all white bitboards into one, having all white pieces,
            // Also combine all black bitboards into one, having all black pieces
            self.bb_pieces[white] ^= bb_w;
            self.bb_pieces[black] ^= bb_b;
        }
    }

    pub fn create_file_bitboards(&mut self) {
        /*
            Remember: Square A1 is on the lower left corner,
            but it is the LSB (Least Significant bit) in a
            64-bit integer, thus at the right hand side when
            writing the integer as binary.
            As a result, the bits set for  File A are shifted
            from RIGHT to LEFT (using <<) inside the integer.
        */

        // Set the LSB for each of the 8 bytes in the 64-bit integer.
        // This will mark the A-file.
        let file_a: u64 = 0x0101_0101_0101_0101;

        // Shift the bits, marking each file.
        for (i, file) in self.bb_files.iter_mut().enumerate() {
            *file = file_a << i;
        }
    }

    pub fn reset(&mut self) {
        self.bb_w = [0; BITBOARDS_PER_SIDE as usize];
        self.bb_b = [0; BITBOARDS_PER_SIDE as usize];
        self.bb_pieces = [0; BITBOARDS_FOR_PIECES as usize];
        self.bb_files = [0; BITBOARDS_PER_FILE as usize];
        self.active_color = WHITE as u8;
        self.castling = 0;
        self.en_passant = -1;
        self.halfmove_clock = 0;
        self.fullmove_number = 0;
        self.create_file_bitboards();
    }

    pub fn initialize(&mut self, fen: &str) {
        fen::read(fen, self);
    }

    pub fn piece(&self, piece: Piece, side: Side) -> Bitboard {
        debug_assert!(piece <= 5, "Not a piece! : {}", piece);
        debug_assert!(side == 0 || side == 1, "Not a side! : {}", side);
        match side {
            WHITE => self.bb_w[piece],
            BLACK => self.bb_b[piece],
            _ => 0,
        }
    }

    pub fn occupancy(&self) -> Bitboard {
        self.bb_pieces[WHITE] ^ self.bb_pieces[BLACK]
    }
}
