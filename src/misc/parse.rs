use crate::board::defs::Pieces;
use crate::defs::{Piece, Square};

pub const ASCII_VALUE_OF_LOWERCASE_A: u8 = 97;
pub const ASCII_VALUE_OF_1: u8 = 49;
pub const COORDINATE_LETTERS: &str = "abcdefgh";
pub const COORDINATE_NUMBERS: &str = "12345678";

#[allow(dead_code)]
pub fn strip_newline(input: &mut String) {
    let mut s = input.trim().to_string();
    for _ in 0..2 {
        let c = s.chars().next_back();
        let cr = Some('\r') == c;
        let lf = Some('\n') == c;

        if cr || lf {
            s.pop();
        }
    }
    *input = s;
}

pub fn algebraic_square_to_number(algebraic_square: &str) -> Result<Square, ()> {
    let length = algebraic_square.len();
    let mut result: Result<Square, ()> = Err(());

    if length == 2 {
        let mut file = 0;
        let mut rank = 0;
        let mut char_ok = 0;

        for (index, c) in algebraic_square.to_lowercase().chars().enumerate() {
            if index == 0 && COORDINATE_LETTERS.contains(c) {
                file = (c as u8) - ASCII_VALUE_OF_LOWERCASE_A;
                char_ok += 1;
            }
            if index == 1 && COORDINATE_NUMBERS.contains(c) {
                rank = (c as u8) - ASCII_VALUE_OF_1;
                char_ok += 1;
            }
        }

        if char_ok == length {
            let square_nr = ((rank * 8) + file) as Square;
            result = Ok(square_nr);
        }
    }
    result
}

#[allow(dead_code)]
pub fn promotion_piece_letter_to_number(piece_letter: char) -> Result<Piece, ()> {
    let mut result: Result<Piece, ()> = Err(());
    if let Some(p) = piece_letter.to_lowercase().next() {
        match p {
            'q' => result = Ok(Pieces::QUEEN),
            'r' => result = Ok(Pieces::ROOK),
            'b' => result = Ok(Pieces::BISHOP),
            'n' => result = Ok(Pieces::KNIGHT),
            _ => (),
        }
    }
    result
}
