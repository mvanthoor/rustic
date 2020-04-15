use crate::defs::{Piece, BISHOP, KNIGHT, QUEEN, ROOK};

pub const ASCII_VALUE_OF_LOWERCASE_A: u8 = 97;
pub const ASCII_VALUE_OF_1: u8 = 49;
pub const COORDINATE_LETTERS: &str = "abcdefgh";
pub const COORDINATE_NUMBERS: &str = "12345678";

pub fn strip_newline(input: &mut String) {
    for _ in 0..2 {
        let c = input.chars().next_back();
        let cr = Some('\r') == c;
        let lf = Some('\n') == c;

        if cr || lf {
            input.pop();
        }
    }
}

pub fn algebraic_square_to_number(algebraic_move: &str) -> Result<u8, ()> {
    let length = algebraic_move.len();
    let mut result: Result<u8, ()> = Err(());

    if length == 2 {
        let mut file = 0;
        let mut rank = 0;
        let mut char_ok = 0;

        for (index, c) in algebraic_move.to_lowercase().chars().enumerate() {
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
            let square_nr = (rank * 8) + file;
            result = Ok(square_nr);
        }
    }
    result
}

pub fn piece_letter_to_number(piece_letter: char) -> Result<Piece, ()> {
    let mut result: Result<Piece, ()> = Err(());
    if let Some(p) = piece_letter.to_lowercase().next() {
        match p {
            'q' => result = Ok(QUEEN),
            'r' => result = Ok(ROOK),
            'b' => result = Ok(BISHOP),
            'n' => result = Ok(KNIGHT),
            _ => (),
        }
    }
    result
}
