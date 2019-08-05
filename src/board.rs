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
    pub fn print(&self) {
        let coordinate_alpha = "ABCDEFGH";
        let mut coordinate_digit = 8;
        for i in 0..8 {
            print!("{} ", coordinate_digit);
            for j in 0..8 {
                let x = (i * 8) + j;
                print!("{} ", self.ascii[x]);
            }
            coordinate_digit -= 1;
            println!();
        }
        print!("  ");
        for c in coordinate_alpha.chars() {
            print!("{} ", c);
        }
        println!();
    }
}
