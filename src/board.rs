use crate::defs::*;

pub struct Board {
    pub bb_w: [Bitboard; 6],
    pub bb_b: [Bitboard; 6],
    pub turn: Color,
    pub castling: u8,
    pub en_passant: i8,
    pub fifty_moves: u8,
    pub full_moves: u16,
    pub ascii: [char; 64],
}

impl Default for Board {
    fn default() -> Board {
        Board {
            bb_w: [0; 6],
            bb_b: [0; 6],
            turn: Color::WHITE,
            castling: 0,
            en_passant: -1,
            fifty_moves: 0,
            full_moves: 0,
            ascii: [ASCII_EMPTY_SQUARE; 64],
        }
    }
}

impl Board {
    fn put_piece_onto_ascii(&mut self, bitboard: Bitboard, piece: char) {
        for i in 0..64 {
            if bitboard >> i & 1 == 1 {
                self.ascii[i] = piece;
            }
        }
    }

    fn bitboards_to_ascii(&mut self) {
        self.ascii = [ASCII_EMPTY_SQUARE; 64];
        for i in 0..6 {
            match i {
                BB_K => {
                    self.put_piece_onto_ascii(self.bb_w[BB_K], CHAR_WK);
                    self.put_piece_onto_ascii(self.bb_b[BB_K], CHAR_BK);
                }
                BB_Q => {
                    self.put_piece_onto_ascii(self.bb_w[BB_Q], CHAR_WQ);
                    self.put_piece_onto_ascii(self.bb_b[BB_Q], CHAR_BQ);
                }
                BB_R => {
                    self.put_piece_onto_ascii(self.bb_w[BB_R], CHAR_WR);
                    self.put_piece_onto_ascii(self.bb_b[BB_R], CHAR_BR);
                }
                BB_B => {
                    self.put_piece_onto_ascii(self.bb_w[BB_B], CHAR_WB);
                    self.put_piece_onto_ascii(self.bb_b[BB_B], CHAR_BB);
                }
                BB_N => {
                    self.put_piece_onto_ascii(self.bb_w[BB_N], CHAR_WN);
                    self.put_piece_onto_ascii(self.bb_b[BB_N], CHAR_BN);
                }
                BB_P => {
                    self.put_piece_onto_ascii(self.bb_w[BB_P], CHAR_WP);
                    self.put_piece_onto_ascii(self.bb_b[BB_P], CHAR_BP);
                }
                _ => (),
            }
        }
    }

    pub fn reset(&mut self) {
        self.bb_w = [0; 6];
        self.bb_b = [0; 6];
        self.turn = Color::WHITE;
        self.castling = 0;
        self.en_passant = -1;
        self.fifty_moves = 0;
        self.full_moves = 0;
        self.ascii = [ASCII_EMPTY_SQUARE; 64];
    }

    pub fn print(&mut self) {
        let coordinate_alpha: &str = "ABCDEFGH";
        let mut coordinate_digit = 8;

        self.bitboards_to_ascii();
        println!();
        for r in (RANK_1..=RANK_8).rev() {
            print!("{}   ", coordinate_digit);
            for f in FILE_A..=FILE_H {
                let square = (r * 8 + f) as usize;
                print!("{} ", self.ascii[square]);
            }
            println!();
            coordinate_digit -= 1;
        }
        println!();
        print!("    ");
        for c in coordinate_alpha.chars() {
            print!("{} ", c);
        }
        println!();
    }
}
